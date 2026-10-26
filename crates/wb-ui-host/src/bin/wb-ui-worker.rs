//! Trusted native presentation worker. Original kernel/bus remain process-local.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::mpsc::{sync_channel, TrySendError},
    thread,
};
use wb_contracts::{
    ipc::RoomSnapshot,
    ui::{strict_json, Command, Request},
};
use wb_kernel::Kernel;
use wb_registry::{Policy, Registry};
use wb_ui_host::{
    framing::{read_frame, write_frame, DEFAULT_CAPACITY},
    selected_pin, Session,
};
struct SnapshotStore {
    dir: File,
    name: String,
}
impl SnapshotStore {
    fn open(
        store: &Path,
        room: &str,
        registry: &Registry,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use std::os::fd::AsRawFd;
        let parent = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open(store)?;
        let metadata = parent.metadata()?;
        if (metadata.dev(), metadata.ino()) != registry.identity()? {
            return Err("registry storage changed".into());
        }
        let anchored =
            PathBuf::from(format!("/proc/self/fd/{}", parent.as_raw_fd())).join("ui-snapshots");
        match DirBuilder::new().mode(0o700).create(&anchored) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.into()),
        }
        let dir = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open(anchored)?;
        let m = dir.metadata()?;
        // Same-user private native storage, no authority derived from package paths.
        if m.mode() & 0o077 != 0 || m.uid() != unsafe { libc::geteuid() } {
            return Err("invalid snapshot store".into());
        }
        Ok(Self {
            dir,
            name: format!("{:x}.json", Sha256::digest(room.as_bytes())),
        })
    }
    fn path(&self) -> PathBuf {
        use std::os::fd::AsRawFd;
        PathBuf::from(format!("/proc/self/fd/{}", self.dir.as_raw_fd())).join(&self.name)
    }
    fn save(&self, snapshot: &RoomSnapshot) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = serde_json::to_vec(snapshot)?;
        if bytes.len() > 1024 * 1024 {
            return Err("snapshot quota".into());
        }
        let path = self.path();
        let temp = path.with_extension(format!("{}.tmp", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temp)?;
        let result = (|| -> io::Result<()> {
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temp, path)?;
            self.dir.sync_all()
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result?;
        Ok(())
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 7 {
        return Err("worker arguments".into());
    }
    let source = PathBuf::from(&args[0]);
    let store = PathBuf::from(&args[1]);
    let python = PathBuf::from(&args[2]);
    let semantic = PathBuf::from(&args[3]);
    let abi = PathBuf::from(&args[4]);
    let room = args[5].to_str().ok_or("room")?;
    let restore = args[6] == "restore";
    let mut policy = Policy::local(
        vec![source.parent().ok_or("source")?.into()],
        python,
        semantic,
        abi,
    );
    policy.semantic_ui_v01 = true;
    let registry = Registry::open(&store, policy)?;
    let receipt = registry.admit_directory(&source)?;
    let handle = registry.resolve_exact(&receipt.pin)?;
    let kernel = Kernel::new(
        wb_wasm_host::Limits {
            fuel: 20_000_000,
            ..Default::default()
        },
        8,
        8,
    )?;
    let pin = selected_pin(&registry, &handle, &kernel).map_err(io::Error::other)?;
    // Explicit native host startup approves only this selected exact pure pin.
    kernel.approve_execution(room, "component", &pin)?;
    let snapshot = SnapshotStore::open(&store, room, &registry)?;
    let mut session = if restore {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(snapshot.path())?;
        if !file.metadata()?.is_file()
            || file.metadata()?.nlink() != 1
            || file.metadata()?.len() > 1024 * 1024
        {
            return Err("snapshot quota".into());
        }
        use std::io::Read;
        let mut bytes = vec![];
        file.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 1024 * 1024 {
            return Err("snapshot quota".into());
        }
        let state: RoomSnapshot = serde_json::from_value(strict_json(&bytes)?)?;
        Session::restore(&kernel, &registry, &handle, state, room, "component")
            .map_err(io::Error::other)?
    } else {
        Session::start(&kernel, &registry, &handle, room, "component").map_err(io::Error::other)?
    };
    let (output_tx, output_rx) =
        sync_channel::<(Value, Option<std::sync::mpsc::SyncSender<()>>)>(DEFAULT_CAPACITY);
    let writer = thread::spawn(move || {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for (reply, ack) in output_rx {
            let bytes = serde_json::to_vec(&reply).unwrap();
            if write_frame(&mut out, &bytes).is_err() {
                break;
            }
            if let Some(ack) = ack {
                let _ = ack.send(());
            }
        }
    });
    output_tx.send((serde_json::to_value(session.ready())?, None))?;
    let (input_tx, input_rx) = sync_channel::<Request>(DEFAULT_CAPACITY);
    let reader_out = output_tx.clone();
    let reader = thread::spawn(move || {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        loop {
            let raw = match read_frame(&mut input) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => break,
                Err(_) => {
                    let _ = reader_out.send((
                        json!({"protocol":"0.1","kind":"transportError","code":"INVALID_UI_FRAME"}),
                        None,
                    ));
                    break;
                }
            };
            let parsed = strict_json(&raw)
                .ok()
                .and_then(|value| serde_json::from_value::<Request>(value).ok());
            let Some(request) = parsed else {
                if reader_out.send((json!({"protocol":"0.1","kind":"transportError","code":"INVALID_UI_REQUEST"}), None)).is_err(){break;}
                continue;
            };
            let token = request.local_token.clone();
            match input_tx.try_send(request) {
            Ok(())=>{},
            Err(TrySendError::Full(_))=>{if reader_out.send((json!({"protocol":"0.1","kind":"backpressure","localToken":token,"code":"QUEUE_FULL"}), None)).is_err(){break;}},
            Err(TrySendError::Disconnected(_))=>break,
        }
        }
    });
    for request in input_rx {
        let explicit_save = matches!(request.command, Command::Save | Command::Shutdown);
        let should_save = explicit_save
            || matches!(
                request.command,
                Command::Edit { .. } | Command::Apply { .. }
            );
        let stop = matches!(request.command, Command::Shutdown);
        let mut reply = session.request(request);
        if should_save && reply.accepted {
            match snapshot.save(&session.snapshot()) {
                Ok(()) => {
                    if explicit_save {
                        reply.kind = "saved".into();
                    }
                }
                Err(_) => {
                    reply = session.rejected(
                        &reply.local_token,
                        wb_contracts::ui::Diagnostic::new("SNAPSHOT_IO", None),
                    );
                }
            }
        }
        let terminate = stop && reply.accepted;
        let (ack_tx, ack_rx) = sync_channel(1);
        if output_tx
            .send((serde_json::to_value(reply)?, terminate.then_some(ack_tx)))
            .is_err()
        {
            break;
        }
        if terminate {
            // Ensure the final committed reply is flushed before process exit.
            ack_rx.recv()?;
            // The reader may be blocked on trusted client stdin. Do not join it:
            // returning from main terminates this process and its detached reader.
            return Ok(());
        }
    }
    let _ = snapshot.save(&session.snapshot());
    drop(output_tx);
    let _ = reader.join();
    let _ = writer.join();
    Ok(())
}
fn main() {
    if run().is_err() {
        let _ = write_frame(
            io::stdout(),
            br#"{"protocol":"0.1","kind":"unavailable","code":"UI_UNAVAILABLE"}"#,
        );
        eprintln!("UI_WORKER_UNAVAILABLE");
        std::process::exit(1);
    }
}
