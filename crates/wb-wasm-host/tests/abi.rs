use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use wb_wasm_host::{Failure, Host, Limits, Message, Registered};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn artifact(name: &str) -> Vec<u8> {
    let target = if name == "denied-wasi" {
        "wasm32-wasip1"
    } else {
        "wasm32-unknown-unknown"
    };
    fs::read(root().join(format!(
        "target/abi-guests/{target}/release/abi_{}.wasm",
        name.replace('-', "_")
    )))
    .expect("build required components first: bash examples/wasm-components/build.sh")
}
fn manifest(name: &str) -> Value {
    serde_json::from_slice(
        &fs::read(root().join(format!("target/abi-fixtures/{name}/component.json")))
            .expect("run guest build script"),
    )
    .unwrap()
}
fn schemas(name: &str) -> BTreeMap<String, Value> {
    let file = if name == "integer" { "integer" } else { "text" };
    BTreeMap::from([(
        "payload.schema.json".into(),
        serde_json::from_slice(
            &fs::read(root().join(format!(
                "examples/wasm-components/fixtures/{file}.schema.json"
            )))
            .unwrap(),
        )
        .unwrap(),
    )])
}
fn register(host: &Host, name: &str) -> Registered {
    host.register(&manifest(name), &artifact(name), &schemas(name))
        .unwrap()
}
fn msg(text: &str) -> Message {
    Message {
        port: "input".into(),
        payload_json: json!({"text": text}).to_string(),
    }
}
fn healthy(host: &Host, guest: &Registered) {
    let output = host.call(guest, &msg("hello")).unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].port, "result");
    assert_eq!(
        serde_json::from_str::<Value>(&output[0].payload_json).unwrap(),
        json!({"text":"HELLO"})
    );
}
fn host() -> Host {
    Host::new(Limits::default()).unwrap()
}

#[test]
fn abi_001_wit_bindings_and_real_component_encoding() {
    for name in ["echo", "integer"] {
        let bytes = artifact(name);
        let first = wasmparser::Parser::new(0)
            .parse_all(&bytes)
            .next()
            .unwrap()
            .unwrap();
        assert!(matches!(
            first,
            wasmparser::Payload::Version {
                encoding: wasmparser::Encoding::Component,
                ..
            }
        ));
        register(&host(), name); // Generated host and cargo-component bindings agree.
    }
}
#[test]
fn abi_002_basic_typed_call() {
    let host = host();
    healthy(&host, &register(&host, "echo"));
}
#[test]
fn abi_003_independent_integer_guest_same_host() {
    let host = host();
    let echo = register(&host, "echo");
    let integer = register(&host, "integer");
    assert_ne!(
        Sha256::digest(artifact("echo")),
        Sha256::digest(artifact("integer"))
    );
    let result = host
        .call(
            &integer,
            &Message {
                port: "input".into(),
                payload_json: "{\"value\":21}".into(),
            },
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&result[0].payload_json).unwrap(),
        json!({"value":42})
    );
    healthy(&host, &echo);
}
#[test]
fn abi_004_world_mismatch_and_core_module_rejected() {
    let host = host();
    for bytes in [
        b"(module)".as_slice(),
        b"(component)",
        br#"(component
        (core module $m (func (export "compute")))
        (core instance $i (instantiate $m))
        (func $compute (canon lift (core func $i "compute")))
        (instance $exports (export "compute" (func $compute)))
        (export "workbench:component/computation@0.1.0" (instance $exports)))"#,
        b"not wasm",
    ] {
        let bytes = wat::parse_bytes(bytes)
            .map(|b| b.into_owned())
            .unwrap_or_else(|_| bytes.to_vec());
        let mut manifest = manifest("echo");
        manifest["runtime"]["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
        let result = host.register(&manifest, &bytes, &schemas("echo"));
        assert!(
            matches!(&result, Err(Failure::World)),
            "fixture {bytes:?}: {:?}",
            result.err()
        );
    }
}
#[test]
fn abi_005_compiled_wasi_guest_is_denied() {
    let host = host();
    let bytes = artifact("denied-wasi");
    // Verify that this really is a compiled component with filesystem imports.
    let engine = wasmtime::Engine::default();
    let component = wasmtime::component::Component::new(&engine, &bytes).unwrap();
    assert!(wasmtime::component::Linker::<()>::new(&engine)
        .instantiate_pre(&component)
        .is_err());
    let imports: Vec<_> = component
        .component_type()
        .imports(&engine)
        .map(|(name, _)| name.to_string())
        .collect();
    assert!(
        imports.iter().any(|s| s.starts_with("wasi:filesystem/")),
        "{imports:?}"
    );
    assert!(matches!(
        host.register(&manifest("denied-wasi"), &bytes, &schemas("echo")),
        Err(Failure::DeniedImport)
    ));
    healthy(&host, &register(&host, "echo"));
}
#[test]
fn abi_006_guest_error_trap_and_isolation() {
    let host = host();
    let echo = register(&host, "echo");
    assert!(
        matches!(host.call(&echo, &msg("guest-error")), Err(Failure::Guest { code, .. }) if code == "EXPECTED")
    );
    assert!(matches!(host.call(&echo, &msg("trap")), Err(Failure::Trap)));
    healthy(&host, &echo);
    let integer = register(&host, "integer");
    assert!(
        matches!(host.call(&integer, &Message { port: "input".into(), payload_json: "{\"value\":9223372036854775807}".into() }), Err(Failure::Guest { code, .. }) if code == "OVERFLOW")
    );
}
#[test]
fn abi_007_schema_validation_and_registered_ports() {
    let host = host();
    let echo = register(&host, "echo");
    for payload in ["{", "{\"text\":0}", "{\"text\":\"trap\",\"extra\":true}"] {
        assert!(matches!(
            host.call(
                &echo,
                &Message {
                    port: "input".into(),
                    payload_json: payload.into()
                }
            ),
            Err(Failure::Schema(_))
        ));
    }
    assert!(matches!(
        host.call(
            &echo,
            &Message {
                port: "undeclared".into(),
                payload_json: "{}".into()
            }
        ),
        Err(Failure::Port {
            direction: "input",
            ..
        })
    ));
    for text in ["invalid-json", "invalid-output"] {
        assert!(matches!(
            host.call(&echo, &msg(text)),
            Err(Failure::Schema(_))
        ));
    }
    assert!(matches!(
        host.call(&echo, &msg("unknown-port")),
        Err(Failure::Port {
            direction: "output",
            ..
        })
    ));
    let mut docs = schemas("echo");
    docs.insert("payload.schema.json".into(), json!({"type":"not-a-type"}));
    assert!(matches!(
        host.register(&manifest("echo"), &artifact("echo"), &docs),
        Err(Failure::Manifest(_))
    ));
    let docs = BTreeMap::from([(
        "payload.schema.json".into(),
        json!({"$ref":"https://example.invalid/never-fetch"}),
    )]);
    assert!(matches!(
        host.register(&manifest("echo"), &artifact("echo"), &docs),
        Err(Failure::Manifest(_))
    ));
    healthy(&host, &echo);
}
#[test]
fn abi_008_memory_fuel_epoch_and_oversize_recover() {
    let host = host();
    let echo = register(&host, "echo");
    for (text, budget) in [
        ("loop", "fuel"),
        ("memory", "memory/table"),
        ("many", "result count"),
        ("oversize", "result bytes"),
    ] {
        let result = host.call(&echo, &msg(text));
        assert!(
            matches!(&result, Err(Failure::Budget(b)) if *b == budget),
            "{text}: {result:?}"
        );
        healthy(&host, &echo);
    }
    assert!(matches!(
        host.call(&echo, &msg(&"x".repeat(20_000))),
        Err(Failure::Budget("message bytes"))
    ));
    let timed = Host::new(Limits {
        fuel: u64::MAX,
        epoch_ticks: 100,
        ..Limits::default()
    })
    .unwrap();
    let timed_guest = register(&timed, "echo");
    assert!(matches!(
        timed.call(&timed_guest, &msg("loop")),
        Err(Failure::Budget("epoch deadline"))
    ));
    healthy(&timed, &timed_guest);
    healthy(&host, &echo);
}
#[test]
fn abi_009_manifest_identity_and_digest() {
    let host = host();
    let bytes = artifact("echo");
    let manifest = manifest("echo");
    host.register(&manifest, &bytes, &schemas("echo")).unwrap();
    let mut tampered = bytes.clone();
    tampered[0] ^= 1;
    assert!(matches!(
        host.register(&manifest, &tampered, &schemas("echo")),
        Err(Failure::Digest)
    ));
    for (key, value) in [
        ("kind", "core-wasm"),
        ("interface", "workbench:component@0.1.0"),
    ] {
        let mut invalid = manifest.clone();
        invalid["runtime"][key] = json!(value);
        assert!(matches!(
            host.register(&invalid, &bytes, &schemas("echo")),
            Err(Failure::Manifest(_))
        ));
    }
    let mut duplicate = manifest.clone();
    duplicate["ports"]["outputs"]
        .as_array_mut()
        .unwrap()
        .push(manifest["ports"]["outputs"][0].clone());
    assert!(matches!(
        host.register(&duplicate, &bytes, &schemas("echo")),
        Err(Failure::Manifest(_))
    ));
}
#[test]
fn abi_010_reproducibility_pins_and_workflow() {
    let root = root();
    let lock = fs::read_to_string(root.join("Cargo.lock")).unwrap();
    assert!(lock.contains("name = \"wasmtime\"\nversion = \"49.0.1\""));
    let toolchain = fs::read_to_string(root.join("rust-toolchain.toml")).unwrap();
    assert!(toolchain.contains("channel = \"1.99.1\""));
    let script = fs::read_to_string(root.join("examples/wasm-components/build.sh")).unwrap();
    assert!(script.contains("cargo-component 0.21.1"));
    assert!(root.join("examples/wasm-components/Cargo.lock").exists());
    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    assert!(workflow.contains("examples/wasm-components/validate.sh"));
}

#[test]
fn cross_engine_artifact_and_invalid_configuration_are_rejected() {
    let first = host();
    let second = host();
    let guest = register(&first, "echo");
    assert!(matches!(
        second.call(&guest, &msg("hello")),
        Err(Failure::Host(_))
    ));
    assert!(matches!(
        Host::new(Limits {
            fuel: 0,
            ..Limits::default()
        }),
        Err(Failure::Host(_))
    ));
}

#[test]
fn count_caps_are_budget_failures_and_host_recovers() {
    let host = host();
    let echo = register(&host, "echo");
    use wasm_encoder::{
        Alias, Component, ComponentAliasSection, ComponentExportKind, ComponentExportSection,
        ComponentInstanceSection, InstanceSection, RawSection,
    };
    for (module, copies) in [
        ("(memory 1)", 1),
        ("(table 1 funcref) (table 1 funcref) (table 1 funcref)", 1),
        ("", 9),
    ] {
        // Re-export the compiled guest's interface without duplicating WIT types.
        let mut wrapper = Component::new();
        wrapper.section(&RawSection {
            id: 4,
            data: &artifact("echo"),
        });
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
        let core = wat::parse_str(format!("(module {module})")).unwrap();
        wrapper.section(&RawSection { id: 1, data: &core });
        let mut instances = InstanceSection::new();
        for _ in 0..copies {
            instances.instantiate(0, std::iter::empty::<(&str, wasm_encoder::ModuleArg)>());
        }
        wrapper.section(&instances);
        let bytes = wrapper.finish();
        let mut manifest = manifest("echo");
        manifest["runtime"]["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
        let registered = host.register(&manifest, &bytes, &schemas("echo")).unwrap();
        let result = host.call(&registered, &msg("hello"));
        assert!(
            matches!(&result, Err(Failure::Budget("instance resources"))),
            "{result:?}"
        );
        healthy(&host, &echo);
    }
}
