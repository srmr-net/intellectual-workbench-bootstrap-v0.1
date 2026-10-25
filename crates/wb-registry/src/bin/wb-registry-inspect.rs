//! Isolated ABI inspection. Host::register typechecks but never instantiates/calls.
use std::{collections::BTreeMap, io::Read, path::PathBuf};
use wb_wasm_host::{Host, Limits};
fn main() {
    let stage = PathBuf::from(std::env::args_os().nth(1).expect("private stage"));
    let checked: wb_semantic::CheckedMetadata =
        serde_json::from_slice(&std::fs::read(stage.join("checked.json")).unwrap()).unwrap();
    let file = std::fs::File::open(
        stage
            .join("payload")
            .join(checked.manifest["runtime"]["module"].as_str().unwrap()),
    )
    .unwrap();
    let mut bytes = Vec::new();
    file.take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .unwrap();
    let schemas: BTreeMap<_, _> = checked.schemas;
    let result = Host::new(Limits::default())
        .and_then(|host| host.register(&checked.manifest, &bytes, &schemas));
    print!(
        "{}",
        if result.is_ok() {
            "VALID"
        } else {
            "ABI_INCOMPATIBLE"
        }
    );
}
