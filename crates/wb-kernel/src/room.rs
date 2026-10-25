//! Serialized room control plane. Invocation tickets contain no room-state authority.
use crate::runtime::{error, host_failure, Artifact, Kernel, Lease, Services};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};
use wb_bus::ipc::{RoomQueue, RoutedData, Sequence};
use wb_contracts::{ipc::*, ComponentRef};
use wb_reactive::scheduler::{check_fence, Graph};
use wb_wasm_host::Message;

#[derive(Clone)]
struct Instance {
    artifact: Arc<Artifact>,
    status: InstanceStatus,
    generation: u64,
    _lease: Arc<Lease>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    pub value: Option<Value>,
    pub valid: bool,
    pub error: Option<ErrorCode>,
    pub revision: u64,
}
struct Active {
    id: String,
    instance: String,
    correlation: String,
    cancel: Arc<AtomicBool>,
}
/// Sealed, room-scoped UI adapter. Not a serialized credential or guest API.
pub struct UiAdapter {
    incarnation: u64,
    token: u64,
}
struct Adapter {
    actor: String,
    allowed: BTreeSet<Endpoint>,
}
pub struct Invocation {
    services: Arc<Services>,
    artifact: Arc<Artifact>,
    room: String,
    instance: String,
    context: DispatchContext,
    correlation: String,
    message: Message,
    cancel: Arc<AtomicBool>,
    _lease: Arc<Lease>,
}
pub struct Completion {
    room: String,
    instance: String,
    context: DispatchContext,
    correlation: String,
    result: Result<Vec<Message>>,
    elapsed: u64,
    _lease: Arc<Lease>,
}
impl Invocation {
    pub fn context(&self) -> &DispatchContext {
        &self.context
    }
    /// Runs outside the room lock. Every attempt uses the existing ADR-001 host.
    pub fn run(self) -> Completion {
        let start = Instant::now();
        let mut result = if self.cancel.load(Ordering::SeqCst) {
            Err(error(ErrorCode::Cancelled))
        } else {
            self.services
                .approved(&self.room, &self.instance, &self.artifact.pin)
                .and_then(|()| {
                    self.services
                        .host
                        .call(&self.artifact.registered, &self.message)
                        .map_err(host_failure)
                })
        };
        if self.cancel.load(Ordering::SeqCst) {
            result = Err(error(ErrorCode::Cancelled));
        }
        Completion {
            room: self.room,
            instance: self.instance,
            context: self.context,
            correlation: self.correlation,
            result,
            elapsed: start.elapsed().as_micros().min(u64::MAX as u128) as u64,
            _lease: self._lease,
        }
    }
}
pub struct Room {
    services: Arc<Services>,
    id: String,
    name: String,
    incarnation: u64,
    revision: u64,
    ui: UiReference,
    instances: BTreeMap<String, Instance>,
    connections: Vec<Connection>,
    graph: Graph,
    queue: RoomQueue,
    clock: Arc<Sequence>,
    active: Option<Active>,
    adapters: BTreeMap<u64, Adapter>,
    inputs: BTreeMap<Endpoint, Value>,
    outputs: BTreeMap<Endpoint, Projection>,
    audit: Vec<AuditEvent>,
}
impl Drop for Room {
    fn drop(&mut self) {
        if let Some(active) = &self.active {
            active.cancel.store(true, Ordering::SeqCst);
        }
        self.services.rooms.lock().unwrap().remove(&self.id);
    }
}
fn validate_document(value: &Value, room: bool) -> Result<()> {
    let plan: Value =
        serde_json::from_str(include_str!("../../../schemas/composition.schema.json")).unwrap();
    let schema: Value = if room {
        serde_json::from_str(include_str!("../../../schemas/room.schema.json")).unwrap()
    } else {
        plan.clone()
    };
    let resource = jsonschema::Resource::from_contents(plan.clone())
        .map_err(|_| error(ErrorCode::SchemaViolation))?;
    let validator = jsonschema::options()
        .with_resource(plan["$id"].as_str().unwrap(), resource)
        .build(&schema)
        .map_err(|_| error(ErrorCode::SchemaViolation))?;
    validator
        .validate(value)
        .map_err(|_| error(ErrorCode::SchemaViolation))
}
impl Room {
    pub(crate) fn new(
        services: Arc<Services>,
        id: &str,
        name: &str,
        ui: UiReference,
        capacity: usize,
    ) -> Result<Self> {
        if id.is_empty() || id.len() > 128 || name.is_empty() {
            return Err(error(ErrorCode::InvalidEnvelope));
        }
        let incarnation = services.token()?;
        let clock = services.clock(id);
        let queue = RoomQueue::new(id, incarnation, capacity, clock.clone())?;
        let graph = Graph::new(Vec::new(), &[])?;
        if !services.rooms.lock().unwrap().insert(id.into()) {
            return Err(error(ErrorCode::DuplicateId));
        }
        Ok(Self {
            services,
            id: id.into(),
            name: name.into(),
            incarnation,
            revision: 0,
            ui,
            instances: BTreeMap::new(),
            connections: Vec::new(),
            graph,
            queue,
            clock,
            active: None,
            adapters: BTreeMap::new(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            audit: Vec::new(),
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn audit(&self) -> &[AuditEvent] {
        &self.audit
    }
    pub fn pending(&self) -> Vec<EventEnvelope> {
        self.queue.pending()
    }
    pub fn state(&self, id: &str) -> Option<InstanceStatus> {
        self.instances.get(id).map(|i| i.status)
    }
    pub fn generation(&self, id: &str) -> Option<u64> {
        self.instances.get(id).map(|i| i.generation)
    }
    pub fn input(&self, id: &str, port: &str) -> Option<&Value> {
        self.inputs.get(&Endpoint::new(id, port))
    }
    pub fn output(&self, id: &str, port: &str) -> Option<&Projection> {
        self.outputs.get(&Endpoint::new(id, port))
    }
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }
    pub fn instance_ids(&self) -> Vec<String> {
        self.instances.keys().cloned().collect()
    }
    fn record(
        &mut self,
        kind: &str,
        id: Option<&str>,
        actor: &str,
        correlation: &str,
        code: Option<ErrorCode>,
        elapsed: Option<u64>,
    ) -> Result<()> {
        self.audit.push(AuditEvent {
            kind: kind.into(),
            room_id: self.id.clone(),
            instance_id: id.map(str::to_owned),
            revision: self.revision,
            sequence: self.clock.next()?,
            actor: actor.into(),
            correlation_id: correlation.into(),
            error_code: code,
            elapsed_micros: elapsed,
        });
        Ok(())
    }
    fn reject(&mut self, e: HostError, id: Option<&str>, correlation: &str) -> HostError {
        let kind = if e.code == ErrorCode::QueueFull {
            "backpressure.rejected"
        } else {
            "dispatch.rejected"
        };
        let _ = self.record(kind, id, "host", correlation, Some(e.code), None);
        e.correlated(correlation)
    }
    fn next_revision(&self) -> Result<u64> {
        self.revision
            .checked_add(1)
            .ok_or_else(|| error(ErrorCode::ResourceLimit))
    }
    fn stage(&self, plan: &CompositionPlan) -> Result<(BTreeMap<String, Instance>, Graph)> {
        if plan
            .instances
            .iter()
            .map(|i| &i.instance_id)
            .collect::<BTreeSet<_>>()
            .len()
            != plan.instances.len()
        {
            return Err(error(ErrorCode::DuplicateId));
        }
        validate_document(&serde_json::to_value(plan).unwrap(), false)?;
        if plan.plan_version != "0.1" {
            return Err(error(ErrorCode::UnsupportedVersion));
        }
        if plan.room_id != self.id || plan.expected_room_revision != self.revision {
            return Err(error(ErrorCode::StateConflict));
        }
        if !plan.requested_permissions.is_empty() {
            return Err(error(ErrorCode::CapabilityDenied));
        }
        let mut staged = BTreeMap::new();
        for spec in &plan.instances {
            if spec.instance_id.len() > 128 {
                return Err(error(ErrorCode::ResourceLimit));
            }
            if staged.contains_key(&spec.instance_id) {
                return Err(error(ErrorCode::DuplicateId));
            }
            let artifact = self.services.resolve(&spec.component)?;
            self.services
                .approved(&self.id, &spec.instance_id, &spec.component)?;
            if artifact.inputs.len() != 1 {
                return Err(error(ErrorCode::IncompatiblePorts));
            }
            let old = self
                .instances
                .get(&spec.instance_id)
                .filter(|i| i.artifact.pin == spec.component);
            let instance = match old {
                Some(i) => i.clone(),
                None => Instance {
                    artifact,
                    status: InstanceStatus::Declared,
                    generation: self.services.token()?,
                    _lease: self.services.instances.reserve()?,
                },
            };
            staged.insert(spec.instance_id.clone(), instance);
        }
        let mut destinations = BTreeSet::new();
        for edge in &plan.connections {
            let from = staged
                .get(&edge.from.instance_id)
                .and_then(|i| i.artifact.outputs.get(&edge.from.port))
                .ok_or_else(|| error(ErrorCode::IncompatiblePorts))?;
            let to = staged
                .get(&edge.to.instance_id)
                .and_then(|i| i.artifact.inputs.get(&edge.to.port))
                .ok_or_else(|| error(ErrorCode::IncompatiblePorts))?;
            if from.schema != to.schema
                || from.semantic != to.semantic
                || from.unit != to.unit
                || !destinations.insert(edge.to.clone())
            {
                return Err(error(ErrorCode::IncompatiblePorts));
            }
        }
        let graph = Graph::new(staged.keys().cloned(), &plan.connections)?;
        Ok((staged, graph))
    }
    pub fn apply_plan(
        &mut self,
        plan: CompositionPlan,
        actor: &str,
        correlation: &str,
    ) -> Result<()> {
        let (staged, graph) = match self.stage(&plan) {
            Ok(v) => v,
            Err(e) => return Err(self.reject(e, None, correlation)),
        };
        let revision = self.next_revision()?;
        let removed: Vec<_> = self
            .instances
            .iter()
            .filter(|(id, i)| {
                staged
                    .get(*id)
                    .is_none_or(|s| s.artifact.pin != i.artifact.pin)
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in &removed {
            self.cancel_instance(id)?;
        }
        let added: Vec<_> = staged
            .iter()
            .filter(|(id, i)| {
                self.instances
                    .get(*id)
                    .is_none_or(|s| s.artifact.pin != i.artifact.pin)
            })
            .map(|(id, _)| id.clone())
            .collect();
        self.instances = staged;
        self.graph = graph;
        self.connections = plan.connections;
        self.ui = plan.ui;
        self.revision = revision;
        for id in &removed {
            self.inputs.retain(|e, _| e.instance_id != *id);
            self.outputs.retain(|e, _| e.instance_id != *id);
            self.record("instance.removed", Some(id), actor, correlation, None, None)?;
        }
        for id in &added {
            self.record(
                "instance.declared",
                Some(id),
                actor,
                correlation,
                None,
                None,
            )?;
        }
        // Topology changes invalidate cached derived projections, never authoritative inputs.
        for projection in self.outputs.values_mut() {
            projection.valid = false;
        }
        self.record("room.revised", None, actor, correlation, None, None)
    }
    pub fn prepare(&mut self, id: &str, actor: &str, correlation: &str) -> Result<()> {
        let check = (|| {
            let i = self
                .instances
                .get(id)
                .ok_or_else(|| error(ErrorCode::ComponentNotFound))?;
            if !matches!(
                i.status,
                InstanceStatus::Declared | InstanceStatus::Stopped | InstanceStatus::Failed
            ) {
                return Err(error(ErrorCode::InvalidTransition));
            }
            self.services.approved(&self.id, id, &i.artifact.pin)?;
            self.services
                .host
                .register(&i.artifact.manifest, &i.artifact.bytes, &i.artifact.schemas)
                .map_err(host_failure)?;
            Ok(())
        })();
        if let Err(e) = check {
            return Err(self.reject(e, Some(id), correlation));
        }
        let generation = self.services.token()?;
        let i = self.instances.get_mut(id).unwrap();
        i.status = InstanceStatus::Prepared;
        i.generation = generation;
        self.record(
            "instance.prepared",
            Some(id),
            actor,
            correlation,
            None,
            None,
        )
    }
    pub fn start(&mut self, id: &str, actor: &str, correlation: &str) -> Result<()> {
        let check = (|| {
            let i = self
                .instances
                .get(id)
                .ok_or_else(|| error(ErrorCode::ComponentNotFound))?;
            if i.status != InstanceStatus::Prepared {
                return Err(error(ErrorCode::InvalidTransition));
            }
            self.services.approved(&self.id, id, &i.artifact.pin)
        })();
        if let Err(e) = check {
            return Err(self.reject(e, Some(id), correlation));
        }
        self.instances.get_mut(id).unwrap().status = InstanceStatus::Running;
        self.record("instance.started", Some(id), actor, correlation, None, None)
    }
    fn invalidate(&mut self, root: &str, code: Option<ErrorCode>) {
        let affected = self.graph.affected(root);
        for (e, p) in &mut self.outputs {
            if affected.contains(&e.instance_id) {
                p.valid = false;
                p.error = code;
            }
        }
    }
    fn cancel_instance(&mut self, id: &str) -> Result<()> {
        let affected = self.graph.affected(id);
        if self
            .active
            .as_ref()
            .is_some_and(|a| affected.contains(&a.instance))
        {
            let active = self.active.take().unwrap();
            active.cancel.store(true, Ordering::SeqCst);
            self.record(
                "invocation.failed",
                Some(&active.instance),
                "host",
                &active.correlation,
                Some(ErrorCode::Cancelled),
                None,
            )?;
        }
        for event in self.queue.cancel_targets(&self.graph.affected(id)) {
            self.record(
                "dispatch.rejected",
                Some(&event.envelope.destination.instance_id),
                "host",
                &event.envelope.correlation_id,
                Some(ErrorCode::Cancelled),
                None,
            )?;
        }
        self.invalidate(id, Some(ErrorCode::Cancelled));
        Ok(())
    }
    /// Direct serialized control path: does not wait for or consume data capacity.
    pub fn stop(&mut self, id: &str, actor: &str, correlation: &str) -> Result<()> {
        if self.state(id) != Some(InstanceStatus::Running) {
            return Err(self.reject(error(ErrorCode::InvalidTransition), Some(id), correlation));
        }
        let generation = self.services.token()?;
        self.cancel_instance(id)?;
        let i = self.instances.get_mut(id).unwrap();
        i.status = InstanceStatus::Stopped;
        i.generation = generation;
        self.record("instance.stopped", Some(id), actor, correlation, None, None)
    }
    pub fn authorize_ui(&mut self, actor: &str, allowed: BTreeSet<Endpoint>) -> Result<UiAdapter> {
        for e in &allowed {
            if !self
                .instances
                .get(&e.instance_id)
                .is_some_and(|i| i.artifact.inputs.contains_key(&e.port))
            {
                return Err(error(ErrorCode::IncompatiblePorts));
            }
        }
        let token = self.services.token()?;
        self.adapters.insert(
            token,
            Adapter {
                actor: actor.into(),
                allowed,
            },
        );
        Ok(UiAdapter {
            incarnation: self.incarnation,
            token,
        })
    }
    pub fn revoke_ui(&mut self, adapter: &UiAdapter) {
        self.adapters.remove(&adapter.token);
    }
    pub fn submit_ui(
        &mut self,
        adapter: &UiAdapter,
        request_id: &str,
        destination: Endpoint,
        payload: Value,
        correlation: &str,
    ) -> Result<EventEnvelope> {
        let result = (|| {
            if request_id.len() > 256 || adapter.incarnation != self.incarnation {
                return Err(error(ErrorCode::CapabilityDenied));
            }
            let a = self
                .adapters
                .get(&adapter.token)
                .filter(|a| a.allowed.contains(&destination))
                .ok_or_else(|| error(ErrorCode::CapabilityDenied))?;
            if self.connections.iter().any(|e| e.to == destination) {
                return Err(error(ErrorCode::IncompatiblePorts));
            }
            let i = self
                .instances
                .get(&destination.instance_id)
                .ok_or_else(|| error(ErrorCode::ComponentNotFound))?;
            if i.status != InstanceStatus::Running {
                return Err(error(ErrorCode::InvalidTransition));
            }
            self.services
                .approved(&self.id, &destination.instance_id, &i.artifact.pin)?;
            i.artifact.validate(true, &destination.port, &payload)?;
            let context = DispatchContext {
                room_revision: self.revision,
                execution_generation: i.generation,
                invocation_id: String::new(),
            };
            let route = RoutedData {
                source: Endpoint::new(&format!("@ui:{}", adapter.token), "input"),
                destination: destination.clone(),
                schema_id: i.artifact.inputs[&destination.port].schema_id.clone(),
                payload: payload.clone(),
                generation: i.generation,
                ui_token: Some(adapter.token),
            };
            let actor = a.actor.clone();
            let mut batch = self.queue.publish(
                &format!("ui:{}:{request_id}", adapter.token),
                correlation,
                context,
                vec![route],
            )?;

            self.record(
                "dispatch.accepted",
                Some(&destination.instance_id),
                &actor,
                correlation,
                None,
                None,
            )?;

            Ok(batch.remove(0))
        })();
        result.map_err(|e| self.reject(e, Some(&destination.instance_id), correlation))
    }
    pub fn next_invocation(&mut self) -> Result<Option<Invocation>> {
        if self.active.is_some() {
            return Err(error(ErrorCode::InvalidTransition));
        }
        let Some(event) = self.queue.pop_scheduled(self.graph.order()) else {
            return Ok(None);
        };
        let id = event.envelope.destination.instance_id.clone();
        let correlation = event.envelope.correlation_id;
        let result = (|| {
            let i = self
                .instances
                .get(&id)
                .ok_or_else(|| error(ErrorCode::StaleResult))?;
            if i.generation != event.context.execution_generation {
                return Err(error(ErrorCode::StaleResult));
            }
            if i.status != InstanceStatus::Running {
                return Err(error(ErrorCode::InvalidTransition));
            }
            let actor = if let Some(token) = event.ui_token {
                if self
                    .connections
                    .iter()
                    .any(|e| e.to == event.envelope.destination)
                {
                    return Err(error(ErrorCode::IncompatiblePorts));
                }
                Some(
                    self.adapters
                        .get(&token)
                        .filter(|a| a.allowed.contains(&event.envelope.destination))
                        .ok_or_else(|| error(ErrorCode::CapabilityDenied))?
                        .actor
                        .clone(),
                )
            } else {
                check_fence(self.revision, i.generation, &event.context)?;
                None
            };
            self.services.approved(&self.id, &id, &i.artifact.pin)?;
            i.artifact.validate(
                true,
                &event.envelope.destination.port,
                &event.envelope.payload,
            )?;
            let artifact = i.artifact.clone();
            let revision = if actor.is_some() {
                self.next_revision()?
            } else {
                self.revision
            };
            let lease = self.services.invocations.reserve()?;
            let invocation_id = format!(
                "{}:{}:{}",
                self.id,
                self.incarnation,
                self.services.token()?
            );
            let cancel = Arc::new(AtomicBool::new(false));
            let context = DispatchContext {
                invocation_id: invocation_id.clone(),
                room_revision: revision,
                execution_generation: event.context.execution_generation,
            };
            let ticket = Invocation {
                services: self.services.clone(),
                artifact,
                room: self.id.clone(),
                instance: id.clone(),
                context,
                correlation: correlation.clone(),
                message: Message {
                    port: event.envelope.destination.port.clone(),
                    payload_json: event.envelope.payload.to_string(),
                },
                cancel: cancel.clone(),
                _lease: lease,
            };
            if let Some(actor) = actor {
                self.revision = revision;
                self.inputs.insert(
                    event.envelope.destination.clone(),
                    event.envelope.payload.clone(),
                );
                self.invalidate(&id, None);
                self.record("room.revised", None, &actor, &correlation, None, None)?;
            }
            self.active = Some(Active {
                id: invocation_id,
                instance: id.clone(),
                correlation: correlation.clone(),
                cancel,
            });
            self.record(
                "invocation.started",
                Some(&id),
                "scheduler",
                &correlation,
                None,
                None,
            )?;
            Ok(Some(ticket))
        })();
        result.map_err(|e| self.reject(e, Some(&id), &correlation))
    }
    pub fn complete(&mut self, completion: Completion) -> Result<Vec<EventEnvelope>> {
        let id = &completion.instance;
        let correlation = &completion.correlation;
        let is_active = self
            .active
            .as_ref()
            .is_some_and(|a| a.id == completion.context.invocation_id);
        let fence = self
            .instances
            .get(id)
            .map(|i| check_fence(self.revision, i.generation, &completion.context));
        if completion.room != self.id || !is_active || !matches!(fence, Some(Ok(()))) {
            if is_active {
                self.active.take();
            }
            return Err(self.reject(error(ErrorCode::StaleResult), Some(id), correlation));
        }
        self.active.take();
        let result = (|| {
            let instance = &self.instances[id];
            self.services
                .approved(&self.id, id, &instance.artifact.pin)?;
            let messages = completion.result?;
            let mut values = BTreeMap::new();
            let mut routes = Vec::new();
            for message in messages {
                let value: Value = serde_json::from_str(&message.payload_json)
                    .map_err(|_| error(ErrorCode::SchemaViolation))?;
                instance.artifact.validate(false, &message.port, &value)?;
                if values.insert(message.port.clone(), value.clone()).is_some() {
                    return Err(error(ErrorCode::SchemaViolation));
                }
                for edge in self
                    .connections
                    .iter()
                    .filter(|e| e.from == Endpoint::new(id, &message.port))
                {
                    let target = &self.instances[&edge.to.instance_id];
                    if target.status != InstanceStatus::Running {
                        return Err(error(ErrorCode::InvalidTransition));
                    }
                    self.services
                        .approved(&self.id, &edge.to.instance_id, &target.artifact.pin)?;
                    target.artifact.validate(true, &edge.to.port, &value)?;
                    routes.push(RoutedData {
                        source: edge.from.clone(),
                        destination: edge.to.clone(),
                        schema_id: target.artifact.inputs[&edge.to.port].schema_id.clone(),
                        payload: value.clone(),
                        generation: target.generation,
                        ui_token: None,
                    });
                }
            }
            routes.sort_by_key(|e| {
                (
                    self.graph
                        .order()
                        .iter()
                        .position(|id| id == &e.destination.instance_id),
                    e.destination.clone(),
                )
            });
            let events = self.queue.publish(
                &completion.context.invocation_id,
                correlation,
                completion.context.clone(),
                routes,
            )?;
            for (port, value) in values {
                self.outputs.insert(
                    Endpoint::new(id, &port),
                    Projection {
                        value: Some(value),
                        valid: true,
                        error: None,
                        revision: self.revision,
                    },
                );
            }
            for e in &events {
                self.inputs.insert(e.destination.clone(), e.payload.clone());
            }
            Ok(events)
        })();
        match result {
            Ok(events) => {
                self.record(
                    "invocation.completed",
                    Some(id),
                    "host",
                    correlation,
                    None,
                    Some(completion.elapsed),
                )?;
                for e in &events {
                    self.record(
                        "dispatch.accepted",
                        Some(&e.destination.instance_id),
                        "host",
                        correlation,
                        None,
                        None,
                    )?;
                }
                Ok(events)
            }
            Err(e) => {
                if e.code == ErrorCode::QueueFull {
                    self.record(
                        "backpressure.rejected",
                        Some(id),
                        "host",
                        correlation,
                        Some(e.code),
                        None,
                    )?;
                }
                self.invalidate(id, Some(e.code));
                if matches!(e.code, ErrorCode::ComponentTrap | ErrorCode::ResourceLimit) {
                    let generation = self.services.token()?;
                    let i = self.instances.get_mut(id).unwrap();
                    i.status = InstanceStatus::Failed;
                    i.generation = generation;
                    self.cancel_instance(id)?;
                    self.invalidate(id, Some(e.code));
                    self.record(
                        "instance.failed",
                        Some(id),
                        "host",
                        correlation,
                        Some(e.code),
                        None,
                    )?;
                }
                self.record(
                    "invocation.failed",
                    Some(id),
                    "host",
                    correlation,
                    Some(e.code),
                    Some(completion.elapsed),
                )?;
                Err(e.correlated(correlation))
            }
        }
    }
    /// Sequential within this room; other room controllers can execute concurrently.
    pub fn run_until_idle(&mut self) -> Vec<HostError> {
        let mut errors = Vec::new();
        while !self.queue.is_empty() {
            match self.next_invocation() {
                Ok(Some(i)) => {
                    if let Err(e) = self.complete(i.run()) {
                        errors.push(e);
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    errors.push(e);
                    if self.active.is_some() {
                        break;
                    }
                }
            }
        }
        errors
    }
    pub fn snapshot(&self) -> RoomSnapshot {
        let mut state = BTreeMap::new();
        for (e, v) in &self.inputs {
            state.insert(format!("{}.{}", e.instance_id, e.port), v.clone());
        }
        RoomSnapshot {
            room_format_version: "0.1".into(),
            room_id: self.id.clone(),
            revision: self.revision,
            name: self.name.clone(),
            components: self
                .instances
                .iter()
                .map(|(id, i)| SnapshotComponent {
                    instance_id: id.clone(),
                    id: i.artifact.pin.id.clone(),
                    version: i.artifact.pin.version.clone(),
                    sha256: i.artifact.pin.sha256.clone(),
                })
                .collect(),
            connections: self.connections.clone(),
            state,
            ui: self.ui.clone(),
        }
    }
}
impl Kernel {
    /// Logical interchange only: no database, Store, queue, adapter or grant replay.
    pub fn restore_room(&self, snapshot: RoomSnapshot, capacity: usize) -> Result<Room> {
        validate_document(&serde_json::to_value(&snapshot).unwrap(), true)?;
        let mut room = self.create_room(
            &snapshot.room_id,
            &snapshot.name,
            snapshot.ui.clone(),
            capacity,
        )?;
        let plan = CompositionPlan {
            plan_version: "0.1".into(),
            room_id: snapshot.room_id,
            expected_room_revision: 0,
            instances: snapshot
                .components
                .into_iter()
                .map(|c| InstanceSpec {
                    instance_id: c.instance_id,
                    component: ComponentRef {
                        id: c.id,
                        version: c.version,
                        sha256: c.sha256,
                    },
                })
                .collect(),
            connections: snapshot.connections,
            ui: snapshot.ui,
            requested_permissions: vec![],
        };
        let (staged, graph) = room.stage(&plan)?;
        room.instances = staged;
        room.graph = graph;
        room.connections = plan.connections;
        room.revision = snapshot.revision;
        for (path, value) in snapshot.state {
            let parts: Vec<_> = path.split('.').collect();
            if parts.len() != 2 {
                return Err(error(ErrorCode::RestoreFailed));
            }
            let e = Endpoint::new(parts[0], parts[1]);
            let i = room
                .instances
                .get(parts[0])
                .ok_or_else(|| error(ErrorCode::RestoreFailed))?;
            if i.artifact.inputs.contains_key(parts[1]) {
                i.artifact.validate(true, parts[1], &value)?;
                room.inputs.insert(e, value);
            } else {
                i.artifact.validate(false, parts[1], &value)?;
            }
        }
        for id in room.instance_ids() {
            let i = &room.instances[&id];
            self.services.approved(&room.id, &id, &i.artifact.pin)?;
            self.services
                .host
                .register(&i.artifact.manifest, &i.artifact.bytes, &i.artifact.schemas)
                .map_err(host_failure)?;
            room.record(
                "instance.declared",
                Some(&id),
                "restore",
                "restore",
                None,
                None,
            )?;
            let generation = self.services.token()?;
            let i = room.instances.get_mut(&id).unwrap();
            i.generation = generation;
            i.status = InstanceStatus::Prepared;
            room.record(
                "instance.prepared",
                Some(&id),
                "restore",
                "restore",
                None,
                None,
            )?;
            room.instances.get_mut(&id).unwrap().status = InstanceStatus::Running;
            room.record(
                "instance.started",
                Some(&id),
                "restore",
                "restore",
                None,
                None,
            )?;
        }
        for (endpoint, value) in room.inputs.clone() {
            if room.connections.iter().any(|e| e.to == endpoint) {
                continue;
            }
            let i = &room.instances[&endpoint.instance_id];
            let route = RoutedData {
                source: Endpoint::new("@restore", "input"),
                destination: endpoint.clone(),
                schema_id: i.artifact.inputs[&endpoint.port].schema_id.clone(),
                payload: value,
                generation: i.generation,
                ui_token: None,
            };
            let context = DispatchContext {
                room_revision: room.revision,
                execution_generation: i.generation,
                invocation_id: String::new(),
            };
            room.queue.publish(
                &format!("restore:{}:{}", endpoint.instance_id, endpoint.port),
                "restore",
                context,
                vec![route],
            )?;
        }
        room.record("room.revised", None, "restore", "restore", None, None)?;
        Ok(room)
    }
}
