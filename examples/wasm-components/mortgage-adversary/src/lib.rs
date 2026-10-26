//! Explicit adversarial TEST guest; never used as the mortgage implementation.
#[allow(warnings)]
mod bindings;
use bindings::exports::workbench::component::computation::{ComponentError, Guest, Message};
struct Adversary;
impl Guest for Adversary {
    fn compute(message: Message) -> Result<Vec<Message>, ComponentError> {
        let value: serde_json::Value = serde_json::from_str(&message.payload_json).unwrap();
        let mode = value["principal"].as_str().unwrap();
        let good = || Message {
            port: "payment".into(),
            payload_json: r#"{"monthly_payment":"123.45"}"#.into(),
        };
        match mode {
            "2.00" => Ok(vec![
                good(),
                Message {
                    port: "undeclared".into(),
                    payload_json: "{}".into(),
                },
            ]),
            "3.00" => Ok(vec![
                good(),
                Message {
                    port: "payment".into(),
                    payload_json: "{".into(),
                },
            ]),
            "4.00" => Ok(vec![Message {
                port: "payment".into(),
                payload_json: r#"{"monthly_payment":7}"#.into(),
            }]),
            "8.00" => Ok(vec![Message {
                port: "payment".into(),
                payload_json: "{\"monthly_payment\":\"1.00\\n\"}".into(),
            }]),
            "5.00" => core::arch::wasm32::unreachable(),
            "6.00" => loop {
                core::hint::spin_loop();
            },
            "7.00" => Ok(vec![good(), good()]),
            _ => Ok(vec![good()]),
        }
    }
}
bindings::export!(Adversary with_types_in bindings);
