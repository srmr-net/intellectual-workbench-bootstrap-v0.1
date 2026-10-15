//! Versioned DTOs. These intentionally model the initial message boundary, not a complete ABI.
use serde::{Deserialize, Serialize};
pub const CONTRACT_VERSION: &str = "0.1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentRef {
    pub id: String,
    pub version: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Envelope {
    pub protocol_version: String,
    pub message_id: String,
    pub instance_id: String,
    pub port: String,
    pub revision: u64,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Edge {
    pub source: String,
    pub target: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn envelope_roundtrip() {
        let e = Envelope {
            protocol_version: CONTRACT_VERSION.into(),
            message_id: "m1".into(),
            instance_id: "i1".into(),
            port: "out".into(),
            revision: 3,
            payload: serde_json::json!({"x": 1}),
        };
        let encoded = serde_json::to_string(&e).unwrap();
        assert_eq!(serde_json::from_str::<Envelope>(&encoded).unwrap(), e);
    }
}
