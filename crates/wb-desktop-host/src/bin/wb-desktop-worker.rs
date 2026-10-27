//! One trusted desktop process owns one workspace writer and bounded room commands.
use serde_json::{json, Value};
use std::{
    io,
    path::PathBuf,
    sync::mpsc::{sync_channel, SyncSender, TrySendError},
    thread,
};
use wb_desktop_host::framing::{read_frame, write_frame};
use wb_desktop_host::{Command, Desktop, Request};
use wb_registry::{Policy, Registry};
use wb_storage::vs2::FileStore;
const CAPACITY: usize = 32;
type Output = (Value, Option<SyncSender<()>>);
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if a.len() != 5 {
        return Err("desktop worker arguments".into());
    }
    let mut policy = Policy::local(
        vec![a[0].parent().ok_or("registry parent")?.into()],
        a[2].clone(),
        a[3].clone(),
        a[4].clone(),
    );
    policy.semantic_ui_v01 = true;
    let registry = Registry::open(&a[0], policy)?;
    let storage = FileStore::open(&a[1]).map_err(io::Error::other)?;
    let mut desktop = Desktop::open(registry, storage);
    let (out_tx, out_rx) = sync_channel::<Output>(CAPACITY);
    let writer = thread::spawn(move || {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for (value, ack) in out_rx {
            let bytes = serde_json::to_vec(&value).unwrap();
            if write_frame(&mut out, &bytes).is_err() {
                break;
            }
            if let Some(ack) = ack {
                let _ = ack.send(());
            }
        }
    });
    out_tx.send((desktop.state("", true, vec![]), None))?;
    let (input_tx, input_rx) = sync_channel::<Request>(CAPACITY);
    let errors = out_tx.clone();
    let reader = thread::spawn(move || {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        loop {
            let frame = match read_frame(&mut input) {
                Ok(Some(f)) => f,
                Ok(None) => break,
                Err(_) => {
                    let _=errors.send((json!({"protocol":"desktop.1","kind":"transportError","code":"INVALID_UI_FRAME"}),None));
                    break;
                }
            };
            let request = wb_contracts::ui::strict_json(&frame)
                .ok()
                .and_then(|v| serde_json::from_value::<Request>(v).ok());
            let Some(request) = request else {
                if errors.send((json!({"protocol":"desktop.1","kind":"transportError","code":"INVALID_UI_REQUEST"}),None)).is_err(){break;}
                continue;
            };
            let token: String = request.token.chars().take(64).collect();
            match input_tx.try_send(request) {
                Ok(())=>{},Err(TrySendError::Full(_))=>{if errors.send((json!({"protocol":"desktop.1","kind":"backpressure","token":token,"code":"QUEUE_FULL"}),None)).is_err(){break;}},Err(TrySendError::Disconnected(_))=>break,
            }
        }
    });
    for request in input_rx {
        let wants_quit = matches!(&request.command, Command::Quit);
        let reply = desktop.request(request);
        let stop = wants_quit && reply["accepted"] == true;
        let (ack_tx, ack_rx) = sync_channel(1);
        out_tx.send((reply, stop.then_some(ack_tx)))?;
        if stop {
            ack_rx.recv()?;
            return Ok(());
        }
    }
    // Every acknowledged transition was already fsynced. No EOF auto-repair or
    // overwrite of corrupt/recovery files; reader EOF drops its output sender.
    let _ = reader.join();
    drop(out_tx);
    let _ = writer.join();
    Ok(())
}
fn main() {
    // Trusted native child dies if its desktop owner vanishes. No guest imports.
    let parent = unsafe { libc::getppid() };
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) } != 0
        || unsafe { libc::getppid() } != parent
    {
        std::process::exit(1);
    }
    if run().is_err() {
        let _ = write_frame(
            io::stdout(),
            br#"{"protocol":"desktop.1","kind":"fatal","code":"WORKSPACE_UNAVAILABLE"}"#,
        );
        std::process::exit(1);
    }
}
