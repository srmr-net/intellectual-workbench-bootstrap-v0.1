//! Trusted bundle cache and execution approvals, not a package installer or capability broker.
use crate::room::Room;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use wb_bus::ipc::Sequence;
use wb_contracts::{
    ipc::{ErrorCode, HostError, Result, UiReference},
    ComponentRef,
};
use wb_wasm_host::{Failure, Host, Limits, Registered};
pub(crate) fn error(code: ErrorCode) -> HostError {
    HostError::new(code, "kernel")
}
pub(crate) fn key(pin: &ComponentRef) -> String {
    format!("{}@{}#{}", pin.id, pin.version, pin.sha256)
}
pub(crate) fn host_failure(f: Failure) -> HostError {
    let code = match f {
        Failure::Manifest(_) | Failure::Digest | Failure::World => ErrorCode::InvalidManifest,
        Failure::DeniedImport => ErrorCode::CapabilityDenied,
        Failure::Schema(_) | Failure::Port { .. } => ErrorCode::SchemaViolation,
        Failure::Guest { .. } => ErrorCode::GuestError,
        Failure::Trap => ErrorCode::ComponentTrap,
        Failure::Budget(_) => ErrorCode::ResourceLimit,
        Failure::Host(_) => ErrorCode::ComponentTrap,
    };
    // Keep guest diagnostics out of audit/user payload logging by default.
    let mut e = error(code);
    match f {
        Failure::Guest { code, message } => {
            e.detail = serde_json::json!({"guestCode":code,"guestMessage":message})
        }
        Failure::Budget(budget) => e.detail = serde_json::json!({"budget":budget}),
        _ => {}
    }
    e
}
pub(crate) struct Port {
    pub schema: Value,
    pub validator: jsonschema::Validator,
    pub schema_id: String,
    pub semantic: String,
    pub unit: Option<String>,
}
pub(crate) struct Artifact {
    pub pin: ComponentRef,
    pub registered: Registered,
    pub manifest: Value,
    pub bytes: Arc<[u8]>,
    pub schemas: BTreeMap<String, Value>,
    pub inputs: BTreeMap<String, Port>,
    pub outputs: BTreeMap<String, Port>,
}
impl Artifact {
    pub fn validate(&self, input: bool, port: &str, value: &Value) -> Result<()> {
        let ports = if input { &self.inputs } else { &self.outputs };
        let p = ports
            .get(port)
            .ok_or_else(|| error(ErrorCode::IncompatiblePorts))?;
        if value.to_string().len() > 16 * 1024 {
            return Err(error(ErrorCode::ResourceLimit));
        }
        p.validator
            .validate(value)
            .map_err(|_| error(ErrorCode::SchemaViolation))
    }
}
pub(crate) struct Pool {
    used: AtomicUsize,
    max: usize,
}
pub(crate) struct Lease(Arc<Pool>);
impl Pool {
    fn new(max: usize) -> Arc<Self> {
        Arc::new(Self {
            used: AtomicUsize::new(0),
            max,
        })
    }
    pub fn reserve(self: &Arc<Self>) -> Result<Arc<Lease>> {
        self.used
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                if n < self.max {
                    Some(n + 1)
                } else {
                    None
                }
            })
            .map_err(|_| error(ErrorCode::ResourceLimit))?;
        Ok(Arc::new(Lease(self.clone())))
    }
    pub fn used(&self) -> usize {
        self.used.load(Ordering::SeqCst)
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.used.fetch_sub(1, Ordering::SeqCst);
    }
}
static NEXT: AtomicU64 = AtomicU64::new(1);
pub(crate) struct Services {
    pub host: Host,
    pub packages: Mutex<BTreeMap<String, Arc<Artifact>>>,
    approvals: Mutex<BTreeSet<(String, String, String)>>,
    pub instances: Arc<Pool>,
    pub invocations: Arc<Pool>,
    sequences: Mutex<BTreeMap<String, Arc<Sequence>>>,
    pub rooms: Mutex<BTreeSet<String>>,
}
impl Services {
    pub fn token(&self) -> Result<u64> {
        NEXT.try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_add(1))
            .map_err(|_| error(ErrorCode::ResourceLimit))
    }
    pub fn resolve(&self, pin: &ComponentRef) -> Result<Arc<Artifact>> {
        self.packages
            .lock()
            .unwrap()
            .get(&key(pin))
            .cloned()
            .ok_or_else(|| error(ErrorCode::ComponentNotFound))
    }
    pub fn approved(&self, room: &str, id: &str, pin: &ComponentRef) -> Result<()> {
        if self
            .approvals
            .lock()
            .unwrap()
            .contains(&(room.into(), id.into(), key(pin)))
        {
            Ok(())
        } else {
            Err(error(ErrorCode::CapabilityDenied))
        }
    }
    pub fn clock(&self, room: &str) -> Arc<Sequence> {
        self.sequences
            .lock()
            .unwrap()
            .entry(room.into())
            .or_default()
            .clone()
    }
}
#[derive(Clone)]
pub struct Kernel {
    pub(crate) services: Arc<Services>,
}
impl Kernel {
    pub fn new(limits: Limits, max_instances: usize, max_invocations: usize) -> Result<Self> {
        if max_instances == 0 || max_invocations == 0 {
            return Err(error(ErrorCode::ResourceLimit));
        }
        Ok(Self {
            services: Arc::new(Services {
                host: Host::new(limits).map_err(host_failure)?,
                packages: Mutex::new(BTreeMap::new()),
                approvals: Mutex::new(BTreeSet::new()),
                instances: Pool::new(max_instances),
                invocations: Pool::new(max_invocations),
                sequences: Mutex::new(BTreeMap::new()),
                rooms: Mutex::new(BTreeSet::new()),
            }),
        })
    }
    /// Caller has already resolved/verified package paths and RDF consistency.
    /// This cache additionally verifies manifest/digest/ABI without running code.
    pub fn cache_verified_bundle(
        &self,
        manifest: Value,
        bytes: Vec<u8>,
        schemas: BTreeMap<String, Value>,
    ) -> Result<ComponentRef> {
        let registered = self
            .services
            .host
            .register(&manifest, &bytes, &schemas)
            .map_err(host_failure)?;
        let pin = ComponentRef {
            id: manifest["id"].as_str().unwrap().into(),
            version: manifest["version"].as_str().unwrap().into(),
            sha256: manifest["runtime"]["sha256"].as_str().unwrap().into(),
        };
        let ports = |direction: &str| -> Result<BTreeMap<String, Port>> {
            let mut result = BTreeMap::new();
            for p in manifest["ports"][direction].as_array().unwrap() {
                let path = p["schema"].as_str().unwrap();
                let schema = schemas[path].clone();
                let validator = jsonschema::options()
                    .with_draft(jsonschema::Draft::Draft202012)
                    .build(&schema)
                    .map_err(|_| error(ErrorCode::InvalidManifest))?;
                result.insert(
                    p["name"].as_str().unwrap().into(),
                    Port {
                        schema_id: format!("{}:{path}", key(&pin)),
                        schema,
                        validator,
                        semantic: p["semanticType"].as_str().unwrap().into(),
                        unit: p.get("unit").and_then(Value::as_str).map(str::to_owned),
                    },
                );
            }
            Ok(result)
        };
        let inputs = ports("inputs")?;
        let outputs = ports("outputs")?;
        let mut packages = self.services.packages.lock().unwrap();
        if packages.values().any(|a| {
            a.pin.id == pin.id && a.pin.version == pin.version && a.pin.sha256 != pin.sha256
        }) {
            return Err(error(ErrorCode::InvalidManifest));
        }
        packages.entry(key(&pin)).or_insert_with(|| {
            Arc::new(Artifact {
                pin: pin.clone(),
                registered,
                manifest,
                bytes: bytes.into(),
                schemas,
                inputs,
                outputs,
            })
        });
        Ok(pin)
    }
    /// Explicit trusted host policy, scoped to room, logical instance and exact pin.
    /// This approves pure execution only, never a native capability or guest import.
    pub fn approve_execution(&self, room: &str, id: &str, pin: &ComponentRef) -> Result<()> {
        self.services.resolve(pin)?;
        self.services
            .approvals
            .lock()
            .unwrap()
            .insert((room.into(), id.into(), key(pin)));
        Ok(())
    }
    pub fn revoke_execution(&self, room: &str, id: &str, pin: &ComponentRef) {
        self.services
            .approvals
            .lock()
            .unwrap()
            .remove(&(room.into(), id.into(), key(pin)));
    }
    pub fn reserved_instances(&self) -> usize {
        self.services.instances.used()
    }
    pub fn in_flight(&self) -> usize {
        self.services.invocations.used()
    }
    pub fn create_room(
        &self,
        id: &str,
        name: &str,
        ui: UiReference,
        capacity: usize,
    ) -> Result<Room> {
        Room::new(self.services.clone(), id, name, ui, capacity)
    }
}
