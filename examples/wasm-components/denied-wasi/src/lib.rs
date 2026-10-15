//! Same exported WIT, but std::fs introduces real WASI filesystem imports.
#[allow(warnings)]
mod bindings;
use bindings::exports::workbench::component::computation::{ComponentError, Guest, Message};
struct Denied;
impl Guest for Denied {
    fn compute(input: Message) -> Result<Vec<Message>, ComponentError> {
        let text = std::fs::read_to_string(&input.payload_json).unwrap_or_default();
        Ok(vec![Message {
            port: "result".into(),
            payload_json: text,
        }])
    }
}
bindings::export!(Denied with_types_in bindings);
