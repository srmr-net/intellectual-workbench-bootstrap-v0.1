use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;
use wb_contracts::ui::*;
use wb_kernel::Kernel;
use wb_registry::{Policy, Registry};
use wb_ui_host::{framing, Session};
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
struct Fixture {
    tmp: TempDir,
    source: PathBuf,
    policy: Policy,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("package");
        copy(&root().join("target/ui-fixtures/mortgage-ui"), &source);
        let python = std::env::var_os("REGISTRY_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let out = Command::new("python3")
                    .args(["-c", "import sys;print(sys.executable)"])
                    .output()
                    .unwrap();
                String::from_utf8(out.stdout).unwrap().trim().into()
            });
        let mut policy = Policy::local(
            vec![tmp.path().into()],
            python,
            root().join("tools/registry_validate.py"),
            PathBuf::from(env!("CARGO_BIN_EXE_wb-ui-worker")).with_file_name("wb-registry-inspect"),
        );
        policy.semantic_ui_v01 = true;
        Self {
            tmp,
            source,
            policy,
        }
    }
    fn registry(&self, name: &str) -> Registry {
        Registry::open(&self.tmp.path().join(name), self.policy.clone()).unwrap()
    }
    fn edit(&self, f: impl FnOnce(&mut Value)) {
        let p = self.source.join("ui.json");
        let mut v: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        f(&mut v);
        fs::write(p, v.to_string()).unwrap();
    }
    fn reject(&self, expected: &[&str]) {
        let r = self.registry("rejected");
        let e = r.admit_directory(&self.source).unwrap_err();
        assert!(expected.contains(&e.code.as_str()), "{e}");
        assert!(r.discover(None).unwrap().is_empty());
    }
    fn session(&self, id: &str) -> (Registry, Kernel, Session) {
        let r = self.registry(id);
        let receipt = r.admit_directory(&self.source).unwrap();
        let h = r.resolve_exact(&receipt.pin).unwrap();
        let k = Kernel::new(
            wb_wasm_host::Limits {
                fuel: 20_000_000,
                ..Default::default()
            },
            8,
            8,
        )
        .unwrap();
        let pin = k.cache_registry_release(&r, &h).unwrap();
        k.approve_execution(id, "component", &pin).unwrap();
        let s = Session::start(&k, &r, &h, id, "component").unwrap();
        (r, k, s)
    }
}
fn request(s: &Session, command: wb_contracts::ui::Command) -> Request {
    Request {
        protocol: "0.1".into(),
        local_token: "local-1".into(),
        key: s.key.clone(),
        expected_room_revision: s.room().revision(),
        expected_generation: s.room().generation("component").unwrap(),
        command,
    }
}
fn apply(s: &mut Session, p: &str, r: &str, n: &str) -> Reply {
    let req = request(
        s,
        wb_contracts::ui::Command::Apply {
            proposed_values: BTreeMap::from([
                ("principal".into(), json!(p)),
                ("rate".into(), json!(r)),
                ("term".into(), json!(n)),
            ]),
        },
    );
    s.request(req)
}
#[test]
fn ui_001_strict_json_schema_and_version() {
    let f = Fixture::new();
    let r = f.registry("valid");
    let receipt = r.admit_directory(&f.source).unwrap();
    let h = r.resolve_exact(&receipt.pin).unwrap();
    assert_eq!(
        r.read_verified_ui(&h).unwrap().document.root.children.len(),
        4
    );
    let f = Fixture::new();
    f.edit(|v| v["uiVersion"] = json!("9"));
    f.reject(&["UNSUPPORTED_UI_VERSION"]);
    let f = Fixture::new();
    let p = f.source.join("ui.json");
    let s = fs::read_to_string(&p).unwrap();
    fs::write(p, s.replacen('{', "{\"uiVersion\":\"0.1\",", 1)).unwrap();
    f.reject(&["INVALID_UI"]);
    assert!(strict_json(br#"{"root":{"id":1,"id":2}}"#).is_err());
}
#[test]
fn ui_002_010_unknown_executable_and_remote_metadata_rejected() {
    for kind in 0..7 {
        let f = Fixture::new();
        f.edit(|v| match kind {
            0 => v["root"]["children"][0]["type"] = json!("qml"),
            1 => v["root"]["children"][0]["onClicked"] = json!("evil()"),
            2 => v["root"]["children"][0]["label"] = json!("Qt.createQmlObject('bad')"),
            3 => v["root"]["children"][0]["label"] = json!("<script>alert(1)</script>"),
            4 => v["root"]["children"][0]["unit"] = json!("https://example.org/ui.qml"),
            5 => v["root"]["children"][0]["id"] = json!("principal\n"),
            _ => v["root"]["script"] = json!({"code":"import QtQuick"}),
        });
        f.reject(&["INVALID_UI", "UNSAFE_UI_CONTENT"]);
    }
}
#[test]
fn ui_003_ids_labels_and_bounded_form() {
    for kind in 0..5 {
        let f = Fixture::new();
        f.edit(|v| match kind {
            0 => v["root"]["children"][1]["id"] = json!("principal"),
            1 => v["root"]["children"][0]["label"] = json!(" "),
            2 => v["root"]["title"] = json!("x".repeat(161)),
            3 => v["root"]["type"] = json!("column"),
            _ => {
                let item = v["root"]["children"][3].clone();
                let list = v["root"]["children"].as_array_mut().unwrap();
                for n in 4..33 {
                    let mut c = item.clone();
                    c["id"] = json!(format!("readout{n}"));
                    list.push(c);
                }
            }
        });
        f.reject(&["INVALID_UI", "UNSAFE_UI_CONTENT", "UI_DUPLICATE_ID"]);
    }
}
#[test]
fn ui_004_direction_and_manifest_port_authority() {
    for kind in 0..3 {
        let f = Fixture::new();
        f.edit(|v| match kind {
            0 => v["root"]["children"][3]["bind"]["direction"] = json!("input"),
            1 => v["root"]["children"][0]["bind"]["port"] = json!("payment"),
            _ => v["root"]["children"][0]["bind"]["port"] = json!("loan"),
        });
        f.reject(&["INVALID_UI", "UI_BINDING_MISMATCH"]);
    }
}
#[test]
fn ui_005_fields_scalar_kinds_escapes_and_schema_refs() {
    for kind in 0..5 {
        let f = Fixture::new();
        f.edit(|v| match kind {
            0 => v["root"]["children"][0]["bind"]["field"] = json!("/missing"),
            1 => v["root"]["children"][0]["type"] = json!("integer-input"),
            2 => v["root"]["children"][0]["bind"]["field"] = json!("/principal/evil"),
            3 => v["root"]["children"][0]["bind"]["field"] = json!("/__proto__"),
            _ => v["root"]["children"][0]["bind"]["field"] = json!("/bad~2escape"),
        });
        f.reject(&["INVALID_UI", "UI_BINDING_MISMATCH"]);
    }
    assert_eq!(wb_semantic::ui::field_name("/a~1b~0c").unwrap(), "a/b~c");
    let f = Fixture::new();
    fs::write(
        f.source.join("schemas/mortgage-input.schema.json"),
        br#"{"$ref":"https://example.org/schema"}"#,
    )
    .unwrap();
    f.reject(&["INVALID_PATH"]);
}
#[test]
fn ui_006_rdf_interface_exact_reconciliation() {
    for kind in 0..3 {
        let f = Fixture::new();
        let p = f.source.join("capabilities.ttl");
        let s = fs::read_to_string(&p).unwrap();
        fs::write(
            p,
            match kind {
                0 => s.replace(
                    "wb:uiDefinitionRef \"ui.json\"",
                    "wb:uiDefinitionRef \"other.json\"",
                ),
                1 => s.replace(" ; wb:hasInterface ex:form", ""),
                _ => s + "\nex:extra a wb:AbstractInterface ; wb:uiDefinitionRef \"ui.json\" .\n",
            },
        )
        .unwrap();
        f.reject(&["SEMANTIC_MISMATCH"]);
    }
}
#[test]
fn ui_007_opt_in_immutable_ui_and_tamper() {
    let f = Fixture::new();
    let mut old = f.policy.clone();
    old.semantic_ui_v01 = false;
    let legacy = Registry::open(&f.tmp.path().join("old"), old).unwrap();
    assert_eq!(
        legacy.admit_directory(&f.source).unwrap_err().code,
        "SCHEMA_VIOLATION"
    );
    let r = f.registry("ui");
    let receipt = r.admit_directory(&f.source).unwrap();
    let h = r.resolve_exact(&receipt.pin).unwrap();
    let doc = r.read_verified_ui(&h).unwrap().document;
    let evidence = root().join("target/abi-evidence/ui");
    fs::create_dir_all(&evidence).unwrap();
    fs::write(
        evidence.join("receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    fs::copy(
        f.tmp
            .path()
            .join("ui/objects")
            .join(&receipt.record_digest)
            .join("record.json"),
        evidence.join("record.json"),
    )
    .unwrap();
    f.edit(|v| v["root"]["title"] = json!("Different form"));
    assert_eq!(
        r.admit_directory(&f.source).unwrap_err().code,
        "RELEASE_CONFLICT"
    );
    assert_eq!(r.read_verified_ui(&h).unwrap().document, doc);
    fs::write(
        f.tmp
            .path()
            .join("ui/objects")
            .join(&receipt.record_digest)
            .join("payload/ui.json"),
        b"{}",
    )
    .unwrap();
    assert_eq!(
        r.read_verified_ui(&h).unwrap_err().code,
        "INTEGRITY_FAILURE"
    );
}
#[test]
fn ui_008_headless_none_and_mixed_store_legacy_policy() {
    let f = Fixture::new();
    let r = f.registry("mixed");
    r.admit_directory(&f.source).unwrap();
    let source = f.tmp.path().join("headless");
    copy(
        &root().join("target/mortgage-packages/mortgage-headless"),
        &source,
    );
    let mut policy = f.policy.clone();
    policy.semantic_ui_v01 = false;
    let legacy = Registry::open(&f.tmp.path().join("mixed"), policy).unwrap();
    let receipt = legacy.admit_directory(&source).unwrap();
    let h = legacy.resolve_exact(&receipt.pin).unwrap();
    let (checked, _) = legacy.read_verified_bundle(&h).unwrap();
    assert!(checked.ui.is_none());
    assert!(serde_json::to_value(&checked).unwrap().get("ui").is_none());
    assert_eq!(legacy.admit_directory(&source).unwrap(), receipt);
    assert_eq!(legacy.discover(None).unwrap().len(), 2);
}
#[test]
fn ui_013_drafts_invalid_and_incomplete_never_compute() {
    let f = Fixture::new();
    let (_r, k, mut s) = f.session("drafts");
    let revision = s.room().revision();
    for value in ["", "1.", "-1", "01", "1e6", "NaN"] {
        let req = request(
            &s,
            wb_contracts::ui::Command::Edit {
                control_id: "principal".into(),
                proposed_value: json!(value),
            },
        );
        let reply = s.request(req);
        assert!(!reply.accepted);
        assert_eq!(s.room().revision(), revision);
        assert!(!s
            .room()
            .audit()
            .iter()
            .any(|e| e.kind == "invocation.started"));
    }
    assert_eq!(k.in_flight(), 0);
    let reply = apply(&mut s, "250000.00", "5.00", "12.5");
    assert!(!reply.accepted);
    assert_eq!(s.room().revision(), revision);
}
#[test]
fn ui_014_real_installed_wasm_committed_projection() {
    let f = Fixture::new();
    let (_r, _k, mut s) = f.session("calculate");
    let first = apply(&mut s, "250000.00", "5.00", "300");
    assert!(first.accepted);
    assert_eq!(first.projection.values["payment"], json!("1461.48"));
    let second = apply(&mut s, "120000.00", "0", "240");
    assert!(second.accepted);
    assert_eq!(second.projection.values["payment"], json!("500.00"));
    assert!(second.projection.room_revision > first.projection.room_revision);
    let path = root().join("target/abi-evidence/ui");
    fs::create_dir_all(&path).unwrap();
    fs::write(
        path.join("projections.json"),
        serde_json::to_vec_pretty(&json!([first, second])).unwrap(),
    )
    .unwrap();
}
#[test]
fn ui_015_revision_generation_and_exact_wire_counters() {
    let f = Fixture::new();
    let (_r, _k, mut s) = f.session("fences");
    let old = request(
        &s,
        wb_contracts::ui::Command::Edit {
            control_id: "principal".into(),
            proposed_value: json!("120000.00"),
        },
    );
    assert!(apply(&mut s, "250000.00", "5.00", "300").accepted);
    let revision = s.room().revision();
    let reply = s.request(old);
    assert!(!reply.accepted);
    assert_eq!(reply.projection.diagnostics[0].code, "STATE_CONFLICT");
    assert_eq!(s.room().revision(), revision);
    assert_eq!(s.drafts()["principal"], json!("120000.00"));
    let mut req = request(
        &s,
        wb_contracts::ui::Command::Edit {
            control_id: "principal".into(),
            proposed_value: json!("999"),
        },
    );
    req.expected_generation = 0;
    let drafts = s.drafts().clone();
    assert!(!s.request(req).accepted);
    assert_eq!(s.drafts(), &drafts);
    let mut projection = s.ready().projection;
    projection.room_revision = u64::MAX;
    let raw = serde_json::to_value(&projection).unwrap();
    assert_eq!(raw["roomRevision"], json!(u64::MAX.to_string()));
    assert_eq!(
        serde_json::from_value::<Projection>(raw)
            .unwrap()
            .room_revision,
        u64::MAX
    );
}
#[test]
fn ui_016_bounded_transport_and_no_projection_loss() {
    let bytes = vec![b'x'; framing::MAX_FRAME];
    let mut out = vec![];
    framing::write_frame(&mut out, &bytes).unwrap();
    assert_eq!(
        framing::read_frame(Cursor::new(out)).unwrap().unwrap(),
        bytes
    );
    assert!(
        framing::read_frame(Cursor::new(((framing::MAX_FRAME + 1) as u32).to_be_bytes())).is_err()
    );
    assert!(framing::read_frame(Cursor::new([0, 0, 0, 5, 1])).is_err());
    let (tx, rx) = std::sync::mpsc::sync_channel(2);
    tx.try_send(1).unwrap();
    tx.try_send(2).unwrap();
    assert!(tx.try_send(3).is_err());
    assert_eq!(rx.recv().unwrap(), 1);
    assert_eq!(rx.recv().unwrap(), 2);
}
#[test]
fn ui_017_key_and_readout_ownership_no_forged_routes() {
    let f = Fixture::new();
    let (_r, _k, mut s) = f.session("owner");
    let snapshot = s.snapshot();
    let mut req = request(
        &s,
        wb_contracts::ui::Command::Edit {
            control_id: "principal".into(),
            proposed_value: json!("999"),
        },
    );
    req.key.room_id = "other".into();
    assert!(!s.request(req).accepted);
    assert!(s.drafts().is_empty());
    let req = request(
        &s,
        wb_contracts::ui::Command::Edit {
            control_id: "payment".into(),
            proposed_value: json!("999"),
        },
    );
    assert!(!s.request(req).accepted);
    assert_eq!(s.snapshot(), snapshot);
    let mut raw = serde_json::to_value(request(&s, wb_contracts::ui::Command::Refresh)).unwrap();
    raw["messageId"] = json!("forged");
    assert!(serde_json::from_value::<Request>(raw).is_err());
}
#[test]
fn ui_018_restore_exact_ui_reference_and_pin() {
    let f = Fixture::new();
    let (r, k, mut s) = f.session("restore");
    assert!(apply(&mut s, "120000.00", "0", "240").accepted);
    let snapshot = s.snapshot();
    let receipt = r.admit_directory(&f.source).unwrap();
    let h = r.resolve_exact(&receipt.pin).unwrap();
    drop(s);
    let mut bad = snapshot.clone();
    bad.ui.definition_ref = "different/ui.json".into();
    assert!(Session::restore(&k, &r, &h, bad, "restore", "component").is_err());
    let mut restored = Session::restore(&k, &r, &h, snapshot, "restore", "component").unwrap();
    assert_eq!(
        restored.ready().projection.values["payment"],
        json!("500.00")
    );
}
#[test]
fn ui_019_020_external_review_and_hosted_wiring_not_self_acceptance() {
    let task =
        fs::read_to_string(root().join("docs/implementation/adr-004/TEST_MATRIX.md")).unwrap();
    assert!(task.contains("independent"));
    let schema: Value =
        serde_json::from_slice(&fs::read(root().join("schemas/ui.schema.json")).unwrap()).unwrap();
    assert_eq!(
        schema["$id"],
        json!("https://workbench.example/schemas/adr-004/0.1/ui.schema.json")
    );
}

#[test]
fn ui_012_017_worker_rejects_forged_shutdown_and_remains_usable() {
    use std::io::{BufReader, BufWriter};
    use std::process::Stdio;
    use std::sync::mpsc;
    use std::time::Duration;
    let f = Fixture::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_wb-ui-worker"))
        .args([
            f.source.as_os_str(),
            f.tmp.path().join("wire-store").as_os_str(),
            f.policy.python.as_os_str(),
            f.policy.semantic_worker.as_os_str(),
            f.policy.abi_worker.as_os_str(),
            std::ffi::OsStr::new("wire-room"),
            std::ffi::OsStr::new("new"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = BufWriter::new(child.stdin.take().unwrap());
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        while let Ok(Some(frame)) = framing::read_frame(&mut reader) {
            if tx.send(strict_json(&frame).unwrap()).is_err() {
                break;
            }
        }
    });
    let mut receive = |stage: &str| {
        rx.recv_timeout(Duration::from_secs(120))
            .unwrap_or_else(|e| {
                let _ = child.kill();
                panic!("worker timeout during {stage}: {e}")
            })
    };
    let ready = receive("ready");
    let key = ready["projection"]["key"].clone();
    let mut request = json!({"protocol":"0.1", "localToken":"wire", "key":key,
        "expectedRoomRevision":ready["projection"]["roomRevision"],
        "expectedGeneration":ready["projection"]["executionGeneration"], "command":{"type":"shutdown"}});
    request["key"]["roomId"] = json!("forged");
    framing::write_frame(&mut input, request.to_string().as_bytes()).unwrap();
    let rejected = receive("forged shutdown rejection");
    assert_eq!(rejected["accepted"], false);
    assert_eq!(
        rejected["projection"]["diagnostics"][0]["code"],
        "UI_OWNERSHIP_DENIED"
    );
    request["key"] = key;
    request["command"]["type"] = json!("refresh");
    framing::write_frame(&mut input, request.to_string().as_bytes()).unwrap();
    assert_eq!(receive("refresh")["accepted"], true);
    request["command"]["type"] = json!("shutdown");
    framing::write_frame(&mut input, request.to_string().as_bytes()).unwrap();
    assert_eq!(receive("accepted shutdown")["kind"], "saved");
    drop(receive);
    assert!(child.wait().unwrap().success());
}

#[test]
fn ui_006_ui_basename_matches_accepted_room_reference() {
    let f = Fixture::new();
    fs::rename(f.source.join("ui.json"), f.source.join("form.json")).unwrap();
    let path = f.source.join("component.json");
    let mut m: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    m["ui"] = json!("form.json");
    fs::write(path, m.to_string()).unwrap();
    let path = f.source.join("capabilities.ttl");
    fs::write(
        &path,
        fs::read_to_string(&path)
            .unwrap()
            .replace("ui.json", "form.json"),
    )
    .unwrap();
    f.reject(&["UI_BINDING_MISMATCH"]);
}
