//! VSLICE-001..020: real registry -> kernel -> native ingress -> WIT -> projection.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};
use tempfile::TempDir;
use wb_contracts::{ipc::*, ComponentRef};
use wb_kernel::{headless::HeadlessRoom, room::Room, Kernel};
use wb_registry::{AdmissionReceipt, Checkpoint, Policy, Registry};
use wb_wasm_host::Limits;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
fn limits() -> Limits {
    Limits {
        fuel: 20_000_000,
        ..Limits::default()
    }
}
fn input(principal: &str, rate: &str, months: u32) -> Value {
    json!({"principal":principal,"annual_rate_percent":rate,"term_months":months})
}
fn standard() -> Value {
    input("250000.00", "5.00", 300)
}
struct Fixture {
    tmp: TempDir,
    source: PathBuf,
    registry: Registry,
    receipt: AdmissionReceipt,
    policy: Policy,
}
impl Fixture {
    fn new(name: &str) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source");
        copy(&root().join("target/mortgage-packages").join(name), &source);
        let python = std::env::var_os("REGISTRY_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let r = Command::new("python3")
                    .args(["-c", "import sys; print(sys.executable)"])
                    .output()
                    .unwrap();
                PathBuf::from(String::from_utf8(r.stdout).unwrap().trim())
            });
        let abi = PathBuf::from(env!("CARGO_BIN_EXE_wb-mortgage-headless"))
            .with_file_name("wb-registry-inspect");
        let policy = Policy::local(
            vec![tmp.path().into()],
            python,
            root().join("tools/registry_validate.py"),
            abi,
        );
        let registry = Registry::open(&tmp.path().join("store"), policy.clone()).unwrap();
        let receipt = registry.admit_directory(&source).unwrap();
        Self {
            tmp,
            source,
            registry,
            receipt,
            policy,
        }
    }
    fn pin(&self) -> ComponentRef {
        ComponentRef {
            id: self.receipt.pin.id.clone(),
            version: self.receipt.pin.version.clone(),
            sha256: self.receipt.pin.sha256.clone(),
        }
    }
    fn kernel(&self, policy: Limits) -> Kernel {
        let k = Kernel::new(policy, 16, 16).unwrap();
        let h = self.registry.resolve_exact(&self.receipt.pin).unwrap();
        assert_eq!(
            k.cache_registry_release(&self.registry, &h).unwrap(),
            self.pin()
        );
        k
    }
    fn session(&self, k: &Kernel, id: &str, capacity: usize) -> HeadlessRoom {
        k.approve_execution(id, "mortgage", &self.pin()).unwrap();
        HeadlessRoom::start(
            k,
            id,
            "Mortgage slice",
            Endpoint::new("mortgage", "calculate"),
            self.pin(),
            capacity,
        )
        .unwrap()
    }
    fn alter(&self, f: impl FnOnce(&mut Value)) {
        let p = self.source.join("component.json");
        let mut m: Value = serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
        f(&mut m);
        fs::write(p, m.to_string()).unwrap();
    }
    fn replace_module(&self, bytes: &[u8]) {
        let p = self.source.join("component.json");
        let m: Value = serde_json::from_slice(&fs::read(p).unwrap()).unwrap();
        let old = m["runtime"]["sha256"].as_str().unwrap();
        let new = digest(bytes);
        self.alter(|m| m["runtime"]["sha256"] = json!(new));
        fs::write(self.source.join("logic.wasm"), bytes).unwrap();
        let p = self.source.join("capabilities.ttl");
        fs::write(&p, fs::read_to_string(&p).unwrap().replace(old, &new)).unwrap();
    }
    fn candidate_registry(&self, name: &str) -> Registry {
        Registry::open(&self.tmp.path().join(name), self.policy.clone()).unwrap()
    }
}
fn mortgage() -> Fixture {
    Fixture::new("mortgage-headless")
}
fn code<T>(value: Result<T>, expected: ErrorCode) {
    match value {
        Ok(_) => panic!("expected {expected:?}"),
        Err(e) => assert_eq!(e.code, expected, "{e:?}"),
    }
}
fn value(session: &HeadlessRoom) -> Value {
    let p = session.room().output("mortgage", "payment").unwrap();
    assert!(p.valid, "{p:?}");
    p.value.clone().unwrap()
}
fn calculate(session: &mut HeadlessRoom, name: &str, payload: Value) -> Value {
    session.submit(name, payload, name).unwrap();
    let ticket = session.room_mut().next_invocation().unwrap().unwrap();
    session.room_mut().complete(ticket.run()).unwrap();
    value(session)
}
fn host_ui() -> UiReference {
    UiReference {
        definition_ref: "host.headless/session/ui.json".into(),
    }
}
fn same_plan(room: &Room, pin: ComponentRef) -> CompositionPlan {
    CompositionPlan {
        plan_version: "0.1".into(),
        room_id: room.id().into(),
        expected_room_revision: room.revision(),
        instances: vec![InstanceSpec {
            instance_id: "mortgage".into(),
            component: pin,
        }],
        connections: vec![],
        ui: host_ui(),
        requested_permissions: vec![],
    }
}
#[test]
fn vslice_001_real_pure_component_and_unchanged_wit() {
    let f = mortgage();
    let h = f.registry.resolve_exact(&f.receipt.pin).unwrap();
    let bytes = f.registry.read_verified_artifact(&h).unwrap();
    assert!(bytes.starts_with(b"\0asm\x0d\0\x01\0"));
    assert_eq!(
        digest(&fs::read(root().join("wit/component.wit")).unwrap()),
        "cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0"
    );
    let lock = fs::read_to_string(root().join("examples/wasm-components/Cargo.lock")).unwrap();
    assert!(lock.contains("name = \"num-bigint\"\nversion = \"0.4.8\""));
    let (checked, _) = f.registry.read_verified_bundle(&h).unwrap();
    assert_eq!(checked.manifest["permissions"], json!([]));
    assert!(checked.manifest.get("ui").is_none());
}
#[test]
fn vslice_002_complete_package_actual_admission() {
    let f = mortgage();
    let discovered = f
        .registry
        .discover(Some(
            "https://workbench.example/ns/0.1#CalculateMortgagePayment",
        ))
        .unwrap();
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].trust, "unreviewed");
    let report = root().join("target/abi-evidence/vslice");
    fs::create_dir_all(&report).unwrap();
    fs::write(
        report.join("receipt.json"),
        serde_json::to_vec(&f.receipt).unwrap(),
    )
    .unwrap();
    fs::copy(
        f.tmp
            .path()
            .join("store/objects")
            .join(&f.receipt.record_digest)
            .join("record.json"),
        report.join("record.json"),
    )
    .unwrap();
    assert_eq!(f.registry.admit_directory(&f.source).unwrap(), f.receipt);
}
#[test]
fn vslice_003_exact_pin_and_boundary_integrity() {
    let f = mortgage();
    let mut bad = f.receipt.pin.clone();
    bad.sha256 = "0".repeat(64);
    assert_eq!(
        f.registry.resolve_exact(&bad).unwrap_err().code,
        "COMPONENT_NOT_FOUND"
    );
    let h = f.registry.resolve_exact(&f.receipt.pin).unwrap();
    let p = f
        .tmp
        .path()
        .join("store/objects")
        .join(&f.receipt.record_digest)
        .join("payload/logic.wasm");
    fs::write(p, b"tamper").unwrap();
    let k = Kernel::new(limits(), 4, 4).unwrap();
    code(
        k.cache_registry_release(&f.registry, &h),
        ErrorCode::ComponentNotFound,
    );
    assert!(f.registry.discover(None).unwrap().is_empty());
}
#[test]
fn vslice_004_real_lifecycle_and_explicit_policy() {
    let f = mortgage();
    let k = f.kernel(limits());
    code(
        HeadlessRoom::start(
            &k,
            "life",
            "Mortgage",
            Endpoint::new("mortgage", "calculate"),
            f.pin(),
            8,
        ),
        ErrorCode::CapabilityDenied,
    );
    assert_eq!(k.reserved_instances(), 0);
    let mut s = f.session(&k, "life", 8);
    assert_eq!(s.room().state("mortgage"), Some(InstanceStatus::Running));
    code(
        s.room_mut().start("mortgage", "headless.host", "again"),
        ErrorCode::InvalidTransition,
    );
    assert!(!s
        .room()
        .audit()
        .iter()
        .any(|e| e.kind == "invocation.started"));
    s.room_mut()
        .stop("mortgage", "headless.host", "stop")
        .unwrap();
    code(
        s.submit("stopped", standard(), "stopped"),
        ErrorCode::InvalidTransition,
    );
    s.room_mut()
        .prepare("mortgage", "headless.host", "prepare")
        .unwrap();
    s.room_mut()
        .start("mortgage", "headless.host", "start")
        .unwrap();
    assert_eq!(
        calculate(&mut s, "good", standard()),
        json!({"monthly_payment":"1461.48"})
    );
}
#[test]
fn vslice_005_actual_full_path_and_correlated_projection() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut s = f.session(&k, "full", 8);
    let envelope = s
        .submit("full-request", standard(), "full-correlation")
        .unwrap();
    assert!(envelope.source.instance_id.starts_with("@ui:"));
    assert_eq!(envelope.destination, Endpoint::new("mortgage", "calculate"));
    let ticket = s.room_mut().next_invocation().unwrap().unwrap();
    let context = ticket.context().clone();
    assert!(!context.invocation_id.is_empty());
    assert!(s.room_mut().complete(ticket.run()).unwrap().is_empty()); // no fabricated output envelope in a single-node graph
    assert_eq!(value(&s), json!({"monthly_payment":"1461.48"}));
    assert_eq!(
        s.room().output("mortgage", "payment").unwrap().revision,
        context.room_revision
    );
    assert!(s
        .room()
        .audit()
        .iter()
        .any(|e| e.kind == "invocation.completed" && e.correlation_id == "full-correlation"));
    let trace = json!({"pin":f.receipt.pin,"inputEnvelope":envelope,"invocationId":context.invocation_id,"revision":context.room_revision,"generation":context.execution_generation,"committedOutput":value(&s),"audit":s.room().audit()});
    let p = root().join("target/abi-evidence/vslice");
    fs::create_dir_all(&p).unwrap();
    fs::write(
        p.join("trace.json"),
        serde_json::to_vec_pretty(&trace).unwrap(),
    )
    .unwrap();
}
#[test]
fn vslice_006_all_vectors_independent_oracle_and_boundaries() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut s = f.session(&k, "vectors", 8);
    let oracle: Value =
        serde_json::from_slice(&fs::read(root().join("target/mortgage-reference.json")).unwrap())
            .unwrap();
    assert_eq!(oracle["cases"].as_array().unwrap().len(), 33);
    let mut results = Vec::new();
    for case in oracle["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let actual = calculate(&mut s, id, case["input"].clone());
        assert_eq!(actual, case["expected"], "{id}");
        let elapsed = s
            .room()
            .audit()
            .iter()
            .rev()
            .find(|e| e.kind == "invocation.completed")
            .unwrap()
            .elapsed_micros;
        results.push(json!({"id":id,"input":case["input"],"expected":case["expected"],"actual":actual,"elapsedMicros":elapsed}));
    }
    let p = root().join("target/abi-evidence/vslice");
    fs::create_dir_all(&p).unwrap();
    fs::write(
        p.join("numerical-results.json"),
        serde_json::to_vec_pretty(&results).unwrap(),
    )
    .unwrap();
    assert_eq!(
        calculate(
            &mut s,
            "integral-json-number",
            json!({"principal":"1.00","annual_rate_percent":"0","term_months":8.0})
        ),
        json!({"monthly_payment":"0.13"})
    );
}
#[test]
fn vslice_007_schema_and_guest_domain_rejection() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut s = f.session(&k, "negative", 8);
    let previous = calculate(&mut s, "first", standard());
    let vectors: Value =
        serde_json::from_slice(&fs::read(root().join("fixtures/mortgage-vectors.json")).unwrap())
            .unwrap();
    let mut bad: Vec<_> = vectors["negative"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["input"].clone())
        .collect();
    for text in [
        "01.00",
        "+1",
        "1.",
        ".1",
        "NaN",
        "Infinity",
        "1000000000.01",
        "0.001",
        " 1",
        "１",
        "1\n",
    ] {
        bad.push(input(text, "5", 12));
    }
    for rate in ["100.000001", "0.0000001", "-0", "01", "1e2", "1\n"] {
        bad.push(input("100", rate, 12));
    }
    bad.extend([json!([]), json!({}), json!(null), input("100", "5", 601)]);
    let snapshot = s.room().snapshot();
    let projection = s.room().output("mortgage", "payment").unwrap().clone();
    for (n, payload) in bad.iter().enumerate() {
        code(
            s.submit(&format!("bad-{n}"), payload.clone(), "negative"),
            ErrorCode::SchemaViolation,
        );
        assert_eq!(s.room().snapshot(), snapshot);
        assert_eq!(s.room().output("mortgage", "payment").unwrap(), &projection);
        assert_eq!(value(&s), previous);
    }
    // Secondary guest validation is exercised through actual registry + WIT using
    // a deliberately permissive declared schema in a separate immutable package.
    let relaxed = mortgage();
    relaxed.alter(|m| {
        m["id"] = json!("org.workbench.example.mortgage-relaxed");
    });
    let p = relaxed.source.join("capabilities.ttl");
    fs::write(
        &p,
        fs::read_to_string(&p).unwrap().replace(
            "org.workbench.example.mortgage-headless",
            "org.workbench.example.mortgage-relaxed",
        ),
    )
    .unwrap();
    fs::write(
        relaxed.source.join("schemas/mortgage-input.schema.json"),
        b"true",
    )
    .unwrap();
    let r = relaxed.candidate_registry("relaxed");
    let receipt = r.admit_directory(&relaxed.source).unwrap();
    let h = r.resolve_exact(&receipt.pin).unwrap();
    let k = Kernel::new(limits(), 4, 4).unwrap();
    let pin = k.cache_registry_release(&r, &h).unwrap();
    k.approve_execution("domain", "mortgage", &pin).unwrap();
    let mut s = HeadlessRoom::start(
        &k,
        "domain",
        "Domain",
        Endpoint::new("mortgage", "calculate"),
        pin,
        8,
    )
    .unwrap();
    for (n, payload) in bad.into_iter().enumerate() {
        s.submit(&format!("domain-{n}"), payload, "domain").unwrap();
        let i = s.room_mut().next_invocation().unwrap().unwrap();
        let e = s.room_mut().complete(i.run()).unwrap_err();
        assert_eq!(e.code, ErrorCode::GuestError);
        assert_eq!(e.detail["guestCode"], json!("INVALID_MORTGAGE_INPUT"));
        assert!(s
            .room()
            .output("mortgage", "payment")
            .unwrap()
            .value
            .is_none());
    }
}
#[test]
fn vslice_008_adversarial_output_atomic_rollback() {
    let f = Fixture::new("mortgage-adversary");
    let k = f.kernel(limits());
    let mut s = f.session(&k, "output-errors", 8);
    let previous = calculate(&mut s, "good", input("1.00", "5", 12));
    for (n, mode) in ["2.00", "3.00", "4.00", "7.00", "8.00"]
        .into_iter()
        .enumerate()
    {
        s.submit(&format!("bad-{n}"), input(mode, "5", 12), "bad-output")
            .unwrap();
        let i = s.room_mut().next_invocation().unwrap().unwrap();
        code(s.room_mut().complete(i.run()), ErrorCode::SchemaViolation);
        let p = s.room().output("mortgage", "payment").unwrap();
        assert_eq!(p.value.as_ref(), Some(&previous));
        assert!(!p.valid);
        assert_eq!(p.error, Some(ErrorCode::SchemaViolation));
        assert!(s.room().pending().is_empty());
        assert_eq!(s.room().state("mortgage"), Some(InstanceStatus::Running));
    }
    assert_eq!(
        calculate(&mut s, "recovery", input("1.00", "5", 12)),
        previous
    );
}
#[test]
fn vslice_009_tamper_never_registers_candidate() {
    for mutation in 0..5 {
        let f = mortgage();
        match mutation {
            0 => {
                fs::write(f.source.join("logic.wasm"), b"tampered").unwrap();
            }
            1 => f.alter(|m| {
                m["ports"]["inputs"][0]["schema"] = json!("schemas/mortgage-output.schema.json")
            }),
            2 => {
                let p = f.source.join("capabilities.ttl");
                fs::write(
                    &p,
                    fs::read_to_string(&p)
                        .unwrap()
                        .replace("wb:MortgageTerms", "wb:MortgagePayment"),
                )
                .unwrap();
            }
            3 => f.replace_module(&wat::parse_str("(component)").unwrap()),
            _ => {
                let p = f.source.join("capabilities.ttl");
                fs::write(
                    &p,
                    fs::read_to_string(&p).unwrap().replace(
                        "wb:schemaRef \"schemas/mortgage-input.schema.json\"",
                        "wb:schemaRef \"schemas/mortgage-output.schema.json\"",
                    ),
                )
                .unwrap();
            }
        }
        let registry = f.candidate_registry("rejected");
        let expected = [
            "DIGEST_MISMATCH",
            "INVALID_PATH", // changed manifest leaves input schema unreferenced
            "SEMANTIC_MISMATCH",
            "ABI_INCOMPATIBLE",
            "SEMANTIC_MISMATCH",
        ][mutation];
        assert_eq!(
            registry.admit_directory(&f.source).unwrap_err().code,
            expected
        );
        assert!(registry.discover(None).unwrap().is_empty());
        assert!(registry.resolve_exact(&f.receipt.pin).is_err());
    }
}

fn trapping_initializer(bytes: &[u8]) -> Vec<u8> {
    use wasm_encoder::{
        Alias, Component, ComponentAliasSection, ComponentExportKind, ComponentExportSection,
        ComponentInstanceSection, InstanceSection, RawSection,
    };
    let mut wrapper = Component::new();
    wrapper.section(&RawSection { id: 4, data: bytes });
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
    wrapper.finish()
}
#[test]
fn vslice_010_admission_discovery_resolution_never_execute_guest() {
    let f = mortgage();
    let bytes = fs::read(f.source.join("logic.wasm")).unwrap();
    f.replace_module(&trapping_initializer(&bytes));
    let r = f.candidate_registry("trapping-init");
    let receipt = r.admit_directory(&f.source).unwrap();
    assert_eq!(r.discover(None).unwrap().len(), 1);
    let h = r.resolve_exact(&receipt.pin).unwrap();
    let k = Kernel::new(limits(), 4, 4).unwrap();
    let pin = k.cache_registry_release(&r, &h).unwrap();
    k.approve_execution("no-init", "mortgage", &pin).unwrap();
    let mut s = HeadlessRoom::start(
        &k,
        "no-init",
        "Nonexecution",
        Endpoint::new("mortgage", "calculate"),
        pin,
        8,
    )
    .unwrap();
    assert!(!s
        .room()
        .audit()
        .iter()
        .any(|e| e.kind == "invocation.started"));
    s.submit("execute", standard(), "execute").unwrap();
    let i = s.room_mut().next_invocation().unwrap().unwrap();
    code(s.room_mut().complete(i.run()), ErrorCode::ComponentTrap);
    assert!(s
        .room()
        .output("mortgage", "payment")
        .unwrap()
        .value
        .is_none());
}
#[test]
fn vslice_011_empty_grants_and_forbidden_imports_permissions() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut s = f.session(&k, "pure", 8);
    assert_eq!(
        calculate(&mut s, "pure", standard()),
        json!({"monthly_payment":"1461.48"})
    );
    let f = mortgage();
    f.replace_module(
        &fs::read(root().join("target/abi-guests/wasm32-wasip1/release/abi_denied_wasi.wasm"))
            .unwrap(),
    );
    let r = f.candidate_registry("imports");
    assert_eq!(
        r.admit_directory(&f.source).unwrap_err().code,
        "ABI_INCOMPATIBLE"
    );
    assert!(r.discover(None).unwrap().is_empty());
    let f = mortgage();
    f.alter(|m| m["permissions"] = json!([{"name":"filesystem.read","scope":{}}]));
    let r = f.candidate_registry("permission");
    assert_eq!(
        r.admit_directory(&f.source).unwrap_err().code,
        "CAPABILITY_DENIED"
    );
    assert!(r.discover(None).unwrap().is_empty());
}
fn observer(f: &Fixture, k: &Kernel, session: &mut HeadlessRoom) -> ComponentRef {
    let source = f.tmp.path().join("observer-source");
    copy(
        &root().join("target/mortgage-packages/mortgage-observer"),
        &source,
    );
    let receipt = f.registry.admit_directory(&source).unwrap();
    let h = f.registry.resolve_exact(&receipt.pin).unwrap();
    let pin = k.cache_registry_release(&f.registry, &h).unwrap();
    k.approve_execution(session.room().id(), "observer", &pin)
        .unwrap();
    let mut plan = same_plan(session.room(), f.pin());
    plan.instances.push(InstanceSpec {
        instance_id: "observer".into(),
        component: pin.clone(),
    });
    plan.connections.push(Connection {
        from: Endpoint::new("mortgage", "payment"),
        to: Endpoint::new("observer", "payment"),
    });
    session
        .room_mut()
        .apply_plan(plan, "headless.host", "connect-observer")
        .unwrap();
    session
        .room_mut()
        .prepare("observer", "headless.host", "observer-prepare")
        .unwrap();
    session
        .room_mut()
        .start("observer", "headless.host", "observer-start")
        .unwrap();
    pin
}
#[test]
fn vslice_012_real_bus_output_correlation_and_fifo_waves() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut s = f.session(&k, "ordering", 8);
    observer(&f, &k, &mut s);
    let first = s.submit("first", standard(), "first-correlation").unwrap();
    let second = s
        .submit("second", input("120000", "0", 240), "second-correlation")
        .unwrap();
    assert!(first.sequence < second.sequence);
    for (correlation, expected) in [
        ("first-correlation", "1461.48"),
        ("second-correlation", "500.00"),
    ] {
        let i = s.room_mut().next_invocation().unwrap().unwrap();
        let ctx = i.context().clone();
        let events = s.room_mut().complete(i.run()).unwrap();
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.source, Endpoint::new("mortgage", "payment"));
        assert_eq!(event.destination, Endpoint::new("observer", "payment"));
        assert_eq!(event.correlation_id, correlation);
        assert_eq!(event.room_id, "ordering");
        assert_eq!(event.payload, json!({"monthly_payment":expected}));
        assert!(event.sequence > second.sequence);
        assert_eq!(
            s.room().output("mortgage", "payment").unwrap().revision,
            ctx.room_revision
        );
        let downstream = s.room_mut().next_invocation().unwrap().unwrap();
        assert!(s.room_mut().complete(downstream.run()).unwrap().is_empty());
        let observed = s.room().output("observer", "observed").unwrap();
        assert!(observed.valid);
        assert_eq!(observed.value, Some(json!({"monthly_payment":expected})));
    }
    assert!(s.room().pending().is_empty());
    assert!(s
        .room()
        .audit()
        .windows(2)
        .all(|p| p[0].sequence < p[1].sequence));
}
#[test]
fn vslice_013_generation_and_revision_fence_real_completions() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut s = f.session(&k, "stale", 8);
    let previous = calculate(&mut s, "baseline", standard());
    s.submit("old", input("100000", "1", 120), "old").unwrap();
    let old = s.room_mut().next_invocation().unwrap().unwrap().run();
    s.room_mut()
        .stop("mortgage", "headless.host", "cancel")
        .unwrap();
    s.room_mut()
        .prepare("mortgage", "headless.host", "prepare")
        .unwrap();
    s.room_mut()
        .start("mortgage", "headless.host", "start")
        .unwrap();
    s.submit("new", input("120000", "0", 240), "new").unwrap();
    let new = s.room_mut().next_invocation().unwrap().unwrap();
    code(s.room_mut().complete(old), ErrorCode::StaleResult);
    assert_eq!(
        s.room()
            .output("mortgage", "payment")
            .unwrap()
            .value
            .as_ref(),
        Some(&previous)
    );
    s.room_mut().complete(new.run()).unwrap();
    let latest = value(&s);
    assert_eq!(latest, json!({"monthly_payment":"500.00"}));
    s.submit("old-revision", standard(), "old-revision")
        .unwrap();
    let old = s.room_mut().next_invocation().unwrap().unwrap().run();
    let plan = same_plan(s.room(), f.pin());
    s.room_mut()
        .apply_plan(plan, "headless.host", "revision")
        .unwrap();
    code(s.room_mut().complete(old), ErrorCode::StaleResult);
    assert_eq!(
        s.room()
            .output("mortgage", "payment")
            .unwrap()
            .value
            .as_ref(),
        Some(&latest)
    );
    assert_eq!(calculate(&mut s, "recovery", standard()), previous);
}
#[test]
fn vslice_014_bounded_ingress_no_silent_drop() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut s = f.session(&k, "queue", 1);
    let snapshot = s.room().snapshot();
    s.submit("accepted", standard(), "accepted").unwrap();
    code(
        s.submit("full", input("120000", "0", 240), "full"),
        ErrorCode::QueueFull,
    );
    assert_eq!(s.room().snapshot(), snapshot);
    assert_eq!(s.room().pending().len(), 1);
    assert!(s.room_mut().run_until_idle().is_empty());
    assert_eq!(value(&s), json!({"monthly_payment":"1461.48"}));
    assert_eq!(
        calculate(&mut s, "retry-full", input("120000", "0", 240)),
        json!({"monthly_payment":"500.00"})
    );
    assert!(s
        .room()
        .audit()
        .iter()
        .any(|e| e.kind == "backpressure.rejected"));
}
#[test]
fn vslice_015_interleaved_and_concurrent_room_isolation() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut a = f.session(&k, "room-a", 8);
    let mut b = f.session(&k, "room-b", 8);
    let ea = a.submit("same-id", standard(), "a").unwrap();
    let eb = b.submit("same-id", input("120000", "0", 240), "b").unwrap();
    assert_ne!(ea.message_id, eb.message_id);
    assert_eq!(ea.room_id, "room-a");
    assert_eq!(eb.room_id, "room-b");
    let ia = a.room_mut().next_invocation().unwrap().unwrap();
    let ib = b.room_mut().next_invocation().unwrap().unwrap();
    let ta = thread::spawn(move || ia.run());
    let tb = thread::spawn(move || ib.run());
    b.room_mut().complete(tb.join().unwrap()).unwrap();
    a.room_mut().complete(ta.join().unwrap()).unwrap();
    assert_eq!(value(&a), json!({"monthly_payment":"1461.48"}));
    assert_eq!(value(&b), json!({"monthly_payment":"500.00"}));
    assert!(a.room().audit().iter().all(|e| e.room_id == "room-a"));
    assert!(b.room().audit().iter().all(|e| e.room_id == "room-b"));
    assert_eq!(k.in_flight(), 0);
}
fn cache_snapshot(k: &Kernel, registry: &Registry, snapshot: &RoomSnapshot, approve: bool) {
    for c in &snapshot.components {
        let pin = wb_registry::ReleasePin {
            id: c.id.clone(),
            version: c.version.clone(),
            sha256: c.sha256.clone(),
        };
        let h = registry.resolve_exact(&pin).unwrap();
        let p = k.cache_registry_release(registry, &h).unwrap();
        if approve {
            k.approve_execution(&snapshot.room_id, &c.instance_id, &p)
                .unwrap();
        }
    }
}
#[test]
fn vslice_016_restore_exact_registry_pins_inputs_and_topology() {
    let f = mortgage();
    let k = f.kernel(limits());
    let mut s = f.session(&k, "restore", 8);
    observer(&f, &k, &mut s);
    s.submit("snapshot", standard(), "snapshot").unwrap();
    assert!(s.room_mut().run_until_idle().is_empty());
    let expected = value(&s);
    let snapshot = s.room().snapshot();
    assert!(!snapshot.state.contains_key("mortgage.payment"));
    assert_eq!(snapshot.connections.len(), 1);
    drop(s);
    drop(k);
    let k = Kernel::new(limits(), 16, 16).unwrap();
    cache_snapshot(&k, &f.registry, &snapshot, false);
    code(
        k.restore_room(snapshot.clone(), 8),
        ErrorCode::CapabilityDenied,
    );
    cache_snapshot(&k, &f.registry, &snapshot, true);
    let mut room = k.restore_room(snapshot.clone(), 8).unwrap();
    assert!(room.run_until_idle().is_empty());
    assert_eq!(
        room.output("mortgage", "payment").unwrap().value.as_ref(),
        Some(&expected)
    );
    assert!(room.output("mortgage", "payment").unwrap().valid);
    assert_eq!(room.connections(), snapshot.connections);
    assert_eq!(room.snapshot().components, snapshot.components);
    assert_eq!(
        room.output("observer", "observed").unwrap().value.as_ref(),
        Some(&expected)
    );
    drop(room);
    let mut bad = snapshot.clone();
    bad.components[0].sha256 = "0".repeat(64);
    code(k.restore_room(bad, 8), ErrorCode::ComponentNotFound);
    let handle = f.registry.resolve_exact(&f.receipt.pin).unwrap();
    fs::write(
        f.tmp
            .path()
            .join("store/objects")
            .join(&f.receipt.record_digest)
            .join("payload/logic.wasm"),
        b"tampered",
    )
    .unwrap();
    let fresh = Kernel::new(limits(), 4, 4).unwrap();
    code(
        fresh.cache_registry_release(&f.registry, &handle),
        ErrorCode::ComponentNotFound,
    );
    code(
        fresh.restore_room(snapshot, 8),
        ErrorCode::ComponentNotFound,
    );
}
#[test]
fn vslice_017_rebuild_evidence_and_repeated_invocation_stability() {
    let data: Value = serde_json::from_slice(
        &fs::read(root().join("target/mortgage-rebuild.json"))
            .expect("run examples/mortgage-headless/rebuild.py"),
    )
    .unwrap();
    assert_eq!(data["firstSha256"], data["cleanSha256"]);
    assert_eq!(data["target"], json!("wasm32-unknown-unknown"));
    let f = mortgage();
    assert_eq!(data["firstSha256"], json!(f.receipt.pin.sha256));
    let k = f.kernel(limits());
    let mut s = f.session(&k, "deterministic", 8);
    let a = calculate(&mut s, "one", standard());
    let b = calculate(&mut s, "two", standard());
    assert_eq!(a.to_string(), b.to_string());
}
#[test]
fn vslice_018_trap_budget_and_interrupted_install_recover() {
    let f = Fixture::new("mortgage-adversary");
    let k = f.kernel(limits());
    let mut s = f.session(&k, "failure", 8);
    let expected = calculate(&mut s, "good", input("1.00", "5", 12));
    for (request, mode, expected_error) in [
        ("trap", "5.00", ErrorCode::ComponentTrap),
        ("fuel", "6.00", ErrorCode::ResourceLimit),
    ] {
        s.submit(request, input(mode, "5", 12), request).unwrap();
        let ticket = s.room_mut().next_invocation().unwrap().unwrap();
        code(s.room_mut().complete(ticket.run()), expected_error);
        assert_eq!(s.room().state("mortgage"), Some(InstanceStatus::Failed));
        let projection = s.room().output("mortgage", "payment").unwrap();
        assert_eq!(projection.value.as_ref(), Some(&expected));
        assert!(!projection.valid);
        assert_eq!(projection.error, Some(expected_error));
        assert!(s.room().pending().is_empty());
        s.room_mut()
            .prepare("mortgage", "headless.host", "retry-prepare")
            .unwrap();
        s.room_mut()
            .start("mortgage", "headless.host", "retry-start")
            .unwrap();
        assert_eq!(
            calculate(
                &mut s,
                &format!("recover-{request}"),
                input("1.00", "5", 12)
            ),
            expected
        );
    }
    let f = mortgage();
    let store = f.tmp.path().join("crashed");
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "vslice_018_crash_child"])
        .env("VSLICE_CRASH_SOURCE", &f.source)
        .env("VSLICE_CRASH_STORE", &store)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(status.signal(), Some(libc::SIGKILL));
    let recovered = Registry::open(&store, f.policy.clone()).unwrap();
    assert!(recovered.discover(None).unwrap().is_empty());
    let receipt = recovered.admit_directory(&f.source).unwrap();
    assert!(recovered.resolve_exact(&receipt.pin).is_ok());
}
#[test]
fn vslice_018_crash_child() {
    let Some(source) = std::env::var_os("VSLICE_CRASH_SOURCE") else {
        return;
    };
    let fixture = mortgage();
    let mut policy = fixture.policy.clone();
    policy.allowed_roots = vec![Path::new(&source).parent().unwrap().into()];
    let store = std::env::var_os("VSLICE_CRASH_STORE").unwrap();
    let r = Registry::open(Path::new(&store), policy).unwrap();
    r.admit_observed(Path::new(&source), |checkpoint| {
        if checkpoint == Checkpoint::ObjectRenamed {
            unsafe {
                libc::kill(libc::getpid(), libc::SIGKILL);
            }
        }
    })
    .unwrap();
    panic!("crash checkpoint not reached");
}
#[test]
fn vslice_019_independent_review_protocol_is_enforced() {
    let spec = fs::read_to_string(
        root().join("docs/implementation/vslice-001/review/INDEPENDENT_REVIEW.md"),
    )
    .unwrap();
    assert!(spec.contains("Review A"));
    assert!(spec.contains("Review B"));
    let workflow = fs::read_to_string(root().join(".github/workflows/ci.yml")).unwrap();
    assert!(workflow.contains("examples/mortgage-headless/validate.sh"));
    // This checks required evidence wiring, not a fabricated independent approval.
    // External review sign-off and actual hosted success remain acceptance gates.
}
#[test]
fn vslice_020_all_accepted_regression_sources_unchanged() {
    for (path, sha) in [
        (
            "wit/component.wit",
            "cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0",
        ),
        (
            "schemas/component.schema.json",
            "15f2b7a2b84655e62f14fc521fe06a291fd00bf5fbcc120e509aaa84e9e1dd55",
        ),
        (
            "ontology/workbench.ttl",
            "1741a428b7ca8bd9d4a761659c30bc5f914fcac60d201ad5361cc54ca9cc3bda",
        ),
        (
            "crates/wb-wasm-host/tests/abi.rs",
            "263f74a47a8daf26197f8b3e947fc51e73e05da4030ac008ea416ed4d9542c95",
        ),
        (
            "crates/wb-testkit/tests/ipc.rs",
            "52a67d69f36f85303816ba12255f3e9c9e06a62f20e1523ee29844e778de8238",
        ),
        (
            "crates/wb-registry/tests/registry.rs",
            "1adcc72c4b320f2c824902f936244c7082085dd862ceac3882c75a7ce0490298",
        ),
        (
            "crates/wb-wasm-host/src/lib.rs",
            "a04bd4f87020d4d06e644dfc324d22f6752f57557273eae4aa543bb6344a2da2",
        ),
    ] {
        assert_eq!(digest(&fs::read(root().join(path)).unwrap()), sha, "{path}");
    }
    let script = fs::read_to_string(root().join("examples/mortgage-headless/validate.sh")).unwrap();
    assert!(script.contains("validate-registry.sh"));
}

#[test]
fn vslice_006_budget_calibration_retains_accepted_defaults() {
    assert_eq!(Limits::default().fuel, 2_000_000);
    assert_eq!(limits().fuel, 20_000_000);
    let f = mortgage();
    let k = f.kernel(Limits::default());
    let mut s = f.session(&k, "calibration", 8);
    s.submit("probe", input("1000000000", "0.000001", 600), "probe")
        .unwrap();
    let i = s.room_mut().next_invocation().unwrap().unwrap();
    let result = match s.room_mut().complete(i.run()) {
        Ok(_) => {
            assert_eq!(value(&s), json!({"monthly_payment":"1666667.08"}));
            json!({"outcome":"success"})
        }
        Err(e) => {
            assert_eq!(e.code, ErrorCode::ResourceLimit);
            json!({"outcome":e.code,"budget":e.detail["budget"]})
        }
    };
    let report = json!({"defaultFuel":Limits::default().fuel,"productFuel":limits().fuel,"memoryBytes":limits().memory_bytes,"epochTicks":limits().epoch_ticks,"defaultWorstCaseProbe":result});
    let p = root().join("target/abi-evidence/vslice");
    fs::create_dir_all(&p).unwrap();
    fs::write(
        p.join("budget-calibration.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
#[test]
fn vslice_005_real_cli_checks_committed_room_projection() {
    let f = mortgage();
    let store = f.tmp.path().join("cli-store");
    let output = Command::new(env!("CARGO_BIN_EXE_wb-mortgage-headless"))
        .arg(&f.source)
        .arg(store)
        .arg(standard().to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["output"], json!({"monthly_payment":"1461.48"}));
    assert_eq!(report["valid"], json!(true));
    assert_eq!(report["pin"]["sha256"], json!(f.receipt.pin.sha256));
    assert!(report["audit"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["kind"] == "invocation.completed"));
}
