#[allow(warnings)]
mod bindings;
use bindings::exports::workbench::component::computation::{ComponentError, Guest, Message};
struct Echo;
impl Guest for Echo {
    fn compute(input: Message) -> Result<Vec<Message>, ComponentError> {
        let value: serde_json::Value = serde_json::from_str(&input.payload_json).unwrap();
        let text = value["text"].as_str().unwrap();
        let mut output = Message {
            port: "result".into(),
            payload_json: serde_json::json!({"text": text.to_uppercase()}).to_string(),
        };
        match text {
            "guest-error" => {
                return Err(ComponentError {
                    code: "EXPECTED".into(),
                    message: "guest rejected request".into(),
                })
            }
            "trap" => core::arch::wasm32::unreachable(),
            "loop" => loop {
                core::hint::spin_loop();
            },
            "memory" => {
                core::arch::wasm32::memory_grow::<0>(256);
            }
            "oversize" => {
                output.payload_json = serde_json::json!({"text": "x".repeat(32768)}).to_string()
            }
            "invalid-json" => output.payload_json = "{".into(),
            "invalid-output" => output.payload_json = "{\"text\":17}".into(),
            "unknown-port" => output.port = "undeclared".into(),
            "many" => {
                return Ok((0..9)
                    .map(|_| Message {
                        port: "result".into(),
                        payload_json: "{\"text\":\"x\"}".into(),
                    })
                    .collect())
            }
            _ => {}
        }
        Ok(vec![output])
    }
}
bindings::export!(Echo with_types_in bindings);
