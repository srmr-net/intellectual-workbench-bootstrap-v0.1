use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;
use wb_contracts::ui::{Command as UiCommand, Request as UiRequest};
use wb_desktop_host::{Command, Desktop, Request};
use wb_registry::{Policy, Registry};
use wb_storage::vs2::FileStore;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn copy(source: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(source).unwrap() {
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
    registry: Registry,
    source: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("package");
        copy(&root().join("target/ui-fixtures/mortgage-ui"), &source);
        let python = std::env::var_os("REGISTRY_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| root().join("target/vs2-venv/bin/python"));
        let mut p = Policy::local(
            vec![tmp.path().into()],
            python,
            root().join("tools/registry_validate.py"),
            root().join("target/debug/wb-registry-inspect"),
        );
        p.semantic_ui_v01 = true;
        let registry = Registry::open(&tmp.path().join("registry"), p).unwrap();
        registry.admit_directory(&source).unwrap();
        Self {
            tmp,
            registry,
            source,
        }
    }
    fn desktop(&self) -> Desktop {
        Desktop::open(
            self.registry.clone(),
            FileStore::open(&self.tmp.path().join("workspace")).unwrap(),
        )
    }
    fn snapshot(&self) -> Vec<u8> {
        fs::read(self.tmp.path().join("workspace/workspace.json")).unwrap()
    }
    fn adversary(&self) {
        use sha2::{Digest, Sha256};
        let bytes = fs::read(root().join("target/mortgage-packages/mortgage-adversary/logic.wasm"))
            .unwrap();
        let mut m: Value =
            serde_json::from_slice(&fs::read(self.source.join("component.json")).unwrap()).unwrap();
        let old = m["runtime"]["sha256"].as_str().unwrap().to_owned();
        m["id"] = json!("org.workbench.example.vs2-adversary-ui");
        let digest = format!("{:x}", Sha256::digest(&bytes));
        m["runtime"]["sha256"] = json!(digest);
        fs::write(self.source.join("component.json"), m.to_string()).unwrap();
        fs::write(self.source.join("logic.wasm"), bytes).unwrap();
        let ttl = fs::read_to_string(self.source.join("capabilities.ttl"))
            .unwrap()
            .replace(
                "org.workbench.example.mortgage-ui",
                "org.workbench.example.vs2-adversary-ui",
            )
            .replace(&old, &digest);
        fs::write(self.source.join("capabilities.ttl"), ttl).unwrap();
        self.registry.admit_directory(&self.source).unwrap();
    }
}
fn state(d: &mut Desktop) -> Value {
    d.state("inspection", true, vec![])
}
fn command(d: &mut Desktop, c: Command) -> Value {
    let s = state(d);
    d.request(Request {
        protocol: "desktop.1".into(),
        token: "test".into(),
        epoch: s["epoch"].as_str().unwrap().into(),
        command: c,
    })
}
fn create(d: &mut Desktop, id: &str) -> Value {
    let s = state(d);
    let p = s["installed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["pin"]["id"] == id)
        .unwrap()["pin"]
        .clone();
    command(
        d,
        Command::Create {
            pin: serde_json::from_value(p).unwrap(),
        },
    )
}
fn ui_request(s: &Value, c: UiCommand) -> UiRequest {
    UiRequest {
        protocol: "0.1".into(),
        local_token: "local".into(),
        key: serde_json::from_value(s["projection"]["key"].clone()).unwrap(),
        expected_room_revision: s["projection"]["roomRevision"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
        expected_generation: s["projection"]["executionGeneration"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
        command: c,
    }
}
fn ui(d: &mut Desktop, c: UiCommand) -> Value {
    let s = state(d);
    let request = ui_request(&s, c);
    command(d, Command::Ui { request })
}
fn calc(d: &mut Desktop, principal: &str, rate: &str, term: &str) -> Value {
    ui(
        d,
        UiCommand::Apply {
            proposed_values: BTreeMap::from([
                ("principal".into(), json!(principal)),
                ("rate".into(), json!(rate)),
                ("term".into(), json!(term)),
            ]),
        },
    )
}
fn value(s: &Value, id: &str) -> Value {
    s["projection"]["values"][id].clone()
}
#[test]
fn vs2_004_exact_pins_and_tampered_release_fail_closed() {
    let f = Fixture::new();
    let mut d = f.desktop();
    assert_eq!(
        create(&mut d, "org.workbench.example.mortgage-ui")["accepted"],
        true
    );
    calc(&mut d, "120000.00", "0", "240");
    let original = f.snapshot();
    drop(d);
    let receipt = f.registry.discover(None).unwrap().remove(0);
    let _handle = f.registry.resolve_exact(&receipt.pin).unwrap();
    let path = f
        .tmp
        .path()
        .join("registry/objects")
        .join(f.registry.admit_directory(&f.source).unwrap().record_digest)
        .join("payload/logic.wasm");
    fs::write(path, b"tampered").unwrap();
    let mut restored = f.desktop();
    assert!(restored.recovery().is_some());
    let frame = state(&mut restored);
    assert!(frame["projection"].is_null());
    assert!(frame["document"].is_null());
    command(&mut restored, Command::Quit);
    assert_eq!(f.snapshot(), original);
}
#[test]
fn vs2_006_declared_bindings_and_readonly_output() {
    let f = Fixture::new();
    let mut d = f.desktop();
    let frame = create(&mut d, "org.workbench.example.mortgage-ui");
    let controls = frame["document"]["root"]["children"].as_array().unwrap();
    assert_eq!(controls.len(), 4);
    assert!(controls
        .iter()
        .filter(|c| c["type"] != "readout")
        .all(|c| c["bind"]["port"] == "calculate"));
    let r = ui(
        &mut d,
        UiCommand::Edit {
            control_id: "payment".into(),
            proposed_value: json!("0.01"),
        },
    );
    assert_eq!(r["accepted"], false);
    assert_eq!(d.invocation_count(), 0);
}
#[test]
fn vs2_007_real_wasm_two_accepted_financial_scenarios() {
    let f = Fixture::new();
    let mut d = f.desktop();
    create(&mut d, "org.workbench.example.mortgage-ui");
    assert_eq!(
        value(&calc(&mut d, "250000.00", "5.00", "300"), "payment"),
        "1461.48"
    );
    assert_eq!(
        value(&calc(&mut d, "120000.00", "0", "240"), "payment"),
        "500.00"
    );
    assert_eq!(d.invocation_count(), 2);
}
#[test]
fn vs2_008_actual_host_ingress_audit_and_committed_projection() {
    let f = Fixture::new();
    let mut d = f.desktop();
    create(&mut d, "org.workbench.example.mortgage-ui");
    let r = calc(&mut d, "250000.00", "5.00", "300");
    assert_eq!(r["accepted"], true);
    let t = d.trace().last().unwrap();
    assert!(t["ingress"]["source"]["instanceId"]
        .as_str()
        .unwrap()
        .starts_with("@ui:"));
    assert_eq!(t["ingress"]["destination"]["port"], "calculate");
    assert_eq!(t["ingress"]["payload"]["principal"], "250000.00");
    assert!(t["audit"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["kind"] == "invocation.completed"));
    assert_eq!(
        t["committedOutput"]["payment"]["monthly_payment"],
        "1461.48"
    );
}
#[test]
fn vs2_009_invalid_incomplete_and_negative_edits_never_execute() {
    let f = Fixture::new();
    let mut d = f.desktop();
    create(&mut d, "org.workbench.example.mortgage-ui");
    let original = f.snapshot();
    let revision = d.workspace().rooms[0].room.revision;
    for (p, r, t) in [
        ("-1", "5", "300"),
        ("1.00", "5", "0"),
        ("1.00", "5", "1.5"),
        ("1,000", "5", "300"),
        ("1.00", "NaN", "300"),
    ] {
        let frame = calc(&mut d, p, r, t);
        assert_eq!(frame["accepted"], false);
        assert!(frame["diagnostics"][0]["controlId"].is_string());
        assert_eq!(d.invocation_count(), 0);
        assert_eq!(d.workspace().rooms[0].room.revision, revision);
        assert_eq!(f.snapshot(), original);
    }
}
#[test]
fn vs2_010_cancellation_and_revision_generation_fences() {
    use wb_contracts::{
        ipc::{CompositionPlan, Endpoint, InstanceSpec},
        ComponentRef,
    };
    use wb_kernel::Kernel;
    let f = Fixture::new();
    let receipt = f.registry.discover(None).unwrap().remove(0);
    let h = f.registry.resolve_exact(&receipt.pin).unwrap();
    let kernel = Kernel::new(
        wb_wasm_host::Limits {
            fuel: 20_000_000,
            ..Default::default()
        },
        4,
        4,
    )
    .unwrap();
    let pin = kernel.cache_registry_release(&f.registry, &h).unwrap();
    kernel
        .approve_execution("fence-room", "component", &pin)
        .unwrap();
    let reference = wb_contracts::ipc::UiReference {
        definition_ref: "org.workbench.example.mortgage-ui/ui.json".into(),
    };
    let mut room = kernel
        .create_room("fence-room", "Fences", reference.clone(), 4)
        .unwrap();
    room.apply_plan(
        CompositionPlan {
            plan_version: "0.1".into(),
            room_id: "fence-room".into(),
            expected_room_revision: 0,
            instances: vec![InstanceSpec {
                instance_id: "component".into(),
                component: ComponentRef {
                    id: pin.id,
                    version: pin.version,
                    sha256: pin.sha256,
                },
            }],
            connections: vec![],
            ui: reference,
            requested_permissions: vec![],
        },
        "test",
        "plan",
    )
    .unwrap();
    room.prepare("component", "test", "prepare").unwrap();
    room.start("component", "test", "start").unwrap();
    let adapter = room
        .authorize_ui(
            "test",
            [Endpoint::new("component", "calculate")]
                .into_iter()
                .collect(),
        )
        .unwrap();
    room.submit_ui(
        &adapter,
        "old",
        Endpoint::new("component", "calculate"),
        json!({"principal":"250000.00","annual_rate_percent":"5.00","term_months":300}),
        "old",
    )
    .unwrap();
    let ticket = room.next_invocation().unwrap().unwrap();
    let old_generation = ticket.context().execution_generation;
    let completion = ticket.run();
    room.stop("component", "test", "cancel").unwrap();
    assert_ne!(room.generation("component").unwrap(), old_generation);
    assert!(room.complete(completion).is_err());
    assert!(room
        .output("component", "payment")
        .is_none_or(|p| p.value.is_none()));
}
#[test]
fn vs2_011_bounded_framing_trace_and_live_audit_recycling() {
    let f = Fixture::new();
    let mut d = f.desktop();
    create(&mut d, "org.workbench.example.mortgage-ui");
    for _ in 0..140 {
        let r = calc(&mut d, "1.00", "0", "1");
        assert_eq!(r["accepted"], true);
        assert!(d.live_audit_entries() <= 280);
        assert!(serde_json::to_vec(&r).unwrap().len() < wb_desktop_host::framing::MAX_FRAME);
    }
    assert_eq!(d.trace().len(), 128);
    let mut invalid =
        std::io::Cursor::new(((wb_desktop_host::framing::MAX_FRAME + 1) as u32).to_be_bytes());
    assert!(wb_desktop_host::framing::read_frame(&mut invalid).is_err());
}
#[test]
fn vs2_012_conflict_preserves_draft_until_explicit_reapply() {
    let f = Fixture::new();
    let mut d = f.desktop();
    let before = create(&mut d, "org.workbench.example.mortgage-ui");
    calc(&mut d, "250000.00", "5.00", "300");
    let snapshot = f.snapshot();
    let invocations = d.invocation_count();
    let request = ui_request(
        &before,
        UiCommand::Apply {
            proposed_values: BTreeMap::from([
                ("principal".into(), json!("120000.00")),
                ("rate".into(), json!("0")),
                ("term".into(), json!("240")),
            ]),
        },
    );
    let conflict = command(&mut d, Command::Ui { request });
    assert_eq!(conflict["diagnostics"][0]["code"], "STATE_CONFLICT");
    assert_eq!(f.snapshot(), snapshot);
    assert_eq!(d.invocation_count(), invocations);
    let repaired = ui(
        &mut d,
        UiCommand::Apply {
            proposed_values: BTreeMap::new(),
        },
    );
    assert_eq!(value(&repaired, "payment"), "500.00");
    assert_eq!(d.invocation_count(), invocations + 1);
}
#[test]
fn vs2_013_guest_trap_invalid_output_and_partial_batches_preserve_checkpoint() {
    let f = Fixture::new();
    f.adversary();
    let mut d = f.desktop();
    create(&mut d, "org.workbench.example.vs2-adversary-ui");
    let first = calc(&mut d, "1.00", "0", "1");
    assert_eq!(value(&first, "payment"), "123.45");
    let snapshot = f.snapshot();
    for principal in ["2.00", "3.00", "4.00", "5.00"] {
        let failed = calc(&mut d, principal, "0", "1");
        assert_eq!(failed["accepted"], false);
        assert_eq!(value(&failed, "payment"), "123.45");
        assert_eq!(f.snapshot(), snapshot);
        assert_eq!(
            d.workspace().rooms[0].room.state["component.calculate"]["principal"],
            "1.00"
        );
        assert_eq!(failed["lastTrace"]["checkpointPreserved"], true);
        assert!(failed["lastTrace"]["audit"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["kind"] == "invocation.failed"));
    }
}
#[test]
fn vs2_016_rooms_and_forged_edit_ownership_are_isolated() {
    let f = Fixture::new();
    let mut d = f.desktop();
    let a = create(&mut d, "org.workbench.example.mortgage-ui");
    let aid = a["selectedRoom"].as_str().unwrap().to_owned();
    calc(&mut d, "250000.00", "5.00", "300");
    let a = state(&mut d);
    let b = create(&mut d, "org.workbench.example.mortgage-ui");
    let bid = b["selectedRoom"].as_str().unwrap().to_owned();
    calc(&mut d, "120000.00", "0", "240");
    let old = ui_request(
        &a,
        UiCommand::Edit {
            control_id: "principal".into(),
            proposed_value: json!("1.00"),
        },
    );
    assert_eq!(
        command(&mut d, Command::Ui { request: old })["accepted"],
        false
    );
    let count = d.invocation_count();
    assert_eq!(
        value(
            &command(&mut d, Command::Select { room_id: aid }),
            "payment"
        ),
        "1461.48"
    );
    assert_eq!(
        value(
            &command(&mut d, Command::Select { room_id: bid }),
            "payment"
        ),
        "500.00"
    );
    assert_eq!(count, d.invocation_count());
}
#[test]
fn vs2_017_pin_unpin_and_selection_are_durable() {
    let f = Fixture::new();
    let mut d = f.desktop();
    let created = create(&mut d, "org.workbench.example.mortgage-ui");
    let id = created["selectedRoom"].as_str().unwrap().to_owned();
    command(&mut d, Command::Pin { pinned: true });
    drop(d);
    let mut restored = f.desktop();
    assert_eq!(
        restored.workspace().selected_room.as_deref(),
        Some(id.as_str())
    );
    assert!(restored.workspace().rooms[0].pinned);
    assert_eq!(restored.invocation_count(), 0);
    command(&mut restored, Command::Pin { pinned: false });
    drop(restored);
    assert!(!f.desktop().workspace().rooms[0].pinned);
}
#[test]
fn vs2_020_invalid_snapshot_latches_readonly_recovery() {
    let f = Fixture::new();
    let mut d = f.desktop();
    create(&mut d, "org.workbench.example.mortgage-ui");
    calc(&mut d, "250000.00", "5.00", "300");
    let good = f.snapshot();
    drop(d);
    let path = f.tmp.path().join("workspace/workspace.json");
    let duplicate = String::from_utf8(good.clone()).unwrap().replace(
        "\"term_months\":300",
        "\"term_months\":300,\"term_months\":300",
    );
    for original in [b"{\"truncated\":".to_vec(), duplicate.into_bytes()] {
        fs::write(&path, &original).unwrap();
        let mut d = f.desktop();
        assert!(d.recovery().is_some());
        let frame = state(&mut d);
        assert!(frame["projection"].is_null());
        assert_eq!(d.invocation_count(), 0);
        command(&mut d, Command::Pin { pinned: true });
        command(&mut d, Command::Quit);
        assert_eq!(fs::read(&path).unwrap(), original);
        drop(d);
    }
    fs::write(&path, &good).unwrap();
    {
        let store = FileStore::open(&f.tmp.path().join("workspace")).unwrap();
        let mut bad = store.load().unwrap().unwrap();
        bad.rooms[0]
            .outputs
            .insert("payment".into(), json!({"monthly_payment":7}));
        store.commit(bad.revision, &bad).unwrap();
    }
    let invalid = f.snapshot();
    let mut d = f.desktop();
    assert_eq!(d.recovery(), Some("INVALID_SNAPSHOT"));
    assert_eq!(d.invocation_count(), 0);
    command(&mut d, Command::Quit);
    assert_eq!(f.snapshot(), invalid);
}
#[test]
fn vs2_021_wire_rejects_extra_identity_and_nested_duplicate_keys() {
    let fixture = Fixture::new();
    let mut desktop = fixture.desktop();
    let too_long = desktop.request(Request {
        protocol: "desktop.1".into(),
        token: "t".repeat(1024 * 1024 - 200),
        epoch: "1".into(),
        command: Command::Quit,
    });
    assert_eq!(too_long["accepted"], false);
    assert!(serde_json::to_vec(&too_long).unwrap().len() < wb_desktop_host::framing::MAX_FRAME);
    create(&mut desktop, "org.workbench.example.mortgage-ui");
    let forged = ui(
        &mut desktop,
        UiCommand::Edit {
            control_id: "x".repeat(1024 * 1024 - 500),
            proposed_value: json!("1"),
        },
    );
    assert_eq!(forged["accepted"], false);
    assert!(serde_json::to_vec(&forged).unwrap().len() < wb_desktop_host::framing::MAX_FRAME);
    assert_eq!(desktop.invocation_count(), 0);

    assert!(serde_json::from_value::<Request>(json!({"protocol":"desktop.1","token":"x","epoch":"1","source":"forged","command":{"type":"quit"}})).is_err());
    assert!(wb_contracts::ui::strict_json(
        br#"{"command":{"request":{"key":{"roomId":"a","roomId":"b"}}}}"#
    )
    .is_err());
}
#[test]
fn vs2_024_frozen_accepted_contracts_and_regressions() {
    use sha2::{Digest, Sha256};
    for (path, expected) in [
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
            "schemas/ui.schema.json",
            "f183fe5d5566d6420fbf6c6ab052a7aa7dc86e41e694a5371551ef51b55c5380",
        ),
        (
            "apps/desktop-qt/qml/Main.qml",
            "ed1533ed44081806e598b533a4f5f9777ccd65fa838d462ea4059ca7920ab13f",
        ),
    ] {
        assert_eq!(
            format!("{:x}", Sha256::digest(fs::read(root().join(path)).unwrap())),
            expected,
            "{}",
            path
        );
    }
}

#[test]
fn vs2_011_large_valid_workspace_response_exceeds_legacy_frame_safely() {
    let f = Fixture::new();
    let echo = f.tmp.path().join("large-ui");
    copy(&root().join("target/registry-fixtures/pure-echo"), &echo);
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(echo.join("component.json")).unwrap()).unwrap();
    let prefix = "org.workbench.maximum.";
    let id = format!("{}{}", prefix, "x".repeat(160 - prefix.len()));
    manifest["id"] = json!(id);
    manifest["ui"] = json!("ui.json");
    fs::write(echo.join("component.json"), manifest.to_string()).unwrap();
    let ttl = fs::read_to_string(echo.join("capabilities.ttl"))
        .unwrap()
        .replace("org.workbench.registry.echo", &id)
        .replace(
            "wb:hasArtifact ex:artifact .",
            "wb:hasArtifact ex:artifact ; wb:hasInterface ex:form .",
        );
    fs::write(
        echo.join("capabilities.ttl"),
        format!("{ttl}\nex:form a wb:AbstractInterface ; wb:uiDefinitionRef \"ui.json\" .\n"),
    )
    .unwrap();
    let schema = json!({"type":"object","additionalProperties":false,"required":["text"],"properties":{"text":{"type":"string","pattern":"^[0-9]+$","minLength":1,"maxLength":510}}});
    for name in ["input", "output"] {
        fs::write(
            echo.join(format!("schemas/{name}.json")),
            schema.to_string(),
        )
        .unwrap();
    }
    let mut controls = vec![];
    for i in 0..32 {
        controls.push(json!({"type":if i==0 {"decimal-input"} else {"readout"},"id":format!("control{i:02}{}","x".repeat(54)),"label":"💷".repeat(160),"unit":"💷".repeat(40),"bind":{"direction":if i==0 {"input"} else {"output"},"port":if i==0 {"input"} else {"result"},"field":"/text"}}));
    }
    let input_id = controls[0]["id"].as_str().unwrap().to_owned();
    fs::write(echo.join("ui.json"),json!({"uiVersion":"0.1","root":{"type":"form","title":"💷".repeat(160),"children":controls}}).to_string()).unwrap();
    f.registry.admit_directory(&echo).unwrap();
    let mut desktop = f.desktop();
    assert!(desktop.recovery().is_none());
    for _ in 0..16 {
        assert_eq!(create(&mut desktop, &id)["accepted"], true);
    }
    let response = ui(
        &mut desktop,
        UiCommand::Apply {
            proposed_values: BTreeMap::from([(input_id, json!("1".repeat(510)))]),
        },
    );
    assert_eq!(response["accepted"], true);
    assert_eq!(desktop.invocation_count(), 1);
    let encoded = serde_json::to_vec(&response).unwrap();
    assert!(encoded.len() > 65536, "measured {} bytes", encoded.len());
    assert!(encoded.len() < wb_desktop_host::framing::MAX_FRAME);
    let mut wire = vec![];
    wb_desktop_host::framing::write_frame(&mut wire, &encoded).unwrap();
    assert_eq!(
        wb_desktop_host::framing::read_frame(std::io::Cursor::new(wire))
            .unwrap()
            .unwrap(),
        encoded
    );
    println!("VS2 maximum valid workspace state: {} bytes,16rooms,32controls,510-digit actual echoed aggregate, transport limit {}",encoded.len(),wb_desktop_host::framing::MAX_FRAME);
}
