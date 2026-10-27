//! VSLICE002 desktop: validated durable display checkpoints, real accepted kernel.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, io::Read};
use wb_contracts::{
    ipc::{CompositionPlan, Endpoint, InstanceSpec, RoomSnapshot, UiReference},
    ui::{
        Command as UiCommand, ControlKind, Diagnostic, Direction, Projection, Request as UiRequest,
        UiKey,
    },
    ComponentRef,
};
use wb_kernel::{
    room::{Room, UiAdapter},
    Kernel,
};
use wb_registry::{Registry, ReleasePin, ValidatedHandle};
use wb_semantic::{ui::ValidatedUi, CheckedMetadata};
use wb_storage::vs2::{Checkpoint, FileStore, UiPin, Workspace, MAX_ROOMS};
pub const CONTRACT_VERSION: &str = "0.1";
pub mod framing;
type Result<T> = std::result::Result<T, String>;
fn code(e: wb_contracts::ipc::HostError) -> String {
    serde_json::to_value(e.code)
        .unwrap()
        .as_str()
        .unwrap()
        .into()
}
fn release(pin: &ComponentRef) -> ReleasePin {
    ReleasePin {
        id: pin.id.clone(),
        version: pin.version.clone(),
        sha256: pin.sha256.clone(),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Installed {
    pub pin: ComponentRef,
    pub label: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub protocol: String,
    pub token: String,
    pub epoch: String,
    pub command: Command,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Command {
    Create { pin: ComponentRef },
    Select { room_id: String },
    Pin { pinned: bool },
    Ui { request: UiRequest },
    Quit,
}
struct Runtime {
    room: Room,
    adapter: UiAdapter,
    reported_invocations: u64,
}
struct View {
    ui: ValidatedUi,
    checked: CheckedMetadata,
    handle: ValidatedHandle,
    runtime: Option<Runtime>,
    drafts: BTreeMap<String, Value>,
}
/// Host-owned workspace. Recovery is latched/read-only, never auto-repaired.
pub struct Desktop {
    registry: Registry,
    store: FileStore,
    workspace: Workspace,
    installed: Vec<Installed>,
    views: BTreeMap<String, View>,
    recovery: Option<String>,
    epoch: u64,
    sequence: u64,
    invocations: u64,
    next_attempt: u64,
    trace: Vec<Value>,
}
impl Desktop {
    pub fn open(registry: Registry, store: FileStore) -> Self {
        let mut result = Self {
            registry,
            store,
            workspace: Workspace::default(),
            installed: vec![],
            views: BTreeMap::new(),
            recovery: None,
            epoch: 1,
            sequence: 0,
            invocations: 0,
            next_attempt: 0,
            trace: vec![],
        };
        match result.store.load() {
            Ok(Some(workspace)) => result.workspace = workspace,
            Ok(None) => {}
            Err(e) => result.recovery = Some(e.into()),
        }
        if result.recovery.is_none() {
            let checkpoints = result.workspace.rooms.clone();
            for checkpoint in &checkpoints {
                match result.read_view(checkpoint) {
                    Ok(view) => {
                        result.views.insert(checkpoint.room_id.clone(), view);
                    }
                    Err(e) => {
                        result.recovery = Some(e);
                        result.views.clear();
                        break;
                    }
                }
            }
        }
        if result.recovery.is_none() {
            match result.discover() {
                Ok(v) => result.installed = v,
                Err(e) => result.recovery = Some(e),
            }
        }
        result
    }
    fn discover(&self) -> Result<Vec<Installed>> {
        let mut result = vec![];
        for item in self.registry.discover(None).map_err(|e| e.code)? {
            let handle = self.registry.resolve_exact(&item.pin).map_err(|e| e.code)?;
            let (checked, _) = self
                .registry
                .read_verified_bundle(&handle)
                .map_err(|e| e.code)?;
            if checked.ui.is_none() {
                continue;
            }
            self.registry
                .read_verified_ui(&handle)
                .map_err(|e| e.code)?;
            if item.pin.id.len() > 200 || item.pin.version.len() > 64 {
                return Err("DESKTOP_METADATA_QUOTA".into());
            }
            let label = checked
                .manifest
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or(&item.pin.id)
                .to_owned();
            if label.chars().count() > 160 || label.chars().any(char::is_control) {
                return Err("DESKTOP_METADATA_QUOTA".into());
            }
            result.push(Installed {
                pin: ComponentRef {
                    id: item.pin.id,
                    version: item.pin.version,
                    sha256: item.pin.sha256,
                },
                label,
            });
        }
        Ok(result)
    }
    fn read_view(&self, checkpoint: &Checkpoint) -> Result<View> {
        let handle = self
            .registry
            .resolve_exact(&release(&checkpoint.component))
            .map_err(|e| e.code)?;
        let (checked, _) = self
            .registry
            .read_verified_bundle(&handle)
            .map_err(|e| e.code)?;
        let ui = self
            .registry
            .read_verified_ui(&handle)
            .map_err(|e| e.code)?;
        let descriptor = checked.ui.as_ref().ok_or("UI_UNAVAILABLE")?;
        if checkpoint.ui
            != (UiPin {
                path: descriptor.path.clone(),
                sha256: descriptor.sha256.clone(),
                schema_sha256: descriptor.schema_sha256.clone(),
            })
        {
            return Err("UI_PIN_MISMATCH".into());
        }
        validate_checkpoint(checkpoint, &ui, &checked)?;
        let mut drafts = BTreeMap::new();
        if let Some(input) = checkpoint
            .room
            .state
            .get(&format!("component.{}", ui.input_port))
        {
            for c in &ui.document.root.children {
                if c.bind.direction == Direction::Input {
                    if let Some(value) = input.get(&ui.controls[&c.id].field) {
                        drafts.insert(c.id.clone(), value.clone());
                    }
                }
            }
        }
        Ok(View {
            ui,
            checked,
            handle,
            runtime: None,
            drafts,
        })
    }
    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }
    pub fn invocation_count(&self) -> u64 {
        self.invocations
    }
    pub fn live_audit_entries(&self) -> usize {
        self.views
            .values()
            .filter_map(|v| v.runtime.as_ref())
            .map(|r| r.room.audit().len())
            .sum()
    }
    pub fn trace(&self) -> &[Value] {
        &self.trace
    }
    pub fn recovery(&self) -> Option<&str> {
        self.recovery.as_deref()
    }
    fn persist(&mut self, candidate: Workspace) -> Result<()> {
        match self.store.commit(self.workspace.revision, &candidate) {
            Ok(next) => {
                self.workspace = next;
                Ok(())
            }
            Err(e) => {
                self.recovery = Some(e.into());
                Err(e.into())
            }
        }
    }
    fn selected(&self) -> Result<&Checkpoint> {
        self.workspace
            .rooms
            .iter()
            .find(|r| Some(&r.room_id) == self.workspace.selected_room.as_ref())
            .ok_or_else(|| "NO_SELECTION".into())
    }
    fn make_projection(&mut self, diagnostics: Vec<Diagnostic>) -> Option<Projection> {
        self.sequence = self.sequence.checked_add(1)?;
        let c = self.selected().ok()?;
        let view = self.views.get(&c.room_id)?;
        let mut values = BTreeMap::new();
        for control in &view.ui.document.root.children {
            let value = if control.bind.direction == Direction::Input {
                c.room
                    .state
                    .get(&format!("component.{}", control.bind.port))
            } else {
                c.outputs.get(&control.bind.port)
            };
            if let Some(value) = value.and_then(|v| v.get(&view.ui.controls[&control.id].field)) {
                values.insert(
                    control.id.clone(),
                    Value::String(
                        value
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| value.to_string()),
                    ),
                );
            }
        }
        Some(Projection {
            key: UiKey {
                room_id: c.room_id.clone(),
                instance_id: "component".into(),
                ui_id: format!("{}#{}", c.ui.path, c.ui.sha256),
            },
            room_revision: c.room.revision,
            execution_generation: self.epoch,
            projection_sequence: self.sequence,
            values,
            diagnostics,
        })
    }
    pub fn state(&mut self, token: &str, accepted: bool, diagnostics: Vec<Diagnostic>) -> Value {
        let projection = if self.recovery.is_none() {
            self.make_projection(diagnostics.clone())
        } else {
            None
        };
        let document = self
            .workspace
            .selected_room
            .as_ref()
            .and_then(|id| self.views.get(id))
            .map(|v| &v.ui.document);
        let document = if self.recovery.is_none() {
            document
        } else {
            None
        };
        json!({"protocol":"desktop.1","kind":"state","token":token,"accepted":accepted,
            "epoch":self.epoch.to_string(),"recovery":self.recovery,"installed":self.installed,
            "rooms":self.workspace.rooms.iter().map(|r|json!({"id":r.room_id,"label":r.label,"pinned":r.pinned,"component":r.component,"ui":r.ui})).collect::<Vec<_>>(),
            "selectedRoom":self.workspace.selected_room,"document":document,"projection":projection,
            "diagnostics":diagnostics,"invocations":self.invocations.to_string(),"pid":std::process::id(),"lastTrace":self.trace.last(),"resources":{"queueCapacity":32,"traceEntries":self.trace.len(),"liveAuditEntries":self.live_audit_entries(),"frameBytes":framing::MAX_FRAME}})
    }
    pub fn request(&mut self, request: Request) -> Value {
        let token: String = request.token.chars().take(64).collect();
        if request.protocol != "desktop.1"
            || request.token.is_empty()
            || request.token.len() > 64
            || request.epoch != self.epoch.to_string()
        {
            return self.state(&token, false, vec![Diagnostic::new("STALE_VIEW", None)]);
        }
        if matches!(&request.command, Command::Quit) {
            return self.state(&token, true, vec![]);
        }
        if let Some(error) = self.recovery.clone() {
            return self.state(&token, false, vec![Diagnostic::new(&error, None)]);
        }
        let result = match request.command {
            Command::Create { pin } => self.create(pin),
            Command::Select { room_id } => self.select(&room_id),
            Command::Pin { pinned } => self.pin(pinned),
            Command::Ui { request } => return self.ui_request(&token, request),
            Command::Quit => Ok(()),
        };
        match result {
            Ok(()) => self.state(&token, true, vec![]),
            Err(e) => self.state(&token, false, vec![Diagnostic::new(&e, None)]),
        }
    }
    fn create(&mut self, pin: ComponentRef) -> Result<()> {
        if self.workspace.rooms.len() >= MAX_ROOMS {
            return Err("ROOM_QUOTA".into());
        }
        if !self.installed.iter().any(|i| i.pin == pin) {
            return Err("COMPONENT_NOT_FOUND".into());
        }
        let handle = self
            .registry
            .resolve_exact(&release(&pin))
            .map_err(|e| e.code)?;
        let (checked, _) = self
            .registry
            .read_verified_bundle(&handle)
            .map_err(|e| e.code)?;
        let ui = self
            .registry
            .read_verified_ui(&handle)
            .map_err(|e| e.code)?;
        let d = checked.ui.as_ref().ok_or("UI_UNAVAILABLE")?;
        let mut random = [0u8; 16];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut random))
            .map_err(|_| "HOST_IO")?;
        let id = format!(
            "room-{}",
            random
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        let checkpoint = Checkpoint {
            room_id: id.clone(),
            label: ui.document.root.title.clone(),
            pinned: false,
            component: pin.clone(),
            ui: UiPin {
                path: d.path.clone(),
                sha256: d.sha256.clone(),
                schema_sha256: d.schema_sha256.clone(),
            },
            room: RoomSnapshot {
                room_format_version: "0.1".into(),
                room_id: id.clone(),
                revision: 1,
                name: ui.document.root.title.clone(),
                components: vec![wb_contracts::ipc::SnapshotComponent {
                    instance_id: "component".into(),
                    id: pin.id,
                    version: pin.version,
                    sha256: pin.sha256,
                }],
                connections: vec![],
                state: BTreeMap::new(),
                ui: UiReference {
                    definition_ref: format!(
                        "{}/{}",
                        checked.manifest["id"].as_str().unwrap(),
                        d.path
                    ),
                },
            },
            outputs: BTreeMap::new(),
        };
        let view = self.read_view(&checkpoint)?;
        let mut candidate = self.workspace.clone();
        candidate.selected_room = Some(id.clone());
        candidate.rooms.push(checkpoint);
        self.persist(candidate)?;
        self.views.insert(id, view);
        self.epoch = self.epoch.checked_add(1).ok_or("ROOM_QUOTA")?;
        Ok(())
    }
    fn select(&mut self, id: &str) -> Result<()> {
        let checkpoint = self
            .workspace
            .rooms
            .iter()
            .find(|r| r.room_id == id)
            .ok_or("ROOM_NOT_FOUND")?
            .clone();
        // Reverify exact bytes on every selection; never fall back to discovered latest.
        let view = match self.read_view(&checkpoint) {
            Ok(v) => v,
            Err(e) => {
                self.recovery = Some(e.clone());
                self.views.clear();
                return Err(e);
            }
        };
        let mut candidate = self.workspace.clone();
        candidate.selected_room = Some(id.into());
        self.persist(candidate)?;
        self.views.insert(id.into(), view);
        self.epoch = self.epoch.checked_add(1).ok_or("ROOM_QUOTA")?;
        Ok(())
    }
    fn pin(&mut self, pinned: bool) -> Result<()> {
        let id = self.selected()?.room_id.clone();
        let mut candidate = self.workspace.clone();
        candidate
            .rooms
            .iter_mut()
            .find(|r| r.room_id == id)
            .unwrap()
            .pinned = pinned;
        self.persist(candidate)
    }
    fn ui_request(&mut self, token: &str, request: UiRequest) -> Value {
        let checkpoint = match self.selected() {
            Ok(c) => c.clone(),
            Err(e) => return self.state(token, false, vec![Diagnostic::new(&e, None)]),
        };
        let id = checkpoint.room_id.clone();
        let key = UiKey {
            room_id: id.clone(),
            instance_id: "component".into(),
            ui_id: format!("{}#{}", checkpoint.ui.path, checkpoint.ui.sha256),
        };
        if request.protocol != "0.1"
            || request.key != key
            || request.local_token.is_empty()
            || request.local_token.len() > 64
        {
            return self.state(
                token,
                false,
                vec![Diagnostic::new("UI_OWNERSHIP_DENIED", None)],
            );
        }
        if request.expected_generation != self.epoch {
            return self.state(
                token,
                false,
                vec![Diagnostic::new("STALE_GENERATION", None)],
            );
        }
        let view = self.views.get_mut(&id).unwrap();
        let edits = match request.command {
            UiCommand::Edit {
                control_id,
                proposed_value,
            } => BTreeMap::from([(control_id, proposed_value)]),
            UiCommand::Apply { proposed_values } => proposed_values,
            _ => {
                return self.state(
                    token,
                    false,
                    vec![Diagnostic::new("INVALID_UI_REQUEST", None)],
                )
            }
        };
        if edits.len() > 32 {
            return self.state(
                token,
                false,
                vec![Diagnostic::new("UI_QUOTA_EXCEEDED", None)],
            );
        }
        for (name, value) in &edits {
            if name.len() > 64 {
                return self.state(
                    token,
                    false,
                    vec![Diagnostic::new("UI_BINDING_MISMATCH", None)],
                );
            }
            if !view
                .ui
                .document
                .root
                .children
                .iter()
                .any(|c| c.id == *name && c.bind.direction == Direction::Input)
                || !value.is_string() && !value.is_i64() && !value.is_u64()
                || value.to_string().len() > 512
            {
                return self.state(
                    token,
                    false,
                    vec![Diagnostic::new("UI_BINDING_MISMATCH", Some(name))],
                );
            }
        }
        view.drafts.extend(edits);
        if request.expected_room_revision != checkpoint.room.revision {
            return self.state(token, false, vec![Diagnostic::new("STATE_CONFLICT", None)]);
        }
        let aggregate = match assemble(view) {
            Ok(value) => value,
            Err(d) => return self.state(token, false, vec![d]),
        };
        // Metadata/state are reverified before any execution; no saved grant is replayed.
        if let Err(e) = self.registry.read_verified_bundle(&view.handle) {
            return self.state(token, false, vec![Diagnostic::new(&e.code, None)]);
        }
        self.next_attempt = self
            .next_attempt
            .checked_add(1)
            .expect("bounded native attempt counter");
        let correlation = format!("desktop:{}", self.next_attempt);
        let event_result = execute(&self.registry, view, &checkpoint, aggregate, &correlation);
        let mut failure = None;
        if let Some(runtime) = &mut view.runtime {
            let count = runtime
                .room
                .audit()
                .iter()
                .filter(|a| a.kind == "invocation.started")
                .count() as u64;
            self.invocations = self
                .invocations
                .saturating_add(count.saturating_sub(runtime.reported_invocations));
            runtime.reported_invocations = count;
        }
        match event_result {
            Ok((snapshot, outputs, trace, _)) => {
                let mut next = checkpoint.clone();
                next.room = snapshot;
                next.outputs = outputs;
                if let Err(e) = validate_checkpoint(&next, &view.ui, &view.checked) {
                    failure = Some(e);
                } else {
                    let mut candidate = self.workspace.clone();
                    *candidate
                        .rooms
                        .iter_mut()
                        .find(|c| c.room_id == id)
                        .unwrap() = next;
                    if let Err(e) = self.persist(candidate) {
                        failure = Some(e);
                    } else {
                        self.push_trace(trace);
                    }
                }
            }
            Err(e) => {
                let audit = self.views[&id]
                    .runtime
                    .as_ref()
                    .map(|runtime| {
                        runtime
                            .room
                            .audit()
                            .iter()
                            .rev()
                            .take(16)
                            .rev()
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                self.push_trace(json!({"diagnostic":e,"audit":audit,"nativeCorrelation":correlation,"requestToken":token,"checkpointPreserved":true}));
                failure = Some(e);
            }
        }
        if failure.is_none()
            && self.views[&id]
                .runtime
                .as_ref()
                .is_some_and(|runtime| runtime.room.audit().len() > 256)
        {
            // Bound accepted kernel audit memory without changing kernel semantics.
            // The next explicit valid edit recreates a fresh approved incarnation.
            self.views.get_mut(&id).unwrap().runtime = None;
        }
        if let Some(error) = failure {
            self.views.get_mut(&id).unwrap().runtime = None;
            self.state(token, false, vec![Diagnostic::new(&error, None)])
        } else {
            self.state(token, true, vec![])
        }
    }
    fn push_trace(&mut self, event: Value) {
        if self.trace.len() == 128 {
            self.trace.remove(0);
        }
        self.trace.push(event);
    }
}
fn validate_checkpoint(c: &Checkpoint, ui: &ValidatedUi, m: &CheckedMetadata) -> Result<()> {
    let input_key = format!("component.{}", ui.input_port);
    if c.room.state.is_empty() && c.outputs.is_empty() {
        return Ok(());
    }
    if c.room.state.len() != 1 || !c.room.state.contains_key(&input_key) {
        return Err("INVALID_SNAPSHOT".into());
    }
    let ports = &m.manifest["ports"];
    for direction in ["inputs", "outputs"] {
        let list = ports[direction].as_array().ok_or("INVALID_MANIFEST")?;
        if direction == "outputs" && c.outputs.len() != list.len() {
            return Err("INVALID_SNAPSHOT".into());
        }
        for p in list {
            let name = p["name"].as_str().ok_or("INVALID_MANIFEST")?;
            if direction == "inputs" && name != ui.input_port {
                return Err("UI_BINDING_MISMATCH".into());
            }
            let value = if direction == "inputs" {
                &c.room.state[&input_key]
            } else {
                c.outputs.get(name).ok_or("INVALID_SNAPSHOT")?
            };
            let schema = &m.schemas[p["schema"].as_str().ok_or("INVALID_MANIFEST")?];
            if !jsonschema::validator_for(schema)
                .map_err(|_| "INVALID_SCHEMA")?
                .is_valid(value)
            {
                return Err("INVALID_SNAPSHOT".into());
            }
        }
    }
    for control in &ui.document.root.children {
        let source = if control.bind.direction == Direction::Input {
            &c.room.state[&input_key]
        } else {
            c.outputs
                .get(&control.bind.port)
                .ok_or("INVALID_SNAPSHOT")?
        };
        let value = source
            .get(&ui.controls[&control.id].field)
            .ok_or("INVALID_SNAPSHOT")?;
        if value.to_string().len() > 512 {
            return Err("SNAPSHOT_QUOTA".into());
        }
    }
    Ok(())
}
fn normalize(kind: &ControlKind, value: &Value) -> Option<Value> {
    match kind {
        ControlKind::DecimalInput => {
            let s = value.as_str()?;
            let mut parts = s.split('.');
            let first = parts.next()?;
            let fraction = parts.next();
            if first.is_empty()
                || first.len() > 1 && first.starts_with('0')
                || !first.bytes().all(|b| b.is_ascii_digit())
                || parts.next().is_some()
                || fraction.is_some_and(|f| f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()))
            {
                None
            } else {
                Some(value.clone())
            }
        }
        ControlKind::IntegerInput => {
            if value.is_i64() || value.is_u64() {
                return Some(value.clone());
            }
            let s = value.as_str()?;
            let unsigned = s.strip_prefix('-').unwrap_or(s);
            if unsigned.is_empty()
                || unsigned.len() > 1 && unsigned.starts_with('0')
                || !unsigned.bytes().all(|b| b.is_ascii_digit())
                || s == "-0"
            {
                return None;
            }
            s.parse::<i64>()
                .ok()
                .map(Value::from)
                .or_else(|| s.parse::<u64>().ok().map(Value::from))
        }
        ControlKind::Readout => None,
    }
}
fn assemble(view: &View) -> std::result::Result<Value, Diagnostic> {
    let mut object = serde_json::Map::new();
    for c in &view.ui.document.root.children {
        if c.bind.direction != Direction::Input {
            continue;
        }
        let raw = view
            .drafts
            .get(&c.id)
            .ok_or_else(|| Diagnostic::new("INCOMPLETE_INPUT", Some(&c.id)))?;
        let value = normalize(&c.kind, raw)
            .ok_or_else(|| Diagnostic::new("SCHEMA_VIOLATION", Some(&c.id)))?;
        let bound = &view.ui.controls[&c.id];
        if !jsonschema::validator_for(&bound.schema)
            .unwrap()
            .is_valid(&value)
        {
            return Err(Diagnostic::new("SCHEMA_VIOLATION", Some(&c.id)));
        }
        object.insert(bound.field.clone(), value);
    }
    let aggregate = Value::Object(object);
    let p = view.checked.manifest["ports"]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == view.ui.input_port)
        .unwrap();
    if !jsonschema::validator_for(&view.checked.schemas[p["schema"].as_str().unwrap()])
        .unwrap()
        .is_valid(&aggregate)
    {
        return Err(Diagnostic::new("SCHEMA_VIOLATION", None));
    }
    Ok(aggregate)
}
type Execution = (RoomSnapshot, BTreeMap<String, Value>, Value, u64);
fn execute(
    registry: &Registry,
    view: &mut View,
    c: &Checkpoint,
    value: Value,
    correlation: &str,
) -> Result<Execution> {
    let mut activation_count = 0;
    if view.runtime.is_none() {
        let kernel = Kernel::new(
            wb_wasm_host::Limits {
                fuel: 20_000_000,
                ..Default::default()
            },
            8,
            8,
        )
        .map_err(code)?;
        let pin = kernel
            .cache_registry_release(registry, &view.handle)
            .map_err(code)?;
        kernel
            .approve_execution(&c.room_id, "component", &pin)
            .map_err(code)?;
        let mut room = if c.room.state.is_empty() {
            let mut room = kernel
                .create_room(&c.room_id, &c.label, c.room.ui.clone(), 128)
                .map_err(code)?;
            room.apply_plan(
                CompositionPlan {
                    plan_version: "0.1".into(),
                    room_id: c.room_id.clone(),
                    expected_room_revision: 0,
                    instances: vec![InstanceSpec {
                        instance_id: "component".into(),
                        component: pin,
                    }],
                    connections: vec![],
                    ui: c.room.ui.clone(),
                    requested_permissions: vec![],
                },
                "desktop.host",
                "desktop.create",
            )
            .map_err(code)?;
            room.prepare("component", "desktop.host", "desktop.prepare")
                .map_err(code)?;
            room.start("component", "desktop.host", "desktop.start")
                .map_err(code)?;
            room
        } else {
            let mut room = kernel.restore_room(c.room.clone(), 128).map_err(code)?;
            if let Some(e) = room.run_until_idle().into_iter().next() {
                return Err(code(e));
            }
            room
        };
        activation_count = room
            .audit()
            .iter()
            .filter(|a| a.kind == "invocation.started")
            .count() as u64;
        let adapter = room
            .authorize_ui(
                "desktop.host",
                [Endpoint::new("component", &view.ui.input_port)]
                    .into_iter()
                    .collect(),
            )
            .map_err(code)?;
        view.runtime = Some(Runtime {
            room,
            adapter,
            reported_invocations: 0,
        });
    }
    let runtime = view.runtime.as_mut().unwrap();
    let before = runtime.room.audit().len();
    let event = runtime
        .room
        .submit_ui(
            &runtime.adapter,
            correlation,
            Endpoint::new("component", &view.ui.input_port),
            value,
            correlation,
        )
        .map_err(code)?;
    let errors = runtime.room.run_until_idle();
    let audit = &runtime.room.audit()[before..];
    let count = activation_count
        + audit
            .iter()
            .filter(|a| a.kind == "invocation.started")
            .count() as u64;
    if let Some(e) = errors.into_iter().next() {
        return Err(code(e));
    }
    let mut outputs = BTreeMap::new();
    for p in view.checked.manifest["ports"]["outputs"]
        .as_array()
        .unwrap()
    {
        let name = p["name"].as_str().unwrap();
        let projection = runtime
            .room
            .output("component", name)
            .ok_or("INCOMPLETE_OUTPUT")?;
        if !projection.valid {
            return Err("STALE_RESULT".into());
        }
        outputs.insert(
            name.into(),
            projection.value.clone().ok_or("INCOMPLETE_OUTPUT")?,
        );
    }
    let snapshot = runtime.room.snapshot();
    let trace = json!({"ingress":event,"audit":audit,"committedInput":snapshot.state,"committedOutput":outputs});
    Ok((snapshot, outputs, trace, count))
}
