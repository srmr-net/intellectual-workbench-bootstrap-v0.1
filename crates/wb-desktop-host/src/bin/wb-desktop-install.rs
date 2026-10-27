//! Explicit native local admission tool, separate from installed-only UI discovery.
use std::path::PathBuf;
use wb_registry::{Policy, Registry};
fn main() {
    let a: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if a.len() < 5 {
        eprintln!("usage: wb-desktop-install REGISTRY PYTHON SEMANTIC ABI PACKAGE...");
        std::process::exit(2);
    }
    let roots = a[4..]
        .iter()
        .filter_map(|p| p.parent().map(PathBuf::from))
        .collect();
    let mut policy = Policy::local(roots, a[1].clone(), a[2].clone(), a[3].clone());
    policy.semantic_ui_v01 = true;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let registry = Registry::open(&a[0], policy)?;
        for source in &a[4..] {
            let receipt = registry.admit_directory(source)?;
            println!("{}", serde_json::to_string(&receipt)?);
        }
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("LOCAL_ADMISSION_FAILED: {error}");
        std::process::exit(1);
    }
}
