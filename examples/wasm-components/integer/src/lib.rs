#[allow(warnings)]
mod bindings;
use bindings::exports::workbench::component::computation::{ComponentError, Guest, Message};
struct Integer;
impl Guest for Integer {
    fn compute(input: Message) -> Result<Vec<Message>, ComponentError> {
        let value: serde_json::Value = serde_json::from_str(&input.payload_json).unwrap();
        let doubled = value["value"]
            .as_i64()
            .unwrap()
            .checked_mul(2)
            .ok_or_else(|| ComponentError {
                code: "OVERFLOW".into(),
                message: "integer multiplication overflow".into(),
            })?;
        Ok(vec![Message {
            port: "result".into(),
            payload_json: serde_json::json!({"value": doubled}).to_string(),
        }])
    }
}
bindings::export!(Integer with_types_in bindings);
