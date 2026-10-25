//! ADR-002 proposed process-local contracts. No execution authority is serialized.
use crate::ComponentRef;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidManifest,
    IncompatiblePorts,
    SchemaViolation,
    CapabilityDenied,
    ComponentNotFound,
    ComponentTrap,
    CyclicDependency,
    StateConflict,
    RestoreFailed,
    UnsupportedVersion,
    QueueFull,
    StaleResult,
    GuestError,
    ResourceLimit,
    Cancelled,
    InvalidTransition,
    DuplicateId,
    InvalidEnvelope,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostError {
    pub code: ErrorCode,
    pub message: String,
    pub source: String,
    pub correlation_id: Option<String>,
    pub detail: Value,
}
impl HostError {
    pub fn new(code: ErrorCode, source: &str) -> Self {
        Self {
            code,
            message: format!("{code:?}"),
            source: source.into(),
            correlation_id: None,
            detail: Value::Null,
        }
    }
    pub fn correlated(mut self, id: &str) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}
impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.source, self.code)
    }
}
impl std::error::Error for HostError {}
pub type Result<T> = std::result::Result<T, HostError>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Endpoint {
    pub instance_id: String,
    pub port: String,
}
impl Endpoint {
    pub fn new(instance: &str, port: &str) -> Self {
        Self {
            instance_id: instance.into(),
            port: port.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventEnvelope {
    pub protocol_version: String,
    pub message_id: String,
    pub room_id: String,
    pub source: Endpoint,
    pub destination: Endpoint,
    pub correlation_id: String,
    pub sequence: u64,
    pub schema_id: String,
    pub payload: Value,
}
impl EventEnvelope {
    pub fn validate(&self) -> Result<()> {
        if self.protocol_version != crate::CONTRACT_VERSION {
            return Err(HostError::new(ErrorCode::UnsupportedVersion, "contracts"));
        }
        let fields = [
            &self.message_id,
            &self.room_id,
            &self.source.instance_id,
            &self.source.port,
            &self.destination.instance_id,
            &self.destination.port,
            &self.correlation_id,
            &self.schema_id,
        ];
        if self.sequence == 0 || fields.iter().any(|s| s.is_empty() || s.len() > 512) {
            return Err(HostError::new(ErrorCode::InvalidEnvelope, "contracts"));
        }
        Ok(())
    }
    pub fn parse(json: &str) -> Result<Self> {
        let e: Self = serde_json::from_str(json)
            .map_err(|_| HostError::new(ErrorCode::InvalidEnvelope, "contracts"))?;
        e.validate()?;
        Ok(e)
    }
}
/// Host-only execution metadata: deliberately not Serialize/Deserialize.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchContext {
    pub room_revision: u64,
    pub execution_generation: u64,
    pub invocation_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Connection {
    pub from: Endpoint,
    pub to: Endpoint,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstanceSpec {
    pub instance_id: String,
    pub component: ComponentRef,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiReference {
    pub definition_ref: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PermissionRequest {
    pub component_instance: String,
    pub capability: String,
    pub scope: Value,
    pub duration: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompositionPlan {
    pub plan_version: String,
    pub room_id: String,
    pub expected_room_revision: u64,
    pub instances: Vec<InstanceSpec>,
    pub connections: Vec<Connection>,
    pub ui: UiReference,
    pub requested_permissions: Vec<PermissionRequest>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotComponent {
    pub instance_id: String,
    pub id: String,
    pub version: String,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomSnapshot {
    pub room_format_version: String,
    pub room_id: String,
    pub revision: u64,
    pub name: String,
    pub components: Vec<SnapshotComponent>,
    pub connections: Vec<Connection>,
    pub state: BTreeMap<String, Value>,
    pub ui: UiReference,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstanceStatus {
    Declared,
    Prepared,
    Running,
    Stopped,
    Failed,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditEvent {
    pub kind: String,
    pub room_id: String,
    pub instance_id: Option<String>,
    pub revision: u64,
    pub sequence: u64,
    pub actor: String,
    pub correlation_id: String,
    pub error_code: Option<ErrorCode>,
    pub elapsed_micros: Option<u64>,
}
