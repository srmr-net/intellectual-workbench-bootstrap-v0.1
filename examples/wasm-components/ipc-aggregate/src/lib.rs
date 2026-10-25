//! ADR-002 aggregate-input fixture; uses exactly the accepted import-free world.
#[allow(warnings)]
mod bindings;
use bindings::exports::workbench::component::computation::{ComponentError, Guest, Message};
struct Aggregate;
impl Guest for Aggregate {
    fn compute(input: Message) -> Result<Vec<Message>, ComponentError> {
        let input: serde_json::Value = serde_json::from_str(&input.payload_json).unwrap();
        let value = input["value"].as_i64().unwrap();
        let add = input.get("add").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let value = value.checked_add(add).ok_or_else(|| ComponentError {code:"OVERFLOW".into(),message:"integer overflow".into()})?;
        let message=|port:&str,payload:serde_json::Value|Message{port:port.into(),payload_json:payload.to_string()};
        match input.get("mode").and_then(serde_json::Value::as_str).unwrap_or("sum") {
            "trap" => core::arch::wasm32::unreachable(),
            "loop" => loop {core::hint::spin_loop();},
            "error" => Err(ComponentError{code:"EXPECTED".into(),message:"private guest diagnostics".into()}),
            "bad-batch" => Ok(vec![message("result",serde_json::json!({"value":value})),message("extra",serde_json::json!({"value":"invalid"}))]),
            "forge" => Ok(vec![message("result",serde_json::json!({"value":value,"source":{"instanceId":"victim","port":"secret"},"destination":{"instanceId":"other-room","port":"secret"},"sequence":777}))]),
            _ => Ok(vec![message("result",serde_json::json!({"value":value}))]),
        }
    }
}
bindings::export!(Aggregate with_types_in bindings);
