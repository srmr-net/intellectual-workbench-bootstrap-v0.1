//! Pure compiled observation fixture for canonical downstream bus delivery.
#[allow(warnings)]
mod bindings;
use bindings::exports::workbench::component::computation::{ComponentError, Guest, Message};
struct Observer;
impl Guest for Observer {
    fn compute(message: Message) -> Result<Vec<Message>, ComponentError> {
        if message.port != "payment" {
            return Err(ComponentError {
                code: "INVALID_PORT".into(),
                message: "Expected payment".into(),
            });
        }
        Ok(vec![Message {
            port: "observed".into(),
            payload_json: message.payload_json,
        }])
    }
}
bindings::export!(Observer with_types_in bindings);
