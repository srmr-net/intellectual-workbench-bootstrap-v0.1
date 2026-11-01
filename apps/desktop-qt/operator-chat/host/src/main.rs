use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io::{self, Read, Write},
    sync::{mpsc, Arc},
    time::Duration,
};
use wb_operator_chat::{
    policy::{Failure, HostCredential, Kind},
    transport::{TlsTransport, Transport},
    ChatHost, Command,
};
const FRAME: usize = 32768;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    protocol: String,
    token: String,
    command: Command,
}
fn write(value: Value) -> io::Result<()> {
    let body = serde_json::to_vec(&value)?;
    if body.len() > 4 * 1024 * 1024 {
        return Err(io::Error::other("frame quota"));
    }
    let mut out = io::stdout().lock();
    out.write_all(&(body.len() as u32).to_be_bytes())?;
    out.write_all(&body)?;
    out.flush()
}
fn main() {
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::setrlimit(libc::RLIMIT_CORE, &limit) != 0 {
            std::process::exit(1);
        }
    }
    if run().is_err() {
        std::process::exit(1);
    }
}
fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let transport: Arc<dyn Transport> = match args.as_slice() {
        [_] => Arc::new(
            TlsTransport::for_provider("/usr/bin/python3".into(), Kind::OpenrouterChat)
                .map_err(|_| io::Error::other("transport"))?,
        ),
        #[cfg(feature = "test-transport")]
        [_, flag] if flag == "--fixture" => {
            std::env::set_var("OPENROUTER_API_KEY", "chat-synthetic-fixture-only");
            Arc::new(FixtureTransport)
        }
        _ => return Err(io::Error::other("arguments")),
    };
    let credential = match std::env::var_os("WB_LLM_SECRET_FILE") {
        Some(p) => HostCredential::PrivateFile(p.into()),
        None => HostCredential::OpenRouterEnvironment,
    };
    let mut host = ChatHost::new(transport, Box::new(credential));
    let (tx, rx) = mpsc::sync_channel::<Result<Vec<u8>, ()>>(16);
    std::thread::spawn(move || {
        let mut input = io::stdin().lock();
        loop {
            let mut prefix = [0; 4];
            if input.read_exact(&mut prefix).is_err() {
                break;
            }
            let n = u32::from_be_bytes(prefix) as usize;
            if n == 0 || n > FRAME {
                let _ = tx.send(Err(()));
                break;
            }
            let mut body = vec![0; n];
            if input.read_exact(&mut body).is_err() {
                let _ = tx.send(Err(()));
                break;
            }
            if tx.send(Ok(body)).is_err() {
                break;
            }
        }
    });
    write(json!({"protocol":"chat.1","kind":"ready","state":host.state()}))?;
    loop {
        match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(Ok(body)) => {
                let parsed = wb_operator_chat::strict_request(&body).and_then(|v| {
                    serde_json::from_value::<Request>(v).map_err(|_| Failure::MalformedResponse)
                });
                let r = match parsed {
                    Ok(r)
                        if r.protocol == "chat.1"
                            && !r.token.is_empty()
                            && r.token.len() <= 64
                            && r.token
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'-') =>
                    {
                        r
                    }
                    _ => {
                        write(json!({"protocol":"chat.1","kind":"fatal","code":"INVALID_FRAME"}))?;
                        return Ok(());
                    }
                };
                let quit = matches!(r.command, Command::Quit {});
                let mut response = match host.handle(&r.token, r.command) {
                    Ok(v) => v,
                    Err(code) => {
                        json!({"kind":"error","ok":false,"code":code,"state":host.state()})
                    }
                };
                response["protocol"] = json!("chat.1");
                response["token"] = json!(r.token);
                write(response)?;
                if quit {
                    return Ok(());
                }
            }
            Ok(Err(())) => {
                host.cancel();
                write(json!({"protocol":"chat.1","kind":"fatal","code":"INVALID_FRAME"}))?;
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                host.cancel();
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(mut response) = host.poll() {
            response["protocol"] = json!("chat.1");
            write(response)?;
        }
    }
}
#[cfg(feature = "test-transport")]
struct FixtureTransport;
#[cfg(feature = "test-transport")]
impl Transport for FixtureTransport {
    fn start(
        &self,
        body: &[u8],
        key: wb_operator_chat::policy::Secret,
        max: usize,
        timeout: u64,
    ) -> Result<Box<dyn wb_operator_chat::transport::Exchange>, Failure> {
        let v: Value = serde_json::from_slice(body).map_err(|_| Failure::MalformedResponse)?;
        let messages = v["messages"].as_array().unwrap();
        let users: Vec<_> = messages.iter().filter(|m| m["role"] == "user").collect();
        let text = if users.len() > 1 {
            format!(
                "You previously said: {}. We can continue that conversation.",
                users[0]["content"].as_str().unwrap()
            )
        } else {
            "This is the offline chat fixture. What would you like to work on?".into()
        };
        let transport=wb_operator_chat::transport::MockTransport::new(vec![wb_operator_chat::transport::MockReply{status:if users.last().unwrap()["content"] == "fixture:rate-limit" {429} else {200},body:serde_json::to_vec(&json!({"model":"fixture","choices":[{"finish_reason":"stop","message":{"role":"assistant","content":text}}],"usage":{"prompt_tokens":10,"completion_tokens":20}})).unwrap(),delay:Duration::from_millis(200),retry_after_ms:None,failure:None}]);
        transport.start(body, key, max, timeout)
    }
}
