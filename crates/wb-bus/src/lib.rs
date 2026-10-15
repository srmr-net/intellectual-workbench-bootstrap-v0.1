//! In-process envelope version checks. Routing and bounded queues are ADR-002 work.
use wb_contracts::Envelope;
pub const CONTRACT_VERSION: &str = "0.1";
pub fn validate_envelope(message: &Envelope) -> Result<(), &'static str> {
    if message.protocol_version != CONTRACT_VERSION {
        return Err("UNSUPPORTED_VERSION");
    }
    if message.instance_id.is_empty() || message.port.is_empty() || message.message_id.is_empty() {
        return Err("INVALID_ENVELOPE");
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrong_protocol_is_rejected() {
        let e = Envelope {
            protocol_version: "9.0".into(),
            message_id: "m".into(),
            instance_id: "i".into(),
            port: "p".into(),
            revision: 0,
            payload: serde_json::Value::Null,
        };
        assert_eq!(validate_envelope(&e), Err("UNSUPPORTED_VERSION"));
    }
}
