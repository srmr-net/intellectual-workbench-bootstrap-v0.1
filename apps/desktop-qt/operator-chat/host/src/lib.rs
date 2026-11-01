//! Host-only contextual text conversation. No registry, kernel or tool authority.
// Compile the accepted policy/transport verbatim, rather than fork networking.
#[path = "../../../live-provider/host/src/policy.rs"]
pub mod policy;
#[path = "../../../live-provider/host/src/transport.rs"]
pub mod transport;
use policy::{
    contains_secret, CredentialSource, Failure, HostCredential, Secret, OPENROUTER_MODEL,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{sync::Arc, task::Poll, time::Instant};
use transport::{Exchange, Transport};

pub const MAX_USER: usize = 4096;
pub const MAX_ASSISTANT: usize = 16384;
pub const MAX_CONTEXT: usize = 48000;
pub const TIMEOUT_MS: u64 = 30000;
pub const MAX_TURNS: usize = 12;
const SYSTEM: &str = "You are Operator, a conversational assistant in Intellectual Workbench. Help the user think, clarify ideas and discuss tasks naturally. You have no tools or execution authority in this conversation. You cannot read files, access the registry, run code or start components. Do not claim to have done those things. Tool execution is a separate explicit host-validated workflow. Ask useful questions when needed; ordinary conversation does not require financial parameters. Reply in clear plain text.";

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    Configure {
        enabled: bool,
        #[serde(rename = "credentialPath")]
        credential_path: Option<String>,
    },
    New {},
    Select {
        #[serde(rename = "sessionId")]
        session_id: String,
    },
    Send {
        #[serde(rename = "sessionId")]
        session_id: String,
        text: String,
    },
    Cancel {},
    Quit {},
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: &'static str,
    pub text: String,
    pub status: &'static str,
    pub error: Option<Failure>,
}
struct Session {
    id: String,
    title: String,
    messages: Vec<Message>,
}
struct Pending {
    token: String,
    session: String,
    user: usize,
    started: Instant,
    exchange: Box<dyn Exchange>,
    secret: Secret,
    context_turns: usize,
}
pub struct ChatHost {
    transport: Arc<dyn Transport>,
    credential: Box<dyn CredentialSource>,
    enabled: bool,
    sessions: Vec<Session>,
    selected: String,
    pending: Option<Pending>,
    calls: u64,
    last_usage: Value,
}
impl ChatHost {
    pub fn new(transport: Arc<dyn Transport>, credential: Box<dyn CredentialSource>) -> Self {
        Self {
            transport,
            credential,
            enabled: false,
            sessions: vec![],
            selected: String::new(),
            pending: None,
            calls: 0,
            last_usage: Value::Null,
        }
    }
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }
    pub fn state(&self) -> Value {
        let session = self.sessions.iter().find(|s| s.id == self.selected);
        json!({"enabled":self.enabled,"available":self.transport.available(),"credentialAvailable":self.credential.load().is_ok(),"model":OPENROUTER_MODEL,"endpoint":policy::OPENROUTER_ENDPOINT,"busy":self.busy(),"calls":self.calls,"selected":self.selected,"sessions":self.sessions.iter().map(|s|json!({"id":s.id,"title":s.title})).collect::<Vec<_>>(),"messages":session.map(|s|&s.messages),"usage":self.last_usage,"contextTurnLimit":MAX_TURNS})
    }
    fn session_mut(&mut self, id: &str) -> &mut Session {
        self.sessions
            .iter_mut()
            .find(|s| s.id == id)
            .expect("host-owned session")
    }
    pub fn cancel(&mut self) {
        if let Some(p) = self.pending.take() {
            let user = &mut self.session_mut(&p.session).messages[p.user];
            user.status = "cancelled";
            user.error = Some(Failure::Cancelled);
            // Dropping the accepted Exchange kills and reaps the network child.
        }
    }
    pub fn handle(&mut self, token: &str, command: Command) -> Result<Value, Failure> {
        match command {
            Command::Configure {
                enabled,
                credential_path,
            } => {
                // Revoke first: even invalid replacement settings must stop inference.
                self.cancel();
                self.enabled = false;
                if credential_path
                    .as_ref()
                    .is_some_and(|p| p.len() > 4096 || !std::path::Path::new(p).is_absolute())
                {
                    return Err(Failure::NotConfigured);
                }
                self.enabled = enabled;
                self.credential = match credential_path {
                    Some(p) => Box::new(HostCredential::PrivateFile(p.into())),
                    None => Box::new(HostCredential::OpenRouterEnvironment),
                };
            }
            Command::New {} => {
                self.cancel();
                if self.sessions.len() >= 16 {
                    return Err(Failure::BudgetExceeded);
                }
                let id = format!("chat-{}", self.sessions.len() + 1);
                self.sessions.push(Session {
                    id: id.clone(),
                    title: "New conversation".into(),
                    messages: vec![],
                });
                self.selected = id;
            }
            Command::Select { session_id } => {
                self.cancel();
                if !self.sessions.iter().any(|s| s.id == session_id) {
                    return Err(Failure::SchemaMismatch);
                }
                self.selected = session_id;
            }
            Command::Cancel {} | Command::Quit {} => self.cancel(),
            Command::Send { session_id, text } => {
                self.start(token, session_id, text)?;
                return Ok(json!({"kind":"started","state":self.state()}));
            }
        }
        Ok(json!({"kind":"state","state":self.state()}))
    }
    fn start(&mut self, token: &str, session: String, text: String) -> Result<(), Failure> {
        if !self.enabled {
            return Err(Failure::ConsentRequired);
        }
        if self.busy() {
            return Err(Failure::BudgetExceeded);
        }
        if text.trim().is_empty() || text.len() > MAX_USER {
            return Err(Failure::BudgetExceeded);
        }
        if session != self.selected || !self.sessions.iter().any(|s| s.id == session) {
            return Err(Failure::SchemaMismatch);
        }
        if self.calls >= 256 || self.session_mut(&session).messages.len() >= 198 {
            return Err(Failure::BudgetExceeded);
        }
        // Reserve serialized space for the next user/assistant pair before
        // starting a request. JSON escaping can expand text sixfold. Keep the
        // selected transcript well below the worker's 4 MiB frame boundary.
        let existing = serde_json::to_vec(&self.session_mut(&session).messages)
            .map_err(|_| Failure::BudgetExceeded)?
            .len();
        let user_bytes = serde_json::to_vec(&text)
            .map_err(|_| Failure::BudgetExceeded)?
            .len();
        if existing + user_bytes + MAX_ASSISTANT * 6 + 2048 > 1_000_000 {
            return Err(Failure::BudgetExceeded);
        }
        let secret = self.credential.load()?;
        let (body, context_turns) = request(&self.session_mut(&session).messages, &text)?;
        if contains_secret(&body, &secret) {
            return Err(Failure::SchemaMismatch);
        }
        let exchange =
            self.transport
                .start(&body, secret.copy_for_transport(), 32768, TIMEOUT_MS)?;
        self.calls += 1;
        let s = self.session_mut(&session);
        if s.messages.is_empty() {
            s.title = text.chars().take(46).collect();
        }
        let user = s.messages.len();
        s.messages.push(Message {
            role: "user",
            text,
            status: "pending",
            error: None,
        });
        self.pending = Some(Pending {
            token: token.into(),
            session,
            user,
            started: Instant::now(),
            exchange,
            secret,
            context_turns,
        });
        Ok(())
    }
    pub fn poll(&mut self) -> Option<Value> {
        let p = self.pending.as_mut()?;
        let response = if p.started.elapsed().as_millis() >= u128::from(TIMEOUT_MS) {
            Err(Failure::Timeout)
        } else {
            match p.exchange.poll() {
                Poll::Pending => return None,
                Poll::Ready(r) => r,
            }
        };
        let p = self.pending.take().unwrap();
        let result = response.and_then(|r| {
            if contains_secret(&r.body, &p.secret) {
                return Err(Failure::SchemaMismatch);
            }
            parse(r.status, &r.body)
        });
        let elapsed = p.started.elapsed().as_millis() as u64;
        let session = self.session_mut(&p.session);
        match result {
            Ok((text, usage)) => {
                session.messages[p.user].status = "complete";
                session.messages.push(Message {
                    role: "assistant",
                    text,
                    status: "complete",
                    error: None,
                });
                self.last_usage =
                    json!({"elapsedMs":elapsed,"contextTurns":p.context_turns,"provider":usage});
                Some(json!({"kind":"completed","token":p.token,"ok":true,"state":self.state()}))
            }
            Err(code) => {
                session.messages[p.user].status = "failed";
                session.messages[p.user].error = Some(code);
                self.last_usage = json!({"elapsedMs":elapsed,"contextTurns":p.context_turns});
                Some(
                    json!({"kind":"error","token":p.token,"ok":false,"code":code,"state":self.state()}),
                )
            }
        }
    }
}
fn request(messages: &[Message], text: &str) -> Result<(Vec<u8>, usize), Failure> {
    // Only successful, host-owned pairs. Never accept role/history objects from Qt.
    let complete: Vec<_> = messages.iter().filter(|m| m.status == "complete").collect();
    let mut start = complete.len().saturating_sub(MAX_TURNS * 2);
    while start < complete.len()
        && complete[start..]
            .iter()
            .map(|m| m.text.len())
            .sum::<usize>()
            + text.len()
            > MAX_CONTEXT
    {
        start += 2;
    }
    let mut context = vec![json!({"role":"system","content":SYSTEM})];
    context.extend(
        complete[start..]
            .iter()
            .map(|m| json!({"role":m.role,"content":m.text})),
    );
    context.push(json!({"role":"user","content":text}));
    let body = serde_json::to_vec(&json!({"model":OPENROUTER_MODEL,"messages":context,"stream":false,"max_tokens":1024,"tools":[],"tool_choice":"none","reasoning":{"enabled":false,"exclude":true},"provider":{"only":["nvidia"],"allow_fallbacks":false,"require_parameters":true,"data_collection":"allow","max_price":{"prompt":0,"completion":0,"request":0}}})).map_err(|_|Failure::SchemaMismatch)?;
    if body.len() > 65536 {
        return Err(Failure::BudgetExceeded);
    }
    Ok((body, (complete.len() - start) / 2))
}
fn parse(status: u16, body: &[u8]) -> Result<(String, Value), Failure> {
    match status {
        401 | 403 => return Err(Failure::AuthFailed),
        429 => return Err(Failure::RateLimited),
        500..=599 => return Err(Failure::Unavailable),
        200 => {}
        _ => return Err(Failure::ProviderError),
    }
    if body.len() > 32768 {
        return Err(Failure::BudgetExceeded);
    }
    let v = wb_contracts::ui::strict_json(body).map_err(|_| Failure::MalformedResponse)?;
    if v.get("error").is_some() {
        return Err(Failure::ProviderError);
    }
    let choices = v["choices"]
        .as_array()
        .filter(|c| c.len() == 1)
        .ok_or(Failure::MalformedResponse)?;
    let choice = &choices[0];
    let m = &choice["message"];
    if m["refusal"].as_str().is_some_and(|s| !s.is_empty())
        || choice["finish_reason"] == "content_filter"
    {
        return Err(Failure::Refused);
    }
    if m.get("tool_calls")
        .is_some_and(|v| !v.is_null() && v.as_array().is_none_or(|a| !a.is_empty()))
        || m.get("function_call").is_some_and(|v| !v.is_null())
        || m["role"] != "assistant"
    {
        return Err(Failure::SchemaMismatch);
    }
    if choice["finish_reason"] == "length" {
        return Err(Failure::Truncated);
    }
    if choice["finish_reason"] != "stop" {
        return Err(Failure::MalformedResponse);
    }
    let text = m["content"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or(Failure::MalformedResponse)?;
    if text.len() > MAX_ASSISTANT {
        return Err(Failure::BudgetExceeded);
    }
    // No raw response, reasoning, IDs, arbitrary metadata or model text in telemetry.
    let mut usage = serde_json::Map::new();
    for key in ["prompt_tokens", "completion_tokens", "total_tokens"] {
        if let Some(n) = v["usage"][key].as_u64().filter(|n| *n <= 1_000_000_000) {
            usage.insert(key.into(), json!(n));
        }
    }
    if let Some(model) = v["model"].as_str().filter(|s| {
        s.len() <= 128
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/:-._".contains(&b))
    }) {
        usage.insert("returnedModel".into(), json!(model));
    }
    Ok((text.into(), Value::Object(usage)))
}

/// Bounded duplicate-key-rejecting protocol input, shared with accepted contracts.
pub fn strict_request(body: &[u8]) -> Result<Value, Failure> {
    if body.len() > 32768 {
        return Err(Failure::BudgetExceeded);
    }
    wb_contracts::ui::strict_json(body).map_err(|_| Failure::MalformedResponse)
}
#[cfg(test)]
mod tests;
