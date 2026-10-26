//! Trusted local headless host. User-selected package/source and store only.
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use wb_contracts::ipc::Endpoint;
use wb_kernel::{headless::HeadlessRoom, Kernel};
use wb_registry::{Policy, Registry};
fn absolute(path: &Path) -> std::io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.into())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
fn run() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: wb-mortgage-headless PACKAGE_DIR REGISTRY_DIR INPUT_JSON".into());
    }
    let source = absolute(Path::new(&args[0]))?;
    let store = absolute(Path::new(&args[1]))?;
    let input: serde_json::Value =
        serde_json::from_str(args[2].to_str().ok_or("invalid input encoding")?)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let python = match std::env::var_os("REGISTRY_PYTHON") {
        Some(p) => PathBuf::from(p),
        None => {
            let result = Command::new("python3")
                .args(["-c", "import sys; print(sys.executable)"])
                .output()?;
            if !result.status.success() {
                return Err("Python prerequisite unavailable".into());
            }
            PathBuf::from(String::from_utf8(result.stdout)?.trim())
        }
    };
    let worker = std::env::current_exe()?
        .parent()
        .ok_or("host executable location")?
        .join("wb-registry-inspect");
    let registry = Registry::open(
        &store,
        Policy::local(
            vec![source.parent().ok_or("source root")?.into()],
            python,
            root.join("tools/registry_validate.py"),
            worker,
        ),
    )?;
    let receipt = registry.admit_directory(&source)?;
    let handle = registry.resolve_exact(&receipt.pin)?;
    let kernel = Kernel::new(
        wb_wasm_host::Limits {
            fuel: 20_000_000,
            ..wb_wasm_host::Limits::default()
        },
        4,
        4,
    )?;
    let pin = kernel.cache_registry_release(&registry, &handle)?;
    // This explicit trusted CLI action approves this exact pure pin only, not a
    // capability grant. Library callers retain their own deny-by-default policy.
    kernel.approve_execution("mortgage-cli", "mortgage", &pin)?;
    let mut session = HeadlessRoom::start(
        &kernel,
        "mortgage-cli",
        "Headless mortgage",
        Endpoint::new("mortgage", "calculate"),
        pin,
        8,
    )?;
    let envelope = session.submit("cli-1", input, "cli-calculate")?;
    let invocation = session
        .room_mut()
        .next_invocation()?
        .ok_or("no invocation")?;
    let context = invocation.context().clone();
    let deliveries = session.room_mut().complete(invocation.run())?;
    let output = session
        .room()
        .output("mortgage", "payment")
        .filter(|v| v.valid)
        .ok_or("no committed payment")?;
    Ok(
        serde_json::json!({"pin":receipt.pin,"inputEnvelope":envelope,
        "invocationId":context.invocation_id,"revision":context.room_revision,
        "output":output.value,"valid":output.valid,"downstreamDeliveries":deliveries,
        "audit":session.room().audit()}),
    )
}
fn main() {
    match run() {
        Ok(report) => println!("{}", report),
        Err(error) => {
            let public = if let Some(e) = error.downcast_ref::<wb_contracts::ipc::HostError>() {
                serde_json::json!({"code":e.code,"source":e.source,"correlationId":e.correlation_id})
            } else if let Some(e) = error.downcast_ref::<wb_registry::RegistryError>() {
                serde_json::json!({"code":e.code,"location":e.location})
            } else {
                serde_json::json!({"code":"INVALID_INPUT"})
            };
            eprintln!("{}", serde_json::json!({"error":public}));
            std::process::exit(1);
        }
    }
}
