use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
use tempfile::TempDir;
use wb_registry::{Policy, Registry, ReleasePin};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copy(&e.path(), &to.join(e.file_name()));
        } else {
            fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}
struct Test {
    tmp: TempDir,
    source: PathBuf,
    registry: Registry,
    policy: Policy,
}
impl Test {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source");
        copy(&root().join("target/registry-fixtures/pure-echo"), &source);
        let python = std::env::var_os("REGISTRY_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let out = Command::new("python3")
                    .args(["-c", "import sys; print(sys.executable)"])
                    .output()
                    .unwrap();
                PathBuf::from(String::from_utf8(out.stdout).unwrap().trim())
            });
        let policy = Policy::local(
            vec![tmp.path().into()],
            python,
            root().join("tools/registry_validate.py"),
            PathBuf::from(env!("CARGO_BIN_EXE_wb-registry-inspect")),
        );
        let registry = Registry::open(&tmp.path().join("store"), policy.clone()).unwrap();
        Self {
            tmp,
            source,
            registry,
            policy,
        }
    }
    fn manifest(&self) -> Value {
        serde_json::from_slice(&fs::read(self.source.join("component.json")).unwrap()).unwrap()
    }
    fn edit(&self, f: impl FnOnce(&mut Value)) {
        let mut m = self.manifest();
        f(&mut m);
        fs::write(
            self.source.join("component.json"),
            serde_json::to_vec(&m).unwrap(),
        )
        .unwrap();
    }
    fn turtle(&self, old: &str, new: &str) {
        let p = self.source.join("capabilities.ttl");
        let s = fs::read_to_string(&p).unwrap();
        assert!(s.contains(old));
        fs::write(p, s.replace(old, new)).unwrap();
    }
    fn reject(&self, code: &str) {
        let e = self.registry.admit_directory(&self.source).unwrap_err();
        assert_eq!(e.code, code, "{e}");
        assert!(!e.location.contains(self.tmp.path().to_str().unwrap()));
        assert!(self.registry.discover(None).unwrap().is_empty());
    }
    fn module(&self, bytes: &[u8]) {
        let old = self.manifest()["runtime"]["sha256"]
            .as_str()
            .unwrap()
            .to_owned();
        let digest = format!("{:x}", Sha256::digest(bytes));
        fs::write(self.source.join("logic.wasm"), bytes).unwrap();
        self.edit(|m| m["runtime"]["sha256"] = json!(digest));
        self.turtle(&old, &digest);
    }
    fn object(&self, digest: &str) -> PathBuf {
        self.tmp.path().join("store/objects").join(digest)
    }
}
#[test]
fn reg_001_compiled_pure_bundle_and_kernel_handle() {
    let t = Test::new();
    let receipt = t.registry.admit_directory(&t.source).unwrap();
    let h = t.registry.resolve_exact(&receipt.pin).unwrap();
    let report = root().join("target/abi-evidence/registry");
    fs::create_dir_all(&report).unwrap();
    fs::write(
        report.join("receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    fs::copy(
        t.object(&receipt.record_digest).join("record.json"),
        report.join("record.json"),
    )
    .unwrap();
    assert_eq!(
        t.registry.read_verified_artifact(&h).unwrap(),
        fs::read(t.source.join("logic.wasm")).unwrap()
    );
    let kernel = wb_kernel::runtime::Kernel::new(wb_wasm_host::Limits::default(), 4, 4).unwrap();
    let pin = kernel.cache_registry_release(&t.registry, &h).unwrap();
    assert_eq!(pin.sha256, receipt.pin.sha256);
    assert_eq!(kernel.in_flight(), 0);
    let other = Registry::open(&t.tmp.path().join("other"), t.policy.clone()).unwrap();
    assert_eq!(
        other.read_verified_artifact(&h).unwrap_err().code,
        "INTEGRITY_FAILURE"
    );
}
#[test]
fn reg_002_strict_manifest() {
    for mutation in [0, 1, 2, 3, 4, 5] {
        let t = Test::new();
        match mutation {
            0 => t.edit(|m| m["surprise"] = json!(true)),
            1 => {
                let p = t.source.join("component.json");
                let s = fs::read_to_string(&p).unwrap();
                fs::write(p, s.replacen('{', "{\"manifestVersion\":\"0.1\",", 1)).unwrap();
            }
            2 => t.edit(|m| m["version"] = json!("1.0")),
            3 => t.edit(|m| m["version"] = json!("1.0.0-01")),
            4 => t.edit(|m| m["runtime"]["unexpected"] = json!(1)),
            _ => t.edit(|m| {
                let p = m["ports"]["inputs"][0].clone();
                m["ports"]["inputs"].as_array_mut().unwrap().push(p);
            }),
        }
        t.reject("INVALID_MANIFEST");
    }
    let t = Test::new();
    t.edit(|m| m["manifestVersion"] = json!("9"));
    t.reject("UNSUPPORTED_VERSION");
}
#[test]
fn reg_003_paths_links_and_nonregular() {
    for path in [
        "../secret",
        "/etc/passwd",
        "schemas/../input.json",
        "schemas\\input.json",
        "schemas//input.json",
        "schemas/.",
    ] {
        let t = Test::new();
        t.edit(|m| m["ports"]["inputs"][0]["schema"] = json!(path));
        assert!(t.registry.admit_directory(&t.source).is_err());
        assert!(t.registry.discover(None).unwrap().is_empty());
    }
    for kind in 0..5 {
        let t = Test::new();
        let p = t.source.join("schemas/input.json");
        match kind {
            0 => {
                fs::remove_file(&p).unwrap();
                std::os::unix::fs::symlink("/etc/passwd", p).unwrap();
            }
            1 => {
                fs::hard_link(&p, t.source.join("alias")).unwrap();
            }
            2 => {
                let path = std::ffi::CString::new(t.source.join("fifo").to_str().unwrap()).unwrap();
                assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
            }
            3 => {
                let _socket =
                    std::os::unix::net::UnixListener::bind(t.source.join("socket")).unwrap();
                t.reject("INVALID_PATH");
                continue;
            }
            _ => {
                fs::rename(t.source.join("schemas"), t.tmp.path().join("schemas")).unwrap();
                std::os::unix::fs::symlink(t.tmp.path().join("schemas"), t.source.join("schemas"))
                    .unwrap();
            }
        }
        t.reject("INVALID_PATH");
    }
    let t = Test::new();
    assert_eq!(
        t.registry
            .admit_directory(Path::new("/etc"))
            .unwrap_err()
            .code,
        "CAPABILITY_DENIED"
    );
}
#[test]
fn reg_004_schemas_missing_malformed_local_refs_and_literal_preservation() {
    for path in ["logic.wasm", "schemas/input.json"] {
        let t = Test::new();
        fs::remove_file(t.source.join(path)).unwrap();
        t.reject("MISSING_FILE");
    }
    for schema in [
        json!({"type":"nope"}),
        json!({"$ref":"#"}),
        json!({"$ref":"#/missing"}),
        json!({"$ref":"output.json"}),
    ] {
        let t = Test::new();
        fs::write(t.source.join("schemas/input.json"), schema.to_string()).unwrap();
        // output.json is valid: exercise actual local-file references separately.
        if schema == json!({"$ref":"output.json"}) {
            t.registry.admit_directory(&t.source).unwrap();
        } else {
            t.reject("SCHEMA_VIOLATION");
        }
    }
    let t = Test::new();
    let literal =
        json!({"const":{"$id":"marker","$ref":"literal"},"properties":{"$id":{"type":"string"}}});
    fs::write(t.source.join("schemas/input.json"), literal.to_string()).unwrap();
    let receipt = t.registry.admit_directory(&t.source).unwrap();
    let h = t.registry.resolve_exact(&receipt.pin).unwrap();
    let (checked, _) = t.registry.read_verified_bundle(&h).unwrap();
    assert_eq!(checked.schemas["schemas/input.json"], literal);
    let t = Test::new();
    let schema = json!({"$defs":{"base":{"properties":{"x":{"type":"integer"}}}},"$ref":"#/$defs/base","unevaluatedProperties":false});
    fs::write(t.source.join("schemas/input.json"), schema.to_string()).unwrap();
    let receipt = t.registry.admit_directory(&t.source).unwrap();
    let h = t.registry.resolve_exact(&receipt.pin).unwrap();
    let (checked, _) = t.registry.read_verified_bundle(&h).unwrap();
    let validator = jsonschema_validator(&checked.schemas["schemas/input.json"]);
    assert!(validator.is_valid(&json!({"x":1})));
    assert!(!validator.is_valid(&json!({"y":1})));
}
#[test]
fn reg_005_digest_and_post_install_integrity() {
    let t = Test::new();
    fs::write(t.source.join("logic.wasm"), b"tamper").unwrap();
    t.reject("DIGEST_MISMATCH");
    let t = Test::new();
    let r = t.registry.admit_directory(&t.source).unwrap();
    let h = t.registry.resolve_exact(&r.pin).unwrap();
    fs::write(
        t.object(&r.record_digest).join("payload/logic.wasm"),
        b"tamper",
    )
    .unwrap();
    assert_eq!(
        t.registry.read_verified_artifact(&h).unwrap_err().code,
        "INTEGRITY_FAILURE"
    );
    assert_eq!(
        t.registry.resolve_exact(&r.pin).unwrap_err().code,
        "INTEGRITY_FAILURE"
    );
    assert!(t.registry.discover(None).unwrap().is_empty());
    Registry::open(&t.tmp.path().join("store"), t.policy.clone()).unwrap();
    assert_eq!(
        t.registry.admit_directory(&t.source).unwrap_err().code,
        "INTEGRITY_FAILURE"
    );
}
#[test]
fn reg_006_turtle_and_shacl() {
    let t = Test::new();
    fs::write(t.source.join("capabilities.ttl"), "not turtle").unwrap();
    t.reject("RDF_INVALID");
    let t = Test::new();
    t.turtle("a wb:Artifact", "a wb:Component");
    t.reject("RDF_INVALID");
    let t = Test::new();
    t.turtle("wb:portName \"input\"", "wb:portName 42");
    t.reject("RDF_INVALID");
}
#[test]
fn reg_007_each_semantic_mismatch() {
    for (old, new) in [
        (
            "org.workbench.registry.echo",
            "org.workbench.registry.other",
        ),
        ("0.1.0", "0.2.0"),
        ("\"input\"", "\"other\""),
        ("schemas/input.json", "schemas/output.json"),
        ("spike#Text>", "spike#Other>"),
        ("spike#TextUnit>", "spike#OtherUnit>"),
        ("logic.wasm", "other.wasm"),
    ] {
        let t = Test::new();
        t.turtle(old, new);
        t.reject("SEMANTIC_MISMATCH");
    }
    let t = Test::new();
    let digest = t.manifest()["runtime"]["sha256"]
        .as_str()
        .unwrap()
        .to_owned();
    t.turtle(&digest, &"0".repeat(64));
    t.reject("SEMANTIC_MISMATCH");
    let t = Test::new();
    t.turtle("wb:hasInputPort ex:input", "wb:hasInputPort ex:output");
    t.reject("RDF_INVALID");
}
#[test]
fn reg_008_ui_explicitly_blocked() {
    for ui in [
        json!({"uiVersion":"0.1","root":{"type":"form","children":[]}}),
        json!({"qml":"evil()"}),
        json!({"uiVersion":"0.1","root":{"type":"readout","id":"x","label":"X","bind":"missing.bad"}}),
    ] {
        let t = Test::new();
        fs::write(t.source.join("ui.json"), ui.to_string()).unwrap();
        t.edit(|m| m["ui"] = json!("ui.json"));
        t.reject("SCHEMA_VIOLATION");
    }
    let t = Test::new();
    t.registry.admit_directory(&t.source).unwrap();
}
#[test]
fn reg_009_permissions_never_granted() {
    let t = Test::new();
    t.edit(|m| m["permissions"] = json!([{"name":"filesystem.read","scope":{}}]));
    t.reject("CAPABILITY_DENIED");
    let t = Test::new();
    t.turtle(
        "wb:providesCapability ex:Echo",
        "wb:requestsPermission ex:permission ; wb:providesCapability ex:Echo",
    );
    t.reject("CAPABILITY_DENIED");
}
#[test]
fn reg_010_abi_encoding_exports_type_and_imports() {
    for bytes in [wat::parse_str("(module)").unwrap(),b"broken".to_vec(),wat::parse_str("(component)").unwrap(),wat::parse_str(r#"(component (core module $m (func (export "compute"))) (core instance $i (instantiate $m)) (func $f (canon lift (core func $i "compute"))) (instance $e (export "compute" (func $f))) (export "workbench:component/computation@0.1.0" (instance $e)))"#).unwrap(),fs::read(root().join("target/abi-guests/wasm32-wasip1/release/abi_denied_wasi.wasm")).unwrap()] {
        let t=Test::new();t.module(&bytes);t.reject("ABI_INCOMPATIBLE");
    }
}
#[test]
fn reg_011_idempotent() {
    let t = Test::new();
    let a = t.registry.admit_directory(&t.source).unwrap();
    let b = t.registry.admit_directory(&t.source).unwrap();
    assert_eq!(a, b);
    assert_eq!(t.registry.discover(None).unwrap().len(), 1);
}
#[test]
fn reg_012_conflict_original_intact() {
    let t = Test::new();
    let a = t.registry.admit_directory(&t.source).unwrap();
    t.module(
        &fs::read(root().join("target/abi-guests/wasm32-unknown-unknown/release/abi_integer.wasm"))
            .unwrap(),
    );
    assert_eq!(
        t.registry.admit_directory(&t.source).unwrap_err().code,
        "RELEASE_CONFLICT"
    );
    assert!(t.registry.resolve_exact(&a.pin).is_ok());
    assert_eq!(t.registry.discover(None).unwrap().len(), 1);
}
#[test]
fn reg_013_exact_pins_no_fallback() {
    let t = Test::new();
    let a = t.registry.admit_directory(&t.source).unwrap();
    for p in [
        ReleasePin {
            version: "0.2.0".into(),
            ..a.pin.clone()
        },
        ReleasePin {
            sha256: "0".repeat(64),
            ..a.pin.clone()
        },
        ReleasePin {
            id: "org.missing.example".into(),
            ..a.pin.clone()
        },
    ] {
        assert_eq!(
            t.registry.resolve_exact(&p).unwrap_err().code,
            "COMPONENT_NOT_FOUND"
        );
    }
    t.edit(|m| m["version"] = json!("0.2.0"));
    t.turtle("0.1.0", "0.2.0");
    let b = t.registry.admit_directory(&t.source).unwrap();
    assert_eq!(a.pin.sha256, b.pin.sha256);
    assert_eq!(t.registry.discover(None).unwrap().len(), 2);
    assert!(t.registry.resolve_exact(&a.pin).is_ok());
}
#[test]
fn reg_014_real_sigkill_all_commit_boundaries() {
    for point in [
        "Copied",
        "Validated",
        "FilesSynced",
        "ObjectRenamed",
        "IndexWritten",
        "IndexRenamed",
        "Committed",
    ] {
        let t = Test::new();
        let status = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "reg_014_child"])
            .env("REG_CRASH_SOURCE", &t.source)
            .env("REG_CRASH_STORE", t.tmp.path().join("store"))
            .env("REG_CRASH_POINT", point)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(libc::SIGKILL));
        let r = Registry::open(&t.tmp.path().join("store"), t.policy.clone()).unwrap();
        let visible = r.discover(None).unwrap();
        let committed = matches!(point, "IndexRenamed" | "Committed");
        assert_eq!(visible.len(), usize::from(committed), "{point}");
        for summary in visible {
            let h = r.resolve_exact(&summary.pin).unwrap();
            r.read_verified_artifact(&h).unwrap();
        }
        assert_eq!(
            fs::read_dir(t.tmp.path().join("store/.staging"))
                .unwrap()
                .count(),
            0
        );
        assert_eq!(
            fs::read_dir(t.tmp.path().join("store/objects"))
                .unwrap()
                .count(),
            usize::from(committed)
        );
        r.admit_directory(&t.source).unwrap();
        assert_eq!(r.discover(None).unwrap().len(), 1);
    }
}
#[test]
fn reg_014_child() {
    let Some(source) = std::env::var_os("REG_CRASH_SOURCE") else {
        return;
    };
    let t = Test::new();
    let store = PathBuf::from(std::env::var_os("REG_CRASH_STORE").unwrap());
    let mut policy = t.policy.clone();
    policy.allowed_roots = vec![Path::new(&source).parent().unwrap().into()];
    let r = Registry::open(&store, policy).unwrap();
    let point = std::env::var("REG_CRASH_POINT").unwrap();
    r.admit_observed(Path::new(&source), |checkpoint| {
        if format!("{checkpoint:?}") == point {
            unsafe {
                libc::kill(libc::getpid(), libc::SIGKILL);
            }
        }
    })
    .unwrap();
    panic!("checkpoint was not exercised");
}
#[test]
fn reg_015_concurrent_installers_and_readers() {
    let t = Test::new();
    let mut threads = vec![];
    for _ in 0..4 {
        let root = t.tmp.path().join("store");
        let policy = t.policy.clone();
        let source = t.source.clone();
        threads.push(thread::spawn(move || {
            let r = Registry::open(&root, policy).unwrap();
            let a = r.admit_directory(&source).unwrap();
            let h = r.resolve_exact(&a.pin).unwrap();
            r.read_verified_artifact(&h).unwrap();
            a
        }));
    }
    let mut receipts = vec![];
    for task in threads {
        receipts.push(task.join().unwrap());
    }
    assert!(receipts.windows(2).all(|p| p[0] == p[1]));
    assert_eq!(t.registry.discover(None).unwrap().len(), 1);
}
#[test]
fn reg_016_discovery_only_committed_host_metadata() {
    let t = Test::new();
    t.edit(|m| m["extra"] = json!(1));
    t.reject("INVALID_MANIFEST");
    t.edit(|m| {
        m.as_object_mut().unwrap().remove("extra");
    });
    t.registry.admit_directory(&t.source).unwrap();
    let mut p = t.policy.clone();
    p.abi_worker = PathBuf::from("/missing-worker");
    p.python = PathBuf::from("/missing-python");
    let r = Registry::open(&t.tmp.path().join("store"), p).unwrap();
    assert_eq!(
        r.discover(Some("https://workbench.example/registry/Echo"))
            .unwrap()
            .len(),
        1
    );
    assert!(r.discover(Some("missing")).unwrap().is_empty());
}
#[test]
fn reg_017_dependencies_and_remote_refs_rejected() {
    for field in [
        "dependencies",
        "devDependencies",
        "externalDependencies",
        "packageImports",
    ] {
        let t = Test::new();
        t.edit(|m| m[field] = json!([]));
        t.reject("UNSUPPORTED_DEPENDENCY");
    }
    for reference in [
        "https://example.org/schema",
        "file:///etc/passwd",
        "../secret.json",
    ] {
        let t = Test::new();
        fs::write(
            t.source.join("schemas/input.json"),
            json!({"$ref":reference}).to_string(),
        )
        .unwrap();
        t.reject("INVALID_PATH");
    }
}
#[test]
fn reg_018_forged_evidence_cannot_elevate() {
    let t = Test::new();
    fs::create_dir(t.source.join("evidence")).unwrap();
    fs::write(
        t.source.join("evidence/tests.json"),
        r#"{"trust":"FormallyVerified","passed":true}"#,
    )
    .unwrap();
    t.edit(|m| m["evidence"] = json!(["evidence/tests.json"]));
    t.turtle(
        "wb:providesCapability ex:Echo",
        "wb:hasAssuranceClaim ex:claim ; wb:providesCapability ex:Echo",
    );
    let p = t.source.join("capabilities.ttl");
    let mut s = fs::read_to_string(&p).unwrap();
    s.push_str("\nex:claim a wb:AssuranceClaim ; wb:evidenceRef \"evidence/tests.json\" ; wb:assuranceLevel \"FormallyVerified\" .\n");
    fs::write(p, s).unwrap();
    t.registry.admit_directory(&t.source).unwrap();
    assert_eq!(t.registry.discover(None).unwrap()[0].trust, "unreviewed");
    let t = Test::new();
    t.turtle(
        "wb:providesCapability ex:Echo",
        "wb:hasAssuranceClaim ex:claim ; wb:providesCapability ex:Echo",
    );
    let p = t.source.join("capabilities.ttl");
    let mut s = fs::read_to_string(&p).unwrap();
    s.push_str("\nex:claim a wb:AssuranceClaim ; wb:evidenceRef \"missing.json\" .");
    fs::write(p, s).unwrap();
    t.reject("SEMANTIC_MISMATCH");
}
#[test]
fn reg_019_quotas_collisions_and_live_race() {
    for quota in 0..5 {
        let t = Test::new();
        let mut p = t.policy.clone();
        match quota {
            0 => p.max_files = 1,
            1 => p.max_file_bytes = 10,
            2 => p.max_total_bytes = 10,
            3 => p.max_depth = 0,
            _ => p.max_store_bytes = 10,
        };
        let r = Registry::open(&t.tmp.path().join("small"), p).unwrap();
        assert_eq!(
            r.admit_directory(&t.source).unwrap_err().code,
            "QUOTA_EXCEEDED"
        );
        assert!(r.discover(None).unwrap().is_empty());
    }
    for name in ["COMPONENT.JSON", "é.json", "script.qml"] {
        let t = Test::new();
        fs::write(t.source.join(name), b"x").unwrap();
        t.reject("INVALID_PATH");
    }
    let t = Test::new();
    let done = Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    let source = t.source.clone();
    let hidden = t.tmp.path().join("hidden");
    let worker = thread::spawn(move || {
        while !flag.load(Ordering::SeqCst) {
            if fs::rename(source.join("schemas"), &hidden).is_ok() {
                let _ = std::os::unix::fs::symlink("/etc", source.join("schemas"));
                let _ = fs::remove_file(source.join("schemas"));
                let _ = fs::rename(&hidden, source.join("schemas"));
            }
        }
    });
    for _ in 0..8 {
        let _ = t.registry.admit_directory(&t.source);
    }
    done.store(true, Ordering::SeqCst);
    worker.join().unwrap();
    for s in t.registry.discover(None).unwrap() {
        let h = t.registry.resolve_exact(&s.pin).unwrap();
        let (metadata, bytes) = t.registry.read_verified_bundle(&h).unwrap();
        assert_eq!(
            metadata.schemas["schemas/input.json"]["type"],
            json!("object")
        );
        assert!(bytes.starts_with(b"\0asm\x0d\0\x01\0"));
    }
    let long = Test::new();
    let version = format!("0.1.0-{}", "a".repeat(5000));
    long.edit(|m| m["version"] = json!(version));
    long.turtle("0.1.0", &version);
    long.reject("QUOTA_EXCEEDED");
    let output =
        wb_semantic::bounded_worker(Path::new("/bin/sleep"), &[Path::new("3")], 1).unwrap_err();
    assert_eq!(output, "QUOTA_EXCEEDED");
    assert_eq!(
        wb_semantic::bounded_worker(Path::new("/usr/bin/yes"), &[], 2).unwrap_err(),
        "QUOTA_EXCEEDED"
    );
    assert_eq!(
        wb_semantic::bounded_worker(
            &t.policy.python,
            &[Path::new("-c"), Path::new("x=bytearray(3*1024**3)")],
            2
        )
        .unwrap_err(),
        "QUOTA_EXCEEDED"
    );
}
#[test]
fn reg_020_frozen_abi_and_ipc_regression_workflow() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(fs::read(root().join("wit/component.wit")).unwrap())
        ),
        "cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(fs::read(root().join("crates/wb-wasm-host/tests/abi.rs")).unwrap())
        ),
        "263f74a47a8daf26197f8b3e947fc51e73e05da4030ac008ea416ed4d9542c95"
    );
    let ci = fs::read_to_string(root().join(".github/workflows/ci.yml")).unwrap();
    assert!(ci.contains("validate-registry.sh"));
    let script =
        fs::read_to_string(root().join("examples/wasm-components/validate-registry.sh")).unwrap();
    assert!(script.contains("validate-ipc.sh"));
}

fn jsonschema_validator(schema: &Value) -> jsonschema::Validator {
    jsonschema::validator_for(schema).unwrap()
}

#[test]
fn reg_010_trapping_initializer_is_not_executed() {
    use wasm_encoder::{
        Alias, Component, ComponentAliasSection, ComponentExportKind, ComponentExportSection,
        ComponentInstanceSection, InstanceSection, RawSection,
    };
    let t = Test::new();
    let mut wrapper = Component::new();
    let echo = fs::read(t.source.join("logic.wasm")).unwrap();
    wrapper.section(&RawSection { id: 4, data: &echo });
    let mut guest = ComponentInstanceSection::new();
    guest.instantiate(0, std::iter::empty::<(&str, ComponentExportKind, u32)>());
    wrapper.section(&guest);
    let mut aliases = ComponentAliasSection::new();
    aliases.alias(Alias::InstanceExport {
        instance: 0,
        kind: ComponentExportKind::Instance,
        name: "workbench:component/computation@0.1.0",
    });
    wrapper.section(&aliases);
    let mut exports = ComponentExportSection::new();
    exports.export(
        "workbench:component/computation@0.1.0",
        ComponentExportKind::Instance,
        1,
        None,
    );
    wrapper.section(&exports);
    let core = wat::parse_str("(module (func $start unreachable) (start $start))").unwrap();
    wrapper.section(&RawSection { id: 1, data: &core });
    let mut instances = InstanceSection::new();
    instances.instantiate(0, std::iter::empty::<(&str, wasm_encoder::ModuleArg)>());
    wrapper.section(&instances);
    let bytes = wrapper.finish();
    t.module(&bytes);
    let receipt = t.registry.admit_directory(&t.source).unwrap();
    let h = t.registry.resolve_exact(&receipt.pin).unwrap();
    assert_eq!(t.registry.read_verified_artifact(&h).unwrap(), bytes);
    // Prove the initializer really traps if instantiated: admission did not run it.
    let (checked, bytes) = t.registry.read_verified_bundle(&h).unwrap();
    let host = wb_wasm_host::Host::new(wb_wasm_host::Limits::default()).unwrap();
    let registered = host
        .register(&checked.manifest, &bytes, &checked.schemas)
        .unwrap();
    assert!(matches!(
        host.call(
            &registered,
            &wb_wasm_host::Message {
                port: "input".into(),
                payload_json: r#"{"text":"hello"}"#.into()
            }
        ),
        Err(wb_wasm_host::Failure::Trap)
    ));
}
#[test]
fn reg_011_idempotent_at_release_quota_and_after_reopen() {
    let t = Test::new();
    let mut p = t.policy.clone();
    p.max_releases = 1;
    let r = Registry::open(&t.tmp.path().join("one"), p.clone()).unwrap();
    let first = r.admit_directory(&t.source).unwrap();
    let r = Registry::open(&t.tmp.path().join("one"), p).unwrap();
    assert_eq!(r.admit_directory(&t.source).unwrap(), first);
    t.edit(|m| m["version"] = json!("0.2.0"));
    t.turtle("0.1.0", "0.2.0");
    assert_eq!(
        r.admit_directory(&t.source).unwrap_err().code,
        "QUOTA_EXCEEDED"
    );
    assert!(r.resolve_exact(&first.pin).is_ok());
}
#[test]
fn reg_005_corrupt_record_after_restart_stays_unavailable() {
    let t = Test::new();
    let receipt = t.registry.admit_directory(&t.source).unwrap();
    fs::write(t.object(&receipt.record_digest).join("record.json"), b"{}").unwrap();
    let r = Registry::open(&t.tmp.path().join("store"), t.policy.clone()).unwrap();
    assert!(r.discover(None).unwrap().is_empty());
    assert_eq!(
        r.resolve_exact(&receipt.pin).unwrap_err().code,
        "INTEGRITY_FAILURE"
    );
}
#[test]
fn reg_015_process_installer_child() {
    let Some(source) = std::env::var_os("REG_INSTALL_SOURCE") else {
        return;
    };
    let t = Test::new();
    let mut p = t.policy.clone();
    p.allowed_roots = vec![Path::new(&source).parent().unwrap().into()];
    let r = Registry::open(
        Path::new(&std::env::var_os("REG_INSTALL_STORE").unwrap()),
        p,
    )
    .unwrap();
    r.admit_directory(Path::new(&source)).unwrap();
}
#[test]
fn reg_015_independent_process_installers() {
    let t = Test::new();
    let mut children = vec![];
    for _ in 0..3 {
        children.push(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "reg_015_process_installer_child"])
                .env("REG_INSTALL_SOURCE", &t.source)
                .env("REG_INSTALL_STORE", t.tmp.path().join("store"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
    }
    for child in &mut children {
        assert!(child.wait().unwrap().success());
    }
    let items = t.registry.discover(None).unwrap();
    assert_eq!(items.len(), 1);
    let h = t.registry.resolve_exact(&items[0].pin).unwrap();
    t.registry.read_verified_artifact(&h).unwrap();
}
#[test]
fn reg_019_worker_parent_child() {
    let Some(pidfile) = std::env::var_os("REG_WORKER_PID") else {
        return;
    };
    let t = Test::new();
    let code = format!(
        "import os,time;open({:?},'w').write(str(os.getpid()));time.sleep(180)",
        pidfile.to_str().unwrap()
    );
    let _ = wb_semantic::bounded_worker(&t.policy.python, &[Path::new("-c"), Path::new(&code)], 30);
}
#[test]
fn reg_019_worker_dies_with_parent() {
    let t = Test::new();
    let pidfile = t.tmp.path().join("worker.pid");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "reg_019_worker_parent_child"])
        .env("REG_WORKER_PID", &pidfile)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !pidfile.exists() {
        assert!(std::time::Instant::now() < deadline);
        thread::sleep(std::time::Duration::from_millis(10));
    }
    let pid = loop {
        let s = fs::read_to_string(&pidfile).unwrap();
        if let Ok(pid) = s.parse::<u32>() {
            break pid;
        }
        thread::sleep(std::time::Duration::from_millis(10));
    };
    child.kill().unwrap();
    child.wait().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat"));
        if stat.is_err() || stat.unwrap().split_whitespace().nth(2) == Some("Z") {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "orphan worker remains live"
        );
        thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn reg_006_unknown_semantic_vocabulary_rejected() {
    let t = Test::new();
    t.turtle(
        "wb:providesCapability ex:Echo",
        "wb:requiresPermission ex:permission ; wb:providesCapability ex:Echo",
    );
    t.reject("RDF_INVALID");
    let t = Test::new();
    t.turtle(
        "wb:providesCapability ex:Echo",
        "wb:permissionName \"filesystem.read\" ; wb:providesCapability ex:Echo",
    );
    t.reject("CAPABILITY_DENIED");
}

#[test]
fn reg_004_referenced_dialect_and_custom_vocabulary_rejected() {
    let t = Test::new();
    fs::write(
        t.source.join("schemas/input.json"),
        r#"{"$ref":"output.json"}"#,
    )
    .unwrap();
    fs::write(
        t.source.join("schemas/output.json"),
        r#"{"$schema":"http://json-schema.org/draft-07/schema#","type":"object"}"#,
    )
    .unwrap();
    t.reject("SCHEMA_VIOLATION");
    let t = Test::new();
    fs::write(
        t.source.join("schemas/input.json"),
        r#"{"properties":{"text":{"$schema":"http://json-schema.org/draft-07/schema#"}}}"#,
    )
    .unwrap();
    t.reject("SCHEMA_VIOLATION");
    let t = Test::new();
    fs::write(
        t.source.join("schemas/input.json"),
        r#"{"$vocabulary":{"https://example.org/custom":true}}"#,
    )
    .unwrap();
    t.reject("SCHEMA_VIOLATION");
}

#[test]
fn reg_014_corrupt_marker_preserves_possible_pinned_object() {
    let t = Test::new();
    let receipt = t.registry.admit_directory(&t.source).unwrap();
    let releases = t.tmp.path().join("store/releases");
    let marker = fs::read_dir(releases)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(marker, b"{}").unwrap();
    let r = Registry::open(&t.tmp.path().join("store"), t.policy.clone()).unwrap();
    assert!(r.discover(None).unwrap().is_empty());
    assert_eq!(
        r.resolve_exact(&receipt.pin).unwrap_err().code,
        "INTEGRITY_FAILURE"
    );
    assert!(t
        .object(&receipt.record_digest)
        .join("payload/logic.wasm")
        .is_file());
}
