//! IPC-001..015 executable conformance. No skips when guest builds are missing.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use wb_bus::ipc::{RoomQueue, RoutedData, Sequence};
use wb_contracts::{ipc::*, ComponentRef};
use wb_kernel::{
    room::{Room, UiAdapter},
    Kernel,
};
use wb_reactive::scheduler::Graph;
use wb_wasm_host::Limits;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn bytes(name: &str) -> Vec<u8> {
    let target = if name == "abi_denied_wasi" {
        "wasm32-wasip1"
    } else {
        "wasm32-unknown-unknown"
    };
    fs::read(root().join(format!("target/abi-guests/{target}/release/{name}.wasm")))
        .expect("run bash examples/wasm-components/build.sh before tests")
}
fn integer() -> Value {
    serde_json::from_slice(
        &fs::read(root().join("examples/wasm-components/fixtures/integer.schema.json")).unwrap(),
    )
    .unwrap()
}
fn aggregate() -> Value {
    json!({"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","required":["value"],"additionalProperties":false,"properties":{"value":{"type":"integer","minimum":-9223372036854775808i64,"maximum":9223372036854775807i64},"add":{"type":"integer","minimum":-9223372036854775808i64,"maximum":9223372036854775807i64},"mode":{"type":"string"}}})
}
fn manifest(id: &str, wasm: &[u8], extra: bool) -> Value {
    let port = |name: &str, schema: &str| json!({"name":name,"schema":schema,"semanticType":"https://workbench.example/ipc#Value"});
    let mut outputs = vec![port("result", "output.json")];
    if extra {
        outputs.push(port("extra", "output.json"));
    }
    json!({"manifestVersion":"0.1","id":format!("org.workbench.ipc.{id}"),"version":"0.1.0","runtime":{"kind":"wasm-component","interface":"org.workbench.component@0.1","module":"logic.wasm","sha256":format!("{:x}",Sha256::digest(wasm))},"metadata":"capabilities.ttl","permissions":[],"ports":{"inputs":[port("input","input.json")],"outputs":outputs}})
}
fn cache(
    k: &Kernel,
    id: &str,
    guest: &str,
    input: Value,
    output: Value,
    extra: bool,
) -> ComponentRef {
    let wasm = bytes(guest);
    k.cache_verified_bundle(
        manifest(id, &wasm, extra),
        wasm,
        BTreeMap::from([("input.json".into(), input), ("output.json".into(), output)]),
    )
    .unwrap()
}
fn kernel(max: usize) -> Kernel {
    Kernel::new(Limits::default(), max, 8).unwrap()
}
fn ui() -> UiReference {
    UiReference {
        definition_ref: "org.workbench.ipc/app/ui.json".into(),
    }
}
fn plan(
    room: &Room,
    instances: &[(&str, &ComponentRef)],
    edges: Vec<Connection>,
) -> CompositionPlan {
    CompositionPlan {
        plan_version: "0.1".into(),
        room_id: room.id().into(),
        expected_room_revision: room.revision(),
        instances: instances
            .iter()
            .map(|(id, pin)| InstanceSpec {
                instance_id: (*id).into(),
                component: (*pin).clone(),
            })
            .collect(),
        connections: edges,
        ui: ui(),
        requested_permissions: vec![],
    }
}
fn edge(from: &str, to: &str) -> Connection {
    Connection {
        from: Endpoint::new(from, "result"),
        to: Endpoint::new(to, "input"),
    }
}
fn declared(
    k: &Kernel,
    id: &str,
    specs: &[(&str, &ComponentRef)],
    edges: Vec<Connection>,
    capacity: usize,
) -> Room {
    for (name, pin) in specs {
        k.approve_execution(id, name, pin).unwrap();
    }
    let mut r = k
        .create_room(id, "IPC conformance", ui(), capacity)
        .unwrap();
    r.apply_plan(plan(&r, specs, edges), "operator", "plan")
        .unwrap();
    r
}
fn start_all(r: &mut Room) {
    for id in r.instance_ids() {
        r.prepare(&id, "host", "prepare").unwrap();
        r.start(&id, "host", "start").unwrap();
    }
}
fn adapter(r: &mut Room, id: &str) -> UiAdapter {
    r.authorize_ui("test-ui", BTreeSet::from([Endpoint::new(id, "input")]))
        .unwrap()
}
fn input(r: &mut Room, a: &UiAdapter, id: &str, request: &str, payload: Value) -> EventEnvelope {
    r.submit_ui(a, request, Endpoint::new(id, "input"), payload, request)
        .unwrap()
}
fn code<T>(result: Result<T>, expected: ErrorCode) {
    match result {
        Ok(_) => panic!("unexpected success; expected {expected:?}"),
        Err(e) => assert_eq!(e.code, expected, "{e:?}"),
    }
}
fn value(r: &Room, id: &str) -> Value {
    let p = r.output(id, "result").unwrap();
    assert!(p.valid, "{p:?}");
    p.value.clone().unwrap()
}

#[test]
fn ipc_001_lifecycle_happy_path_no_guest_at_install() {
    let k = kernel(8);
    let pin = cache(
        &k,
        "aggregate",
        "ipc_aggregate",
        aggregate(),
        integer(),
        false,
    );
    assert_eq!(k.in_flight(), 0);
    let mut r = declared(&k, "room-1", &[("first", &pin)], vec![], 8);
    assert_eq!(r.state("first"), Some(InstanceStatus::Declared));
    assert!(!r.audit().iter().any(|e| e.kind == "invocation.started"));
    r.prepare("first", "host", "p").unwrap();
    r.start("first", "host", "s").unwrap();
    r.stop("first", "user", "x").unwrap();
    let transitions: Vec<_> = r
        .audit()
        .iter()
        .filter(|e| e.kind.starts_with("instance."))
        .map(|e| e.kind.as_str())
        .collect();
    assert_eq!(
        transitions,
        [
            "instance.declared",
            "instance.prepared",
            "instance.started",
            "instance.stopped"
        ]
    );
    assert!(r.audit().windows(2).all(|w| w[0].sequence < w[1].sequence));
    for e in r.audit() {
        assert!(!e.actor.is_empty() && !e.correlation_id.is_empty());
    }
}
#[test]
fn ipc_002_invalid_transitions_and_duplicate_identity() {
    let k = kernel(8);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(&k, "room-2", &[("first", &p)], vec![], 8);
    code(r.start("first", "host", "s"), ErrorCode::InvalidTransition);
    let dup = plan(&r, &[("first", &p), ("first", &p)], vec![]);
    code(r.apply_plan(dup, "operator", "dup"), ErrorCode::DuplicateId);
    start_all(&mut r);
    let a = adapter(&mut r, "first");
    r.stop("first", "host", "stop").unwrap();
    code(
        r.submit_ui(
            &a,
            "input",
            Endpoint::new("first", "input"),
            json!({"value":2}),
            "input",
        ),
        ErrorCode::InvalidTransition,
    );
    code(
        r.stop("first", "host", "again"),
        ErrorCode::InvalidTransition,
    );
}
#[test]
fn ipc_003_admission_pins_world_and_execution_approval() {
    let k = kernel(8);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(&k, "room-3", &[("first", &p)], vec![], 8);
    k.revoke_execution("room-3", "first", &p);
    code(
        r.prepare("first", "host", "deny"),
        ErrorCode::CapabilityDenied,
    );
    assert_eq!(r.state("first"), Some(InstanceStatus::Declared));
    k.approve_execution("room-3", "first", &p).unwrap();
    start_all(&mut r);
    let mut missing = p.clone();
    missing.sha256 = "f".repeat(64);
    code(
        r.apply_plan(plan(&r, &[("first", &missing)], vec![]), "host", "missing"),
        ErrorCode::ComponentNotFound,
    );
    let wasm = b"\0asm\x0d\0\x01\0".to_vec();
    code(
        k.cache_verified_bundle(
            manifest("empty", &wasm, false),
            wasm,
            BTreeMap::from([
                ("input.json".into(), integer()),
                ("output.json".into(), integer()),
            ]),
        ),
        ErrorCode::InvalidManifest,
    );
    let wasm = bytes("abi_denied_wasi");
    code(
        k.cache_verified_bundle(
            manifest("wasi", &wasm, false),
            wasm,
            BTreeMap::from([
                ("input.json".into(), integer()),
                ("output.json".into(), integer()),
            ]),
        ),
        ErrorCode::CapabilityDenied,
    );
    let wasm = bytes("ipc_aggregate");
    let mut m = manifest("grant", &wasm, false);
    m["permissions"] = json!([{"name":"filesystem.read","scope":{}}]);
    code(
        k.cache_verified_bundle(
            m,
            wasm,
            BTreeMap::from([
                ("input.json".into(), integer()),
                ("output.json".into(), integer()),
            ]),
        ),
        ErrorCode::CapabilityDenied,
    );
}
#[test]
fn ipc_004_host_provenance_and_declared_routes() {
    let k = kernel(8);
    let schema = json!({"type":"object","required":["value"],"properties":{"value":{"type":"integer"}},"additionalProperties":true});
    let p = cache(
        &k,
        "forgery",
        "ipc_aggregate",
        schema.clone(),
        schema,
        false,
    );
    let mut r = declared(
        &k,
        "room-4",
        &[("first", &p), ("second", &p)],
        vec![edge("first", "second")],
        8,
    );
    start_all(&mut r);
    let a = adapter(&mut r, "first");
    input(
        &mut r,
        &a,
        "first",
        "forge",
        json!({"value":21,"mode":"forge"}),
    );
    let ticket = r.next_invocation().unwrap().unwrap();
    let events = r.complete(ticket.run()).unwrap();
    assert_eq!(events.len(), 1);
    let e = &events[0];
    assert_eq!(e.source, Endpoint::new("first", "result"));
    assert_eq!(e.destination, Endpoint::new("second", "input"));
    assert_eq!(e.room_id, "room-4");
    assert_ne!(e.sequence, 777);
    assert_eq!(e.payload["sequence"], 777);
    assert_eq!(e.correlation_id, "forge");
    let other = k.create_room("other", "Other", ui(), 8).unwrap();
    let mut other = other;
    code(
        other.submit_ui(
            &a,
            "steal",
            Endpoint::new("first", "input"),
            json!({"value":1}),
            "steal",
        ),
        ErrorCode::CapabilityDenied,
    );
    let b = adapter(&mut r, "second");
    code(
        r.submit_ui(
            &b,
            "bypass",
            Endpoint::new("second", "input"),
            json!({"value":1}),
            "bypass",
        ),
        ErrorCode::IncompatiblePorts,
    );
}
#[test]
fn ipc_005_concurrent_producers_monotonic_sequences_and_lexical_order() {
    let queue = Arc::new(Mutex::new(
        RoomQueue::new("room-5", 1, 128, Arc::new(Sequence::default())).unwrap(),
    ));
    let mut threads = vec![];
    for producer in 0..4 {
        let q = queue.clone();
        threads.push(thread::spawn(move || {
            let mut seq = Vec::new();
            for n in 0..16 {
                let route = RoutedData {
                    source: Endpoint::new(&format!("producer-{producer}"), "out"),
                    destination: Endpoint::new("target", "input"),
                    schema_id: "test".into(),
                    payload: json!({"value":n}),
                    generation: 1,
                    ui_token: None,
                };
                let c = DispatchContext {
                    room_revision: 1,
                    execution_generation: 1,
                    invocation_id: String::new(),
                };
                seq.push(
                    q.lock()
                        .unwrap()
                        .publish(&format!("{producer}:{n}"), "test", c, vec![route])
                        .unwrap()[0]
                        .sequence,
                );
            }
            seq
        }));
    }
    for t in threads {
        let seq = t.join().unwrap();
        assert!(seq.windows(2).all(|w| w[0] < w[1]));
    }
    let mut q = queue.lock().unwrap();
    let mut seq = vec![];
    while let Some(e) = q.pop() {
        seq.push(e.envelope.sequence);
    }
    assert_eq!(seq, (1..=64).collect::<Vec<_>>());
    for ids in [vec!["a", "z", "b", "c"], vec!["z", "c", "b", "a"]] {
        let g = Graph::new(
            ids.into_iter().map(str::to_owned),
            &[edge("a", "z"), edge("a", "b"), edge("b", "c")],
        )
        .unwrap();
        assert_eq!(g.order(), ["a", "b", "c", "z"]);
    }
}
#[test]
fn ipc_006_ingress_and_entire_output_batch_atomicity() {
    let k = kernel(8);
    let p = cache(&k, "batch", "ipc_aggregate", aggregate(), integer(), true);
    let mut r = declared(&k, "room-6", &[("first", &p)], vec![], 8);
    start_all(&mut r);
    let a = adapter(&mut r, "first");
    input(&mut r, &a, "first", "good", json!({"value":10}));
    assert!(r.run_until_idle().is_empty());
    let before = value(&r, "first");
    let rev = r.revision();
    code(
        r.submit_ui(
            &a,
            "bad-add",
            Endpoint::new("first", "input"),
            json!({"value":1,"add":u64::MAX}),
            "bad-add",
        ),
        ErrorCode::SchemaViolation,
    );
    code(
        r.submit_ui(
            &a,
            "bad",
            Endpoint::new("first", "input"),
            json!({"value":"wrong"}),
            "bad",
        ),
        ErrorCode::SchemaViolation,
    );
    assert_eq!(rev, r.revision());
    assert_eq!(value(&r, "first"), before);
    input(
        &mut r,
        &a,
        "first",
        "batch",
        json!({"value":999,"mode":"bad-batch"}),
    );
    let errors = r.run_until_idle();
    assert_eq!(errors[0].code, ErrorCode::SchemaViolation);
    let after = r.output("first", "result").unwrap();
    assert_eq!(after.value.as_ref(), Some(&before));
    assert!(!after.valid);
    assert_eq!(after.error, Some(ErrorCode::SchemaViolation));
    assert!(r.output("first", "extra").is_none());
}
#[test]
fn ipc_007_aggregate_two_components_and_descendants_without_operator() {
    let k = kernel(16);
    let aggregate = cache(
        &k,
        "aggregate",
        "ipc_aggregate",
        aggregate(),
        integer(),
        false,
    );
    let integer = cache(&k, "integer", "abi_integer", integer(), integer(), false);
    let mut r = declared(
        &k,
        "room-7",
        &[
            ("a", &aggregate),
            ("b", &integer),
            ("c", &integer),
            ("z", &integer),
            ("unrelated", &integer),
        ],
        vec![edge("a", "z"), edge("a", "b"), edge("b", "c")],
        16,
    );
    start_all(&mut r);
    let a = adapter(&mut r, "a");
    input(&mut r, &a, "a", "sum", json!({"value":20,"add":1}));
    let revision = r.revision() + 1;
    assert!(r.run_until_idle().is_empty());
    assert_eq!(value(&r, "a"), json!({"value":21}));
    assert_eq!(value(&r, "b"), json!({"value":42}));
    assert_eq!(value(&r, "c"), json!({"value":84}));
    assert_eq!(value(&r, "z"), json!({"value":42}));
    assert!(r.output("unrelated", "result").is_none());
    let calls: Vec<_> = r
        .audit()
        .iter()
        .filter(|e| e.kind == "invocation.started")
        .map(|e| e.instance_id.as_deref().unwrap())
        .collect();
    assert_eq!(calls, ["a", "b", "c", "z"]);
    assert_eq!(r.output("c", "result").unwrap().revision, revision);
    assert!(r
        .audit()
        .iter()
        .filter(|e| e.kind.starts_with("invocation."))
        .all(|e| e.actor != "operator"));
}
#[test]
fn ipc_008_stale_revision_generation_cancel_and_at_most_once() {
    let k = kernel(8);
    let p = cache(
        &k,
        "aggregate",
        "ipc_aggregate",
        aggregate(),
        integer(),
        false,
    );
    let mut r = declared(&k, "room-8", &[("first", &p)], vec![], 8);
    start_all(&mut r);
    let a = adapter(&mut r, "first");
    input(&mut r, &a, "first", "old", json!({"value":1}));
    let ticket = r.next_invocation().unwrap().unwrap();
    let old = ticket.run();
    input(&mut r, &a, "first", "new", json!({"value":2}));
    r.apply_plan(
        plan(&r, &[("first", &p)], vec![]),
        "operator",
        "new-topology",
    )
    .unwrap();
    code(r.complete(old), ErrorCode::StaleResult);
    assert!(r.run_until_idle().is_empty());
    assert_eq!(value(&r, "first"), json!({"value":2}));
    input(&mut r, &a, "first", "cancelled", json!({"value":3}));
    let ticket = r.next_invocation().unwrap().unwrap();
    let generation = r.generation("first").unwrap();
    r.stop("first", "user", "stop").unwrap();
    r.prepare("first", "host", "restart").unwrap();
    r.start("first", "host", "start").unwrap();
    assert_ne!(r.generation("first"), Some(generation));
    code(r.complete(ticket.run()), ErrorCode::StaleResult);
    assert!(r
        .audit()
        .iter()
        .any(|e| e.error_code == Some(ErrorCode::Cancelled) && e.correlation_id == "cancelled"));
    assert_eq!(k.in_flight(), 0);
    input(&mut r, &a, "first", "once", json!({"value":4}));
    assert!(r.run_until_idle().is_empty());
    let rev = r.revision();
    code(
        r.submit_ui(
            &a,
            "once",
            Endpoint::new("first", "input"),
            json!({"value":5}),
            "once",
        ),
        ErrorCode::DuplicateId,
    );
    assert_eq!(rev, r.revision());
}
#[test]
fn ipc_009_queue_saturation_preserves_commits_and_stop_liveness() {
    let k = kernel(8);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(&k, "room-9", &[("first", &p)], vec![], 1);
    start_all(&mut r);
    let a = adapter(&mut r, "first");
    input(&mut r, &a, "first", "committed", json!({"value":1}));
    assert!(r.run_until_idle().is_empty());
    let rev = r.revision();
    input(&mut r, &a, "first", "pending", json!({"value":2}));
    code(
        r.submit_ui(
            &a,
            "full",
            Endpoint::new("first", "input"),
            json!({"value":3}),
            "full",
        ),
        ErrorCode::QueueFull,
    );
    assert_eq!(r.pending().len(), 1);
    assert_eq!(rev, r.revision());
    assert_eq!(r.input("first", "input"), Some(&json!({"value":1})));
    r.stop("first", "user", "stop-full").unwrap();
    assert!(r.pending().is_empty());
    assert_eq!(r.state("first"), Some(InstanceStatus::Stopped));
    assert!(r
        .audit()
        .iter()
        .any(|e| e.kind == "backpressure.rejected" && e.error_code == Some(ErrorCode::QueueFull)));
    assert!(r
        .audit()
        .iter()
        .any(|e| e.error_code == Some(ErrorCode::Cancelled)));
}
#[test]
fn ipc_010_trap_isolation_across_instances_and_rooms() {
    let k = kernel(16);
    let p = cache(
        &k,
        "aggregate",
        "ipc_aggregate",
        aggregate(),
        integer(),
        false,
    );
    let mut one = declared(&k, "room-10a", &[("bad", &p), ("good", &p)], vec![], 8);
    start_all(&mut one);
    let bad = adapter(&mut one, "bad");
    input(
        &mut one,
        &bad,
        "bad",
        "trap",
        json!({"value":1,"mode":"trap"}),
    );
    assert_eq!(one.run_until_idle()[0].code, ErrorCode::ComponentTrap);
    assert_eq!(one.state("bad"), Some(InstanceStatus::Failed));
    let good = adapter(&mut one, "good");
    input(&mut one, &good, "good", "ok", json!({"value":7}));
    assert!(one.run_until_idle().is_empty());
    assert_eq!(value(&one, "good"), json!({"value":7}));
    let mut two = declared(&k, "room-10b", &[("good", &p)], vec![], 8);
    start_all(&mut two);
    let ui = adapter(&mut two, "good");
    input(&mut two, &ui, "good", "ok", json!({"value":8}));
    assert!(two.run_until_idle().is_empty());
    assert_eq!(value(&two, "good"), json!({"value":8}));
}
#[test]
fn ipc_011_atomic_plans_and_reservation_rollback() {
    let k = kernel(4);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(&k, "room-11", &[("a", &p)], vec![], 8);
    let rev = r.revision();
    let reserved = k.reserved_instances();
    k.approve_execution("room-11", "b", &p).unwrap();
    let mut bad = edge("a", "b");
    bad.to.port = "missing".into();
    code(
        r.apply_plan(
            plan(&r, &[("a", &p), ("b", &p)], vec![bad]),
            "operator",
            "bad-edge",
        ),
        ErrorCode::IncompatiblePorts,
    );
    assert_eq!(k.reserved_instances(), reserved);
    assert_eq!(r.instance_ids(), ["a"]);
    assert_eq!(r.revision(), rev);
    code(
        r.apply_plan(
            plan(
                &r,
                &[("a", &p), ("b", &p)],
                vec![edge("a", "b"), edge("b", "a")],
            ),
            "operator",
            "cycle",
        ),
        ErrorCode::CyclicDependency,
    );
    assert_eq!(k.reserved_instances(), reserved);
    let mut stale = plan(&r, &[("a", &p)], vec![]);
    stale.expected_room_revision -= 1;
    code(
        r.apply_plan(stale, "operator", "stale"),
        ErrorCode::StateConflict,
    );
    assert_eq!(r.revision(), rev);
    assert!(r.connections().is_empty());
    r.apply_plan(plan(&r, &[], vec![]), "operator", "remove")
        .unwrap();
    assert_eq!(k.reserved_instances(), 0);
}
#[test]
fn ipc_012_restore_pins_host_state_no_queue_or_grant_replay() {
    let k = kernel(8);
    let p = cache(
        &k,
        "aggregate",
        "ipc_aggregate",
        aggregate(),
        integer(),
        false,
    );
    let integer_pin = cache(&k, "integer", "abi_integer", integer(), integer(), false);
    let mut r = declared(
        &k,
        "room-12",
        &[("a", &p), ("b", &integer_pin)],
        vec![edge("a", "b")],
        8,
    );
    start_all(&mut r);
    let old_ui = adapter(&mut r, "a");
    input(&mut r, &old_ui, "a", "sum", json!({"value":20,"add":1}));
    assert!(r.run_until_idle().is_empty());
    let snapshot = r.snapshot();
    let generation = r.generation("a");
    let encoded = serde_json::to_value(&snapshot).unwrap();
    assert!(!encoded.to_string().contains("generation"));
    assert!(encoded.get("grants").is_none());
    assert!(encoded.get("queue").is_none());
    drop(r);
    let mut restored = k.restore_room(snapshot.clone(), 8).unwrap();
    assert_ne!(restored.generation("a"), generation);
    assert_eq!(restored.connections(), snapshot.connections);
    code(
        restored.submit_ui(
            &old_ui,
            "old-adapter",
            Endpoint::new("a", "input"),
            json!({"value":0}),
            "old",
        ),
        ErrorCode::CapabilityDenied,
    );
    assert!(restored.run_until_idle().is_empty());
    assert_eq!(value(&restored, "b"), json!({"value":42}));
    drop(restored);
    let fresh = kernel(8);
    cache(
        &fresh,
        "aggregate",
        "ipc_aggregate",
        aggregate(),
        integer(),
        false,
    );
    cache(
        &fresh,
        "integer",
        "abi_integer",
        integer(),
        integer(),
        false,
    );
    code(
        fresh.restore_room(snapshot.clone(), 8),
        ErrorCode::CapabilityDenied,
    );
    assert_eq!(fresh.reserved_instances(), 0);
    let mut missing = snapshot.clone();
    missing.components[0].sha256 = "f".repeat(64);
    code(k.restore_room(missing, 8), ErrorCode::ComponentNotFound);
    assert_eq!(k.reserved_instances(), 0);
}
#[test]
fn ipc_013_correlated_error_classes_and_redacted_audit() {
    let k = kernel(8);
    let p = cache(
        &k,
        "aggregate",
        "ipc_aggregate",
        aggregate(),
        integer(),
        false,
    );
    let mut r = declared(&k, "room-13", &[("a", &p)], vec![], 8);
    start_all(&mut r);
    let a = adapter(&mut r, "a");
    for (mode, expected) in [
        ("error", ErrorCode::GuestError),
        ("loop", ErrorCode::ResourceLimit),
        ("trap", ErrorCode::ComponentTrap),
    ] {
        if r.state("a") == Some(InstanceStatus::Failed) {
            r.prepare("a", "host", "restart").unwrap();
            r.start("a", "host", "start").unwrap();
        }
        input(&mut r, &a, "a", mode, json!({"value":1,"mode":mode}));
        let errors = r.run_until_idle();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, expected);
        if mode == "error" {
            assert_eq!(errors[0].detail["guestCode"], "EXPECTED");
            assert_eq!(
                errors[0].detail["guestMessage"],
                "private guest diagnostics"
            );
        }
        assert_eq!(errors[0].correlation_id.as_deref(), Some(mode));
        assert!(r.audit().iter().any(|e| e.kind == "invocation.failed"
            && e.error_code == Some(expected)
            && e.correlation_id == mode));
    }
    let serialized = serde_json::to_string(r.audit()).unwrap();
    assert!(!serialized.contains("private guest diagnostics"));
    assert!(!serialized.contains("\"payload\""));
}
#[test]
fn ipc_014_accepted_wit_and_canonical_dto_regression() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(fs::read(root().join("wit/component.wit")).unwrap())
        ),
        "cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0"
    );
    let path = root().join("fixtures/ipc/valid/envelope.json");
    let e = EventEnvelope::parse(&fs::read_to_string(path).unwrap()).unwrap();
    let value = serde_json::to_value(&e).unwrap();
    assert!(value.get("instanceId").is_none());
    assert!(value.get("roomId").is_some());
    assert_eq!(EventEnvelope::parse(&value.to_string()).unwrap(), e);
    for entry in fs::read_dir(root().join("fixtures/ipc/invalid")).unwrap() {
        assert!(EventEnvelope::parse(&fs::read_to_string(entry.unwrap().path()).unwrap()).is_err());
    }
    let mut v = value;
    v["protocolVersion"] = json!("9");
    code(
        EventEnvelope::parse(&v.to_string()),
        ErrorCode::UnsupportedVersion,
    );
    let error: HostError = serde_json::from_str(
        &fs::read_to_string(root().join("fixtures/ipc/errors/valid/error.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(error.code, ErrorCode::ResourceLimit);
    assert_eq!(
        serde_json::from_value::<HostError>(serde_json::to_value(&error).unwrap()).unwrap(),
        error
    );
    for entry in fs::read_dir(root().join("fixtures/ipc/errors/invalid")).unwrap() {
        assert!(serde_json::from_str::<HostError>(
            &fs::read_to_string(entry.unwrap().path()).unwrap()
        )
        .is_err());
    }
    let plan_json: Value = serde_json::from_str(
        &fs::read_to_string(root().join("fixtures/valid/composition.json")).unwrap(),
    )
    .unwrap();
    let plan_dto: CompositionPlan = serde_json::from_value(plan_json.clone()).unwrap();
    assert_eq!(serde_json::to_value(plan_dto).unwrap(), plan_json);
    let room_json: Value =
        serde_json::from_str(&fs::read_to_string(root().join("fixtures/valid/room.json")).unwrap())
            .unwrap();
    let room_dto: RoomSnapshot = serde_json::from_value(room_json.clone()).unwrap();
    assert_eq!(serde_json::to_value(room_dto).unwrap(), room_json);
    for error in [
        ErrorCode::QueueFull,
        ErrorCode::StaleResult,
        ErrorCode::GuestError,
        ErrorCode::ResourceLimit,
        ErrorCode::Cancelled,
    ] {
        let s = serde_json::to_string(&error).unwrap();
        assert_eq!(serde_json::from_str::<ErrorCode>(&s).unwrap(), error);
    }
}
#[test]
fn ipc_015_ci_declarations_are_not_a_hosted_success_claim() {
    let workflow = fs::read_to_string(root().join(".github/workflows/ci.yml")).unwrap();
    assert!(workflow.contains("ipc-validation"));
    assert!(workflow.contains("examples/wasm-components/validate-ipc.sh"));
    assert!(workflow.contains("1.99.1"));
    let lock = fs::read_to_string(root().join("Cargo.lock")).unwrap();
    assert!(lock.contains("name = \"wasmtime\"\nversion = \"49.0.1\""));
}

#[test]
fn ipc_005_kernel_waves_preserve_ui_fifo_and_all_committed_updates() {
    let k = kernel(16);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(
        &k,
        "waves",
        &[("a", &p), ("a-child", &p), ("z", &p), ("z-child", &p)],
        vec![edge("a", "a-child"), edge("z", "z-child")],
        16,
    );
    start_all(&mut r);
    let a = r
        .authorize_ui(
            "one-producer",
            BTreeSet::from([Endpoint::new("z", "input"), Endpoint::new("a", "input")]),
        )
        .unwrap();
    let before = r.revision();
    let z = input(&mut r, &a, "z", "z-first", json!({"value":1}));
    let a_event = input(&mut r, &a, "a", "a-second", json!({"value":2}));
    assert!(z.sequence < a_event.sequence);
    assert_eq!(r.revision(), before);
    assert!(r.run_until_idle().is_empty());
    assert_eq!(r.revision(), before + 2);
    let calls: Vec<_> = r
        .audit()
        .iter()
        .filter(|e| e.kind == "invocation.started")
        .map(|e| e.instance_id.as_deref().unwrap())
        .collect();
    assert_eq!(calls, ["z", "z-child", "a", "a-child"]);
    assert_eq!(value(&r, "z-child"), json!({"value":1}));
    assert_eq!(value(&r, "a-child"), json!({"value":2}));
    input(&mut r, &a, "z", "again-1", json!({"value":3}));
    input(&mut r, &a, "z", "again-2", json!({"value":4}));
    assert!(r.run_until_idle().is_empty());
    assert_eq!(value(&r, "z-child"), json!({"value":4}));
    assert_eq!(r.revision(), before + 4);
}
#[test]
fn ipc_008_new_pending_ui_and_unrelated_stop_do_not_lose_active_work() {
    let k = kernel(8);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(&k, "active", &[("a", &p), ("b", &p), ("c", &p)], vec![], 8);
    start_all(&mut r);
    let a = adapter(&mut r, "a");
    let b = adapter(&mut r, "b");
    input(&mut r, &a, "a", "a", json!({"value":1}));
    let ticket = r.next_invocation().unwrap().unwrap();
    let revision = r.revision();
    input(&mut r, &b, "b", "b", json!({"value":2}));
    assert_eq!(r.revision(), revision);
    r.stop("c", "user", "unrelated-stop").unwrap();
    assert_eq!(r.revision(), revision);
    r.complete(ticket.run()).unwrap();
    assert!(r.run_until_idle().is_empty());
    assert_eq!(value(&r, "a"), json!({"value":1}));
    assert_eq!(value(&r, "b"), json!({"value":2}));
}
#[test]
fn ipc_012_shared_direction_port_name_preserves_authoritative_input() {
    let k = kernel(8);
    let wasm = bytes("abi_integer");
    let mut m = manifest("shared-port", &wasm, false);
    m["ports"]["inputs"][0]["name"] = json!("result");
    let p = k
        .cache_verified_bundle(
            m,
            wasm,
            BTreeMap::from([
                ("input.json".into(), integer()),
                ("output.json".into(), integer()),
            ]),
        )
        .unwrap();
    let mut r = declared(&k, "shared", &[("first", &p)], vec![], 8);
    start_all(&mut r);
    let a = r
        .authorize_ui("ui", BTreeSet::from([Endpoint::new("first", "result")]))
        .unwrap();
    r.submit_ui(
        &a,
        "input",
        Endpoint::new("first", "result"),
        json!({"value":12}),
        "input",
    )
    .unwrap();
    assert!(r.run_until_idle().is_empty());
    assert_eq!(value(&r, "first"), json!({"value":24}));
    let snapshot = r.snapshot();
    assert_eq!(snapshot.state["first.result"], json!({"value":12}));
    drop(r);
    let mut r = k.restore_room(snapshot, 8).unwrap();
    assert!(r.run_until_idle().is_empty());
    assert_eq!(r.input("first", "result"), Some(&json!({"value":12})));
    assert_eq!(value(&r, "first"), json!({"value":24}));
}
#[test]
fn ipc_009_output_fanout_full_rejects_whole_batch() {
    let k = kernel(8);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(
        &k,
        "fanout",
        &[("a", &p), ("b", &p), ("c", &p)],
        vec![edge("a", "b"), edge("a", "c")],
        1,
    );
    start_all(&mut r);
    let a = adapter(&mut r, "a");
    input(&mut r, &a, "a", "fanout", json!({"value":1}));
    let errors = r.run_until_idle();
    assert_eq!(errors[0].code, ErrorCode::QueueFull);
    assert!(r.output("a", "result").is_none());
    assert!(r.input("b", "input").is_none());
    assert!(r.input("c", "input").is_none());
    assert!(r.pending().is_empty());
    assert_eq!(r.input("a", "input"), Some(&json!({"value":1})));
}

#[test]
fn ipc_008_stopping_upstream_quarantines_active_descendant() {
    let k = kernel(8);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(
        &k,
        "cancel-descendant",
        &[("a", &p), ("b", &p), ("c", &p)],
        vec![edge("a", "b"), edge("b", "c")],
        8,
    );
    start_all(&mut r);
    let a = adapter(&mut r, "a");
    input(&mut r, &a, "a", "wave", json!({"value":1}));
    let first = r.next_invocation().unwrap().unwrap();
    r.complete(first.run()).unwrap();
    let descendant = r.next_invocation().unwrap().unwrap();
    assert_eq!(descendant.context().room_revision, r.revision());
    let result = descendant.run();
    r.stop("a", "user", "stop-root").unwrap();
    code(r.complete(result), ErrorCode::StaleResult);
    assert!(r.run_until_idle().is_empty());
    assert!(r.output("b", "result").is_none());
    assert!(r.output("c", "result").is_none());
    assert!(r
        .audit()
        .iter()
        .any(|e| e.instance_id.as_deref() == Some("b")
            && e.error_code == Some(ErrorCode::Cancelled)
            && e.correlation_id == "wave"));
}

#[test]
fn ipc_003_approvals_and_adapters_rechecked_at_time_of_use() {
    let k = kernel(4);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut r = declared(&k, "revoked", &[("a", &p)], vec![], 8);
    start_all(&mut r);
    let a = adapter(&mut r, "a");
    input(&mut r, &a, "a", "before-revoke", json!({"value":1}));
    let completion = r.next_invocation().unwrap().unwrap().run();
    k.revoke_execution("revoked", "a", &p);
    code(r.complete(completion), ErrorCode::CapabilityDenied);
    assert!(r.output("a", "result").is_none());
    k.approve_execution("revoked", "a", &p).unwrap();
    input(&mut r, &a, "a", "adapter-revoke", json!({"value":2}));
    let rev = r.revision();
    r.revoke_ui(&a);
    let errors = r.run_until_idle();
    assert_eq!(errors[0].code, ErrorCode::CapabilityDenied);
    assert_eq!(r.revision(), rev);
    assert_eq!(r.input("a", "input"), Some(&json!({"value":1})));
    let mut raw = serde_json::to_value(plan(&r, &[("a", &p)], vec![])).unwrap();
    raw["instances"][0]["component"]
        .as_object_mut()
        .unwrap()
        .remove("sha256");
    assert!(serde_json::from_value::<CompositionPlan>(raw).is_err());
}
#[test]
fn ipc_011_quota_failure_releases_staged_reservations() {
    let k = kernel(1);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    k.approve_execution("quota", "a", &p).unwrap();
    k.approve_execution("quota", "b", &p).unwrap();
    let mut r = k.create_room("quota", "Quota", ui(), 8).unwrap();
    code(
        r.apply_plan(
            plan(&r, &[("a", &p), ("b", &p)], vec![]),
            "operator",
            "quota",
        ),
        ErrorCode::ResourceLimit,
    );
    assert_eq!(k.reserved_instances(), 0);
    assert!(r.instance_ids().is_empty());
    assert_eq!(r.revision(), 0);
}

#[test]
fn ipc_001_transition_table_and_resource_release() {
    let k = kernel(8);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    for state in [
        InstanceStatus::Declared,
        InstanceStatus::Prepared,
        InstanceStatus::Running,
        InstanceStatus::Stopped,
    ] {
        let name = format!("table-{state:?}");
        let mut r = declared(&k, &name, &[("a", &p)], vec![], 4);
        code(
            r.stop("a", "host", "declared-stop"),
            ErrorCode::InvalidTransition,
        );
        if state != InstanceStatus::Declared {
            r.prepare("a", "host", "prepare").unwrap();
            code(
                r.prepare("a", "host", "prepared-prepare"),
                ErrorCode::InvalidTransition,
            );
            code(
                r.stop("a", "host", "prepared-stop"),
                ErrorCode::InvalidTransition,
            );
        }
        if matches!(state, InstanceStatus::Running | InstanceStatus::Stopped) {
            r.start("a", "host", "start").unwrap();
            code(
                r.start("a", "host", "running-start"),
                ErrorCode::InvalidTransition,
            );
            code(
                r.prepare("a", "host", "running-prepare"),
                ErrorCode::InvalidTransition,
            );
        }
        if state == InstanceStatus::Stopped {
            r.stop("a", "host", "stop").unwrap();
            code(
                r.start("a", "host", "stopped-start"),
                ErrorCode::InvalidTransition,
            );
        }
        assert_eq!(r.state("a"), Some(state));
        r.apply_plan(plan(&r, &[], vec![]), "operator", "remove")
            .unwrap();
        assert_eq!(k.reserved_instances(), 0);
        assert!(r.instance_ids().is_empty());
    }
}
#[test]
fn ipc_010_rooms_execute_concurrently_with_isolated_stores() {
    let k = kernel(8);
    let p = cache(&k, "identity", "ipc_aggregate", integer(), integer(), false);
    let mut rooms = vec![];
    for (id, n) in [("parallel-a", 11), ("parallel-b", 22)] {
        let mut r = declared(&k, id, &[("same-logical-id", &p)], vec![], 8);
        start_all(&mut r);
        let a = adapter(&mut r, "same-logical-id");
        input(&mut r, &a, "same-logical-id", "input", json!({"value":n}));
        rooms.push((r, n));
    }
    let threads: Vec<_> = rooms
        .into_iter()
        .map(|(mut r, n)| {
            thread::spawn(move || {
                assert!(r.run_until_idle().is_empty());
                assert_eq!(value(&r, "same-logical-id"), json!({"value":n}));
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(k.in_flight(), 0);
    assert_eq!(k.reserved_instances(), 0);
}
