//! ADR-001 validation adapter. No WASI, ambient imports or kernel integration.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, ResourceLimiter, Store, StoreLimits, StoreLimitsBuilder, Trap,
};

wasmtime::component::bindgen!({ path: "../../wit", world: "compute-component" });
use exports::workbench::component::computation::ComponentError;
pub use exports::workbench::component::computation::Message;
pub const CONTRACT_VERSION: &str = "0.1";

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("invalid manifest: {0}")]
    Manifest(String),
    #[error("artifact digest mismatch")]
    Digest,
    #[error("invalid WIT component/world")]
    World,
    #[error("component imports are forbidden")]
    DeniedImport,
    #[error("schema violation: {0}")]
    Schema(String),
    #[error("undeclared {direction} port: {port}")]
    Port {
        direction: &'static str,
        port: String,
    },
    #[error("guest error {code}: {message}")]
    Guest { code: String, message: String },
    #[error("component trapped")]
    Trap,
    #[error("resource budget exceeded: {0}")]
    Budget(&'static str),
    #[error("host configuration: {0}")]
    Host(String),
}

/// Spike policy, not a recommended production numerical workload policy.
#[derive(Clone, Copy)]
pub struct Limits {
    pub memory_bytes: usize,
    pub fuel: u64,
    /// 10ms engine ticks; scheduling can delay interruption.
    pub epoch_ticks: u64,
    pub payload_bytes: usize,
    pub result_count: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            memory_bytes: 8 * 1024 * 1024,
            fuel: 2_000_000,
            epoch_ticks: 100,
            payload_bytes: 16 * 1024,
            result_count: 8,
        }
    }
}

struct Budget {
    limits: StoreLimits,
    exceeded: bool,
}
impl ResourceLimiter for Budget {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let result = self.limits.memory_growing(current, desired, maximum);
        if !matches!(result, Ok(true)) {
            self.exceeded = true;
        }
        result
    }
    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let result = self.limits.table_growing(current, desired, maximum);
        if !matches!(result, Ok(true)) {
            self.exceeded = true;
        }
        result
    }
    fn instances(&self) -> usize {
        8
    }
    fn tables(&self) -> usize {
        2
    }
    fn memories(&self) -> usize {
        1
    }
}

/// Engine-wide ticker. Drop signals and joins it, including after failed calls.
struct Ticker {
    stop: Arc<(Mutex<bool>, Condvar)>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Ticker {
    fn new(engine: Engine) -> Result<Self, Failure> {
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let signal = stop.clone();
        let worker = thread::Builder::new()
            .name("wb-epoch".into())
            .spawn(move || {
                let (lock, wake) = &*signal;
                let mut stopped = lock.lock().unwrap();
                while !*stopped {
                    let (next, _) = wake
                        .wait_timeout(stopped, Duration::from_millis(10))
                        .unwrap();
                    stopped = next;
                    if !*stopped {
                        engine.increment_epoch();
                    }
                }
            })
            .map_err(|e| Failure::Host(e.to_string()))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}
impl Drop for Ticker {
    fn drop(&mut self) {
        let (lock, wake) = &*self.stop;
        *lock.lock().unwrap() = true;
        wake.notify_one();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Construct once for a validation run. All compiled artifacts belong to this engine.
pub struct Host {
    engine: Engine,
    limits: Limits,
    _ticker: Ticker,
}

/// Registered manifest declarations and compiled artifact; no caller-supplied ports on call.
pub struct Registered {
    component: Component,
    inputs: BTreeMap<String, jsonschema::Validator>,
    outputs: BTreeMap<String, jsonschema::Validator>,
}

impl Host {
    pub fn new(limits: Limits) -> Result<Self, Failure> {
        if limits.memory_bytes == 0
            || limits.fuel == 0
            || limits.epoch_ticks == 0
            || limits.payload_bytes == 0
            || limits.result_count == 0
        {
            return Err(Failure::Host("budgets must be positive".into()));
        }
        let mut config = Config::new();
        config
            .wasm_component_model(true)
            .consume_fuel(true)
            .epoch_interruption(true);
        let engine = Engine::new(&config).map_err(|e| Failure::Host(e.to_string()))?;
        let ticker = Ticker::new(engine.clone())?;
        Ok(Self {
            engine,
            limits,
            _ticker: ticker,
        })
    }

    /// Trusted caller supplies manifest JSON, raw artifact bytes and schema documents
    /// resolved from an already verified bundle. This is not a filesystem installer.
    pub fn register(
        &self,
        manifest: &Value,
        bytes: &[u8],
        schemas: &BTreeMap<String, Value>,
    ) -> Result<Registered, Failure> {
        let canonical: Value =
            serde_json::from_str(include_str!("../../../schemas/component.schema.json")).unwrap();
        let validator =
            jsonschema::validator_for(&canonical).map_err(|e| Failure::Manifest(e.to_string()))?;
        validator
            .validate(manifest)
            .map_err(|e| Failure::Manifest(e.to_string()))?;
        if !manifest["permissions"].as_array().unwrap().is_empty() {
            return Err(Failure::DeniedImport);
        }
        if bytes.len() > 2 * 1024 * 1024 {
            return Err(Failure::Budget("artifact bytes"));
        }
        if format!("{:x}", Sha256::digest(bytes)) != manifest["runtime"]["sha256"].as_str().unwrap()
        {
            return Err(Failure::Digest);
        }
        fn ports(
            manifest: &Value,
            direction: &str,
            schemas: &BTreeMap<String, Value>,
        ) -> Result<BTreeMap<String, jsonschema::Validator>, Failure> {
            let mut result = BTreeMap::new();
            for port in manifest["ports"][direction].as_array().unwrap() {
                let name = port["name"].as_str().unwrap();
                let path = port["schema"].as_str().unwrap();
                let schema = schemas
                    .get(path)
                    .ok_or_else(|| Failure::Manifest(format!("missing schema {path}")))?;
                let validator = jsonschema::options()
                    .with_draft(jsonschema::Draft::Draft202012)
                    .build(schema)
                    .map_err(|e| Failure::Manifest(e.to_string()))?;
                if result.insert(name.to_owned(), validator).is_some() {
                    return Err(Failure::Manifest("duplicate port".into()));
                }
            }
            Ok(result)
        }
        if !bytes.starts_with(b"\0asm\x0d\0\x01\0") {
            return Err(Failure::World);
        }
        let inputs = ports(manifest, "inputs", schemas)?;
        let outputs = ports(manifest, "outputs", schemas)?;
        let component = Component::new(&self.engine, bytes).map_err(|_| Failure::World)?;
        if component
            .component_type()
            .imports(&self.engine)
            .next()
            .is_some()
        {
            return Err(Failure::DeniedImport);
        }
        // Generated world typechecking happens before any guest starts.
        let linker = Linker::<Budget>::new(&self.engine);
        let pre = linker
            .instantiate_pre(&component)
            .map_err(|_| Failure::World)?;
        // Interface export indices alone do not check function signatures in
        // Wasmtime 49 bindgen. Typecheck without instantiating guest code.
        let interface = component
            .get_export_index(None, "workbench:component/computation@0.1.0")
            .ok_or(Failure::World)?;
        let (item, _) = component
            .get_export(Some(&interface), "compute")
            .ok_or(Failure::World)?;
        let wasmtime::component::types::ComponentItem::ComponentFunc(function) = item else {
            return Err(Failure::World);
        };
        function
            .typecheck::<(Message,), (Result<Vec<Message>, ComponentError>,)>(&pre.instance_type())
            .map_err(|_| Failure::World)?;
        ComputeComponentPre::new(pre).map_err(|_| Failure::World)?;
        Ok(Registered {
            component,
            inputs,
            outputs,
        })
    }

    fn check_message(
        &self,
        validators: &BTreeMap<String, jsonschema::Validator>,
        message: &Message,
        direction: &'static str,
    ) -> Result<(), Failure> {
        if message.payload_json.len() > self.limits.payload_bytes || message.port.len() > 64 {
            return Err(Failure::Budget("message bytes"));
        }
        let validator = validators.get(&message.port).ok_or_else(|| Failure::Port {
            direction,
            port: message.port.clone(),
        })?;
        let payload: Value = serde_json::from_str(&message.payload_json)
            .map_err(|e| Failure::Schema(e.to_string()))?;
        validator
            .validate(&payload)
            .map_err(|e| Failure::Schema(e.to_string()))
    }

    /// Each pure event uses a separate Store; failures cannot poison another guest.
    /// No results are returned until the entire batch passes validation.
    pub fn call(
        &self,
        registered: &Registered,
        message: &Message,
    ) -> Result<Vec<Message>, Failure> {
        if !Engine::same(&self.engine, registered.component.engine()) {
            return Err(Failure::Host("artifact belongs to another engine".into()));
        }
        self.check_message(&registered.inputs, message, "input")?;
        let budget = Budget {
            limits: StoreLimitsBuilder::new()
                .memory_size(self.limits.memory_bytes)
                .table_elements(1024)
                .trap_on_grow_failure(true)
                .build(),
            exceeded: false,
        };
        let mut store = Store::new(&self.engine, budget);
        store.limiter(|budget| budget);
        store
            .set_fuel(self.limits.fuel)
            .map_err(|e| Failure::Host(e.to_string()))?;
        store.set_epoch_deadline(self.limits.epoch_ticks);
        let classify = |error: wasmtime::Error, store: &Store<Budget>| {
            if store.data().exceeded {
                Failure::Budget("memory/table")
            } else if error
                .chain()
                .any(|cause| cause.to_string().starts_with("resource limit exceeded:"))
            {
                // Wasmtime 49 exposes count-cap failures as textual errors,
                // not Trap variants. Regression fixtures pin this mapping.
                Failure::Budget("instance resources")
            } else {
                match error.downcast_ref::<Trap>() {
                    Some(Trap::OutOfFuel) => Failure::Budget("fuel"),
                    Some(Trap::Interrupt) => Failure::Budget("epoch deadline"),
                    _ => Failure::Trap,
                }
            }
        };
        let linker = Linker::<Budget>::new(&self.engine);
        let instance = ComputeComponent::instantiate(&mut store, &registered.component, &linker)
            .map_err(|e| classify(e, &store))?;
        let result = instance
            .workbench_component_computation()
            .call_compute(&mut store, message)
            .map_err(|e| classify(e, &store))?;
        // bindgen's typed call performs canonical post-return cleanup.
        let outputs = result.map_err(|e| {
            if e.code.len() + e.message.len() > self.limits.payload_bytes {
                Failure::Budget("guest error bytes")
            } else {
                Failure::Guest {
                    code: e.code,
                    message: e.message,
                }
            }
        })?;
        if outputs.len() > self.limits.result_count {
            return Err(Failure::Budget("result count"));
        }
        let mut total = 0usize;
        for output in &outputs {
            total = total.saturating_add(output.payload_json.len() + output.port.len());
            if total > self.limits.payload_bytes {
                return Err(Failure::Budget("result bytes"));
            }
            self.check_message(&registered.outputs, output, "output")?;
        }
        Ok(outputs)
    }
}
