//! VSLICE003: append one fresh empty room to a validated existing workspace.
//! Trusted desktop-compatible worker. Ordinary edit/restore delegates unchanged
//! Desktop; a separate bounded operator.1 channel carries explicit user intents.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{sync_channel, SyncSender, TrySendError},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use wb_contracts::{ui::strict_json, ComponentRef};
use wb_desktop_host::{
    framing::{read_frame, write_frame},
    Command, Desktop, Request,
};
use wb_operator::{
    catalog::Selection,
    proposal::ProposalError,
    provider::{Cancellation, CountedProvider, DisabledProvider, OfflineFixture, OperatorProvider},
    session::Session,
};
use wb_registry::{Policy, Registry};
use wb_storage::vs2::{Checkpoint, FileStore, UiPin, MAX_ROOMS};

const CAPACITY: usize = 32;
const MAX_OPERATOR_FRAME: usize = 64 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperatorRequest {
    protocol: String,
    token: String,
    command: OperatorCommand,
    #[serde(skip)]
    ticket: Option<Cancellation>,
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum OperatorCommand {
    Prepare { utterance: String },
    Approve { digest: String },
    Decline,
    Cancel,
    Status,
}
enum Input {
    Desktop(Request),
    Operator(OperatorRequest),
}
struct Output {
    operator: bool,
    value: Value,
    ack: Option<SyncSender<()>>,
}
type PendingPrepare = Arc<Mutex<Option<(String, Cancellation)>>>;
struct PrepareLease {
    active: PendingPrepare,
    ticket: Cancellation,
}
impl Drop for PrepareLease {
    fn drop(&mut self) {
        let mut active = self.active.lock().unwrap();
        if active
            .as_ref()
            .is_some_and(|(_, ticket)| ticket.same_request(&self.ticket))
        {
            active.take();
        }
    }
}
fn release_rejected_prepare(active: &PendingPrepare, rejected: Input) {
    if let Input::Operator(OperatorRequest {
        command: OperatorCommand::Prepare { .. },
        ticket: Some(ticket),
        ..
    }) = rejected
    {
        let mut pending = active.lock().unwrap();
        if pending
            .as_ref()
            .is_some_and(|(_, cancel)| cancel.same_request(&ticket))
        {
            pending.take();
        }
    }
}
type Controller = Arc<Mutex<Option<Arc<Session>>>>;
fn op_reply(token: &str, ok: bool, code: &str, calls: u64) -> Value {
    json!({"protocol":"operator.1","kind":"reply","token":token,"ok":ok,"code":code,"providerCalls":calls.to_string()})
}
fn bounded_operator_bytes(value: &Value, calls: u64) -> Vec<u8> {
    let bytes = serde_json::to_vec(value).unwrap();
    if bytes.len() <= MAX_OPERATOR_FRAME {
        return bytes;
    }
    let mut compact = value.clone();
    if let Some(object) = compact.as_object_mut() {
        object.remove("audit");
        object.remove("nativeAudit");
    }
    let bytes = serde_json::to_vec(&compact).unwrap();
    if bytes.len() <= MAX_OPERATOR_FRAME {
        return bytes;
    }
    let mut reply = op_reply(
        value["token"].as_str().unwrap_or(""),
        false,
        "QUEUE_FULL",
        calls,
    );
    for key in [
        "topologyCommitted",
        "topologyCheckpointPersisted",
        "initializationSucceeded",
        "checkpointPersisted",
        "refreshDesktop",
        "roomId",
        "roomRevision",
        "nativeCode",
        "storageCode",
    ] {
        if value[key].is_boolean() || value[key].as_str().is_some_and(|s| s.len() <= 256) {
            reply[key] = value[key].clone();
        }
    }
    serde_json::to_vec(&reply).unwrap()
}
fn send_operator(tx: &SyncSender<Output>, value: Value) {
    let _ = tx.send(Output {
        operator: true,
        value,
        ack: None,
    });
}
fn wire_audit(session: &Session) -> Vec<wb_operator::session::Audit> {
    session
        .audit()
        .into_iter()
        .rev()
        .take(16)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
fn send_operator_ack(
    tx: &SyncSender<Output>,
    value: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let (ack_tx, ack_rx) = sync_channel(1);
    tx.send(Output {
        operator: true,
        value,
        ack: Some(ack_tx),
    })?;
    ack_rx.recv_timeout(Duration::from_secs(2))?;
    Ok(())
}
fn checkpoint(
    room: wb_contracts::ipc::RoomSnapshot,
    selection: &Selection,
    outputs: std::collections::BTreeMap<String, Value>,
) -> Checkpoint {
    Checkpoint {
        room_id: room.room_id.clone(),
        label: room.name.clone(),
        pinned: false,
        component: ComponentRef {
            id: selection.pin.id.clone(),
            version: selection.pin.version.clone(),
            sha256: selection.pin.sha256.clone(),
        },
        ui: UiPin {
            path: selection.ui_path.clone(),
            sha256: selection.ui_sha256.clone(),
            schema_sha256: selection.ui_schema_sha256.clone(),
        },
        room,
        outputs,
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.len() != 5 {
        return Err("worker arguments".into());
    }
    let mut policy = Policy::local(
        vec![args[0].parent().ok_or("registry parent")?.into()],
        args[2].clone(),
        args[3].clone(),
        args[4].clone(),
    );
    policy.semantic_ui_v01 = true;
    let registry = Registry::open(&args[0], policy)?;
    let storage = FileStore::open(&args[1]).map_err(io::Error::other)?;
    // Cold restore creates no Session, Kernel or provider. Accepted Desktop owns
    // validation, exact pins and zero-compute display of saved committed state.
    let mut desktop = Some(Desktop::open(registry.clone(), storage));
    let controller: Controller = Arc::default();
    let target_room = Arc::new(Mutex::new(String::new()));
    let mut prepared_workspace = None;
    let mut epoch_offset = 0_u64;
    let mut visible_epoch = 1_u64;
    let mut handoff_from = None;
    let pending_prepare: PendingPrepare = Arc::default();
    let total_calls = Arc::new(AtomicU64::new(0));
    let (out_tx, out_rx) = sync_channel::<Output>(CAPACITY);
    let writer_calls = total_calls.clone();
    let writer = thread::spawn(move || {
        let stdout = io::stdout();
        let stderr = io::stderr();
        for out in out_rx {
            let bytes = if out.operator {
                bounded_operator_bytes(&out.value, writer_calls.load(Ordering::SeqCst))
            } else {
                serde_json::to_vec(&out.value).unwrap()
            };
            let result = if out.operator {
                write_frame(stderr.lock(), &bytes)
            } else {
                write_frame(stdout.lock(), &bytes)
            };
            if result.is_err() {
                break;
            }
            if let Some(ack) = out.ack {
                let _ = ack.send(());
            }
        }
    });
    out_tx.send(Output {
        operator: false,
        value: desktop.as_mut().unwrap().state("", true, vec![]),
        ack: None,
    })?;
    let (in_tx, in_rx) = sync_channel::<Input>(CAPACITY);
    let errors = out_tx.clone();
    let active = controller.clone();
    let reader_room = target_room.clone();
    let queued = pending_prepare.clone();
    let calls = total_calls.clone();
    let reader = thread::spawn(move || {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        loop {
            let frame = match read_frame(&mut input) {
                Ok(Some(f)) => f,
                Ok(None) => break,
                Err(_) => {
                    let _=errors.send(Output { operator:false,value:json!({"protocol":"desktop.1","kind":"transportError","code":"INVALID_UI_FRAME"}),ack:None });
                    break;
                }
            };
            let Ok(value) = strict_json(&frame) else {
                send_operator(&errors, op_reply("", false, "UNSUPPORTED_REQUEST", 0));
                continue;
            };
            let parsed = if value["protocol"] == "operator.1" {
                if frame.len() > 20 * 1024 {
                    send_operator(&errors, op_reply("", false, "UNSUPPORTED_REQUEST", 0));
                    continue;
                }
                match serde_json::from_value::<OperatorRequest>(value) {
                    Ok(mut r)
                        if r.protocol == "operator.1"
                            && !r.token.is_empty()
                            && r.token.len() <= 64
                            && r.token
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'-') =>
                    {
                        if matches!(r.command, OperatorCommand::Cancel) {
                            let ticket = queued.lock().unwrap().as_ref().map(|(_, c)| c.clone());
                            if let Some(cancel) = &ticket {
                                cancel.cancel();
                            }
                            let session = active.lock().unwrap().clone();
                            let cancelled = session
                                .as_ref()
                                .is_some_and(|s| s.cancel(&reader_room.lock().unwrap()))
                                || ticket.is_some();
                            let observed_calls = calls.load(Ordering::SeqCst);
                            send_operator(
                                &errors,
                                op_reply(
                                    &r.token,
                                    cancelled,
                                    if cancelled {
                                        "CANCELLED"
                                    } else {
                                        "APPROVAL_REQUIRED"
                                    },
                                    observed_calls,
                                ),
                            );
                            continue;
                        }
                        if matches!(r.command, OperatorCommand::Prepare { .. }) {
                            let mut pending = queued.lock().unwrap();
                            if pending.is_some() {
                                send_operator(
                                    &errors,
                                    op_reply(
                                        &r.token,
                                        false,
                                        "QUEUE_FULL",
                                        calls.load(Ordering::SeqCst),
                                    ),
                                );
                                continue;
                            }
                            let ticket = Cancellation::default();
                            *pending = Some((r.token.clone(), ticket.clone()));
                            r.ticket = Some(ticket);
                        }
                        Input::Operator(r)
                    }
                    _ => {
                        send_operator(&errors, op_reply("", false, "UNSUPPORTED_REQUEST", 0));
                        continue;
                    }
                }
            } else {
                match serde_json::from_value::<Request>(value) {
                    Ok(r) => Input::Desktop(r),
                    Err(_) => {
                        let _=errors.send(Output { operator:false,value:json!({"protocol":"desktop.1","kind":"transportError","code":"INVALID_UI_REQUEST"}),ack:None });
                        continue;
                    }
                }
            };
            let (operator, token) = match &parsed {
                Input::Operator(r) => (true, r.token.clone()),
                Input::Desktop(r) => (false, r.token.chars().take(64).collect()),
            };
            match in_tx.try_send(parsed) {
                Ok(()) => {}
                Err(TrySendError::Full(rejected)) => {
                    release_rejected_prepare(&queued, rejected);
                    if operator {
                        send_operator(&errors, op_reply(&token, false, "QUEUE_FULL", 0));
                    } else {
                        let _=errors.send(Output { operator:false,value:json!({"protocol":"desktop.1","kind":"backpressure","token":token,"code":"QUEUE_FULL"}),ack:None });
                    }
                }
                Err(TrySendError::Disconnected(_)) => break,
            }
        }
    });
    for input in in_rx {
        let mut room_id = target_room.lock().unwrap().clone();
        match input {
            Input::Desktop(mut request) => {
                if matches!(request.command, Command::Create { .. } | Command::Quit) {
                    if let Some(session) = controller.lock().unwrap().take() {
                        session.cancel(&room_id);
                    }
                }
                // Desktop is rematerialized after durable handoff. Keep its private
                // counter separate from the monotonic counter already seen by Qt.
                let wire_epoch = request.epoch.parse::<u64>().ok();
                let resync = handoff_from.is_some()
                    && wire_epoch == handoff_from
                    && matches!(request.command, Command::Select { .. } | Command::Quit);
                let internal_epoch = desktop.as_mut().ok_or("desktop unavailable")?.state(
                    "counter.inspection",
                    true,
                    vec![],
                )["epoch"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or("invalid counter")?;
                request.epoch = if resync {
                    internal_epoch
                } else {
                    wire_epoch
                        .and_then(|n| n.checked_sub(epoch_offset))
                        .unwrap_or(u64::MAX)
                }
                .to_string();
                if let Command::Ui { request: ui } = &mut request.command {
                    ui.expected_generation = ui
                        .expected_generation
                        .checked_sub(epoch_offset)
                        .unwrap_or(u64::MAX);
                }
                let selection = matches!(request.command, Command::Select { .. });
                let quit = matches!(request.command, Command::Quit);
                let mut reply = desktop
                    .as_mut()
                    .ok_or("desktop unavailable")?
                    .request(request);
                if selection && reply["accepted"] == true {
                    handoff_from = None;
                }
                let internal = reply["epoch"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or("invalid counter")?;
                visible_epoch = internal
                    .checked_add(epoch_offset)
                    .ok_or("counter exhausted")?;
                reply["epoch"] = json!(visible_epoch.to_string());
                if reply["projection"].is_object() {
                    // Accepted Desktop projects its view epoch as executionGeneration.
                    reply["projection"]["executionGeneration"] = json!(visible_epoch.to_string());
                }
                let stop = quit && reply["accepted"] == true;
                let (ack_tx, ack_rx) = sync_channel(1);
                out_tx.send(Output {
                    operator: false,
                    value: reply,
                    ack: stop.then_some(ack_tx),
                })?;
                if stop {
                    ack_rx.recv()?;
                    return Ok(());
                }
            }
            Input::Operator(request) => {
                let provider_calls = total_calls.load(Ordering::SeqCst);
                match request.command {
                    OperatorCommand::Prepare { utterance } => {
                        let cancel = request.ticket.ok_or("missing prepare lease")?;
                        let _lease = PrepareLease {
                            active: pending_prepare.clone(),
                            ticket: cancel.clone(),
                        };
                        if cancel.is_cancelled() {
                            send_operator(
                                &out_tx,
                                op_reply(&request.token, false, "CANCELLED", provider_calls),
                            );
                            continue;
                        }
                        let d = desktop.as_mut().ok_or("desktop unavailable")?;
                        let state = d.state("operator.inspection", true, vec![]);
                        if d.workspace().rooms.len() >= MAX_ROOMS || !state["recovery"].is_null() {
                            send_operator(
                                &out_tx,
                                op_reply(
                                    &request.token,
                                    false,
                                    "UNSUPPORTED_REQUEST",
                                    provider_calls,
                                ),
                            );
                            continue;
                        }
                        if prepared_workspace.as_ref()
                            != Some(&(d.workspace().revision, d.workspace().content_sha256.clone()))
                        {
                            if let Some(old) = controller.lock().unwrap().take() {
                                old.cancel(&room_id);
                            }
                        }
                        if controller.lock().unwrap().is_none() {
                            room_id = (1..=MAX_ROOMS + 1)
                                .map(|i| {
                                    if i == 1 {
                                        "intent-room".to_string()
                                    } else {
                                        format!("intent-room-{i}")
                                    }
                                })
                                .find(|id| !d.workspace().rooms.iter().any(|r| &r.room_id == id))
                                .ok_or("room quota")?;
                            *target_room.lock().unwrap() = room_id.clone();
                            prepared_workspace = Some((
                                d.workspace().revision,
                                d.workspace().content_sha256.clone(),
                            ));
                            let provider: Arc<dyn OperatorProvider> =
                                if std::env::var("WB_OPERATOR_PROVIDER").as_deref()
                                    == Ok("disabled")
                                {
                                    Arc::new(DisabledProvider)
                                } else {
                                    Arc::new(OfflineFixture)
                                };
                            let session = Arc::new(
                                Session::new(
                                    registry.clone(),
                                    Arc::new(CountedProvider {
                                        inner: provider,
                                        calls: total_calls.clone(),
                                    }),
                                    wb_wasm_host::Limits {
                                        fuel: 20_000_000,
                                        ..Default::default()
                                    },
                                )
                                .map_err(|e| io::Error::other(e.code()))?,
                            );
                            session
                                .create_room(&room_id, "Intent room")
                                .map_err(|e| io::Error::other(e.code()))?;
                            *controller.lock().unwrap() = Some(session);
                        }
                        let session = controller.lock().unwrap().clone().unwrap();
                        let result = session.prepare_with_cancel(
                            &room_id,
                            &utterance,
                            Duration::from_secs(30),
                            cancel,
                        );
                        let provider_calls = total_calls.load(Ordering::SeqCst);
                        let mut reply = op_reply(
                            &request.token,
                            result.is_ok(),
                            result.as_ref().err().map_or("PREVIEW", |e| e.code()),
                            provider_calls,
                        );
                        if let Err(error) = &result {
                            if let Some(native) = error.native_code() {
                                reply["nativeCode"] = serde_json::to_value(native)?;
                            }
                        }
                        if let Ok(preview) = result {
                            reply["preview"] = serde_json::to_value(preview)?;
                        }
                        reply["audit"] = serde_json::to_value(wire_audit(&session))?;
                        send_operator(&out_tx, reply);
                    }
                    OperatorCommand::Decline => {
                        let session = controller.lock().unwrap().clone();
                        let ok = session.is_some_and(|s| s.decline(&room_id).is_ok());
                        send_operator(
                            &out_tx,
                            op_reply(
                                &request.token,
                                ok,
                                if ok { "DECLINED" } else { "APPROVAL_REQUIRED" },
                                provider_calls,
                            ),
                        );
                    }
                    OperatorCommand::Approve { digest } => {
                        if digest.len() != 64
                            || !digest
                                .bytes()
                                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                        {
                            if let Some(session) = controller.lock().unwrap().clone() {
                                session.cancel(&room_id);
                            }
                            send_operator(
                                &out_tx,
                                op_reply(
                                    &request.token,
                                    false,
                                    "APPROVAL_REJECTED",
                                    provider_calls,
                                ),
                            );
                            continue;
                        }
                        let Some(session) = controller.lock().unwrap().clone() else {
                            send_operator(
                                &out_tx,
                                op_reply(
                                    &request.token,
                                    false,
                                    "APPROVAL_REQUIRED",
                                    provider_calls,
                                ),
                            );
                            continue;
                        };
                        let current = desktop
                            .as_ref()
                            .ok_or("desktop unavailable")?
                            .workspace()
                            .clone();
                        if prepared_workspace.as_ref()
                            != Some(&(current.revision, current.content_sha256.clone()))
                        {
                            session.cancel(&room_id);
                            send_operator(
                                &out_tx,
                                op_reply(&request.token, false, "STATE_CONFLICT", provider_calls),
                            );
                            continue;
                        }
                        // FileStore cannot be extracted from accepted Desktop. Release,
                        // reacquire, then revalidate CAS before any topology mutation.
                        drop(desktop.take());
                        let store = match FileStore::open(&args[1]) {
                            Ok(s) => s,
                            Err(native) => {
                                session.cancel(&room_id);
                                let mut reply = op_reply(
                                    &request.token,
                                    false,
                                    "STATE_CONFLICT",
                                    provider_calls,
                                );
                                reply["nativeCode"] = json!(native);
                                reply["topologyCommitted"] = json!(false);
                                reply["initializationSucceeded"] = json!(false);
                                reply["checkpointPersisted"] = json!(false);
                                send_operator_ack(&out_tx, reply)?;
                                return Err("workspace handoff busy".into());
                            }
                        };
                        let loaded = match store.load() {
                            Ok(workspace) => workspace.unwrap_or_default(),
                            Err(native) => {
                                session.cancel(&room_id);
                                desktop = Some(Desktop::open(registry.clone(), store));
                                epoch_offset = visible_epoch;
                                handoff_from = Some(visible_epoch);
                                *controller.lock().unwrap() = None;
                                let mut reply = op_reply(
                                    &request.token,
                                    false,
                                    "STATE_CONFLICT",
                                    provider_calls,
                                );
                                reply["nativeCode"] = json!(native);
                                reply["topologyCommitted"] = json!(false);
                                reply["refreshDesktop"] = json!(true);
                                send_operator(&out_tx, reply);
                                continue;
                            }
                        };
                        if loaded.revision != current.revision
                            || loaded.content_sha256 != current.content_sha256
                            || loaded.rooms.iter().any(|r| r.room_id == room_id)
                            || loaded.rooms.len() >= MAX_ROOMS
                        {
                            session.cancel(&room_id);
                            desktop = Some(Desktop::open(registry.clone(), store));
                            epoch_offset = visible_epoch;
                            handoff_from = Some(visible_epoch);
                            send_operator(
                                &out_tx,
                                op_reply(&request.token, false, "STATE_CONFLICT", provider_calls),
                            );
                            continue;
                        }
                        let mut workspace = loaded;
                        let mut selected = None;
                        let mut persistence_native: Option<String> = None;
                        // Callback is trusted bounded FileStore I/O only. It must not
                        // reenter Session while approval holds its control-plane lock.
                        let result = session.approve(&room_id, &digest, |room, selection| {
                            let cp = checkpoint(room.snapshot(), selection, Default::default());
                            let mut candidate = workspace.clone();
                            candidate.rooms.push(cp);
                            candidate.selected_room = Some(room_id.clone());
                            workspace =
                                store.commit(workspace.revision, &candidate).map_err(|e| {
                                    persistence_native = Some(e.into());
                                    ProposalError::Persistence
                                })?;
                            selected = Some(selection.clone());
                            Ok(())
                        });
                        let mut persisted = false;
                        let mut persistence_error = false;
                        if let (Ok(outcome), Some(selection)) = (&result, selected) {
                            if outcome.initialization_succeeded {
                                match (session.snapshot(&room_id), session.outputs(&room_id)) {
                                    (Ok(room), Ok(outputs)) => {
                                        let mut candidate = workspace.clone();
                                        let cp = checkpoint(room, &selection, outputs);
                                        let target = candidate
                                            .rooms
                                            .iter_mut()
                                            .find(|r| r.room_id == room_id)
                                            .ok_or("missing committed room")?;
                                        *target = cp;
                                        match store.commit(workspace.revision, &candidate) {
                                            Ok(saved) => {
                                                workspace = saved;
                                                persisted = true
                                            }
                                            Err(e) => {
                                                persistence_native = Some(e.into());
                                                persistence_error = true
                                            }
                                        }
                                    }
                                    _ => persistence_error = true,
                                }
                            }
                        }
                        let mut reply = op_reply(
                            &request.token,
                            result.as_ref().is_ok_and(|o| o.initialization_succeeded)
                                && !persistence_error,
                            result.as_ref().err().map_or(
                                if persistence_error {
                                    "INGRESS_FAILED"
                                } else {
                                    "APPLIED"
                                },
                                |e| e.code(),
                            ),
                            provider_calls,
                        );
                        if let Err(error) = &result {
                            if let Some(native) = error.native_code() {
                                reply["nativeCode"] = serde_json::to_value(native)?;
                            }
                        }
                        if let Some(native) = persistence_native {
                            reply["storageCode"] = json!(native);
                        }
                        reply["refreshDesktop"] = json!(true);
                        reply["topologyCheckpointPersisted"] =
                            json!(workspace.rooms.iter().any(|r| r.room_id == room_id));
                        if let Ok(outcome) = result {
                            reply["topologyCommitted"] = json!(outcome.topology_committed);
                            reply["initializationSucceeded"] =
                                json!(outcome.initialization_succeeded);
                            reply["checkpointPersisted"] = json!(persisted);
                            reply["roomId"] = json!(&room_id);
                            reply["roomRevision"] = json!(workspace
                                .rooms
                                .iter()
                                .find(|r| r.room_id == room_id)
                                .map(|r| r.room.revision.to_string()));
                            if let Some(e) = outcome.initialization_error {
                                reply["code"] = json!("INGRESS_FAILED");
                                reply["nativeCode"] = serde_json::to_value(e.code)?;
                            }
                        }
                        let native_audit = session
                            .execution_audit(&room_id)
                            .map_err(|e| io::Error::other(e.code()))?;
                        reply["initialInvocations"] = json!(native_audit
                            .iter()
                            .filter(|e| e.kind == "invocation.started")
                            .count()
                            .to_string());
                        reply["nativeAudit"] = serde_json::to_value(native_audit)?;
                        reply["audit"] = serde_json::to_value(wire_audit(&session))?;
                        desktop = Some(Desktop::open(registry.clone(), store));
                        epoch_offset = visible_epoch;
                        handoff_from = Some(visible_epoch);
                        *controller.lock().unwrap() = None;
                        send_operator(&out_tx, reply);
                    }
                    OperatorCommand::Status => {
                        send_operator(
                            &out_tx,
                            op_reply(&request.token, true, "READY", provider_calls),
                        );
                    }
                    OperatorCommand::Cancel => unreachable!("reader handles cancellation"),
                }
            }
        }
    }
    let _ = reader.join();
    drop(out_tx);
    let _ = writer.join();
    Ok(())
}
fn main() {
    let parent = unsafe { libc::getppid() };
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) } != 0
        || unsafe { libc::getppid() } != parent
    {
        std::process::exit(1)
    }
    if run().is_err() {
        let _ = write_frame(
            io::stdout(),
            br#"{"protocol":"desktop.1","kind":"fatal","code":"WORKSPACE_UNAVAILABLE"}"#,
        );
        std::process::exit(1)
    }
}
