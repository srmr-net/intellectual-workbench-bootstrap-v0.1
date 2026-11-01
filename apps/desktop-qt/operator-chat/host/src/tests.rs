use super::*;
use std::{sync::atomic::Ordering, time::Duration};
use transport::{MockReply, MockTransport};
struct Credential;
impl CredentialSource for Credential {
    fn load(&self) -> Result<Secret, Failure> {
        Secret::from_bytes(b"synthetic-chat-test-only".to_vec())
    }
}
fn reply(text: &str) -> MockReply {
    MockReply{status:200,body:serde_json::to_vec(&json!({"model":OPENROUTER_MODEL,"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":text}}],"usage":{"prompt_tokens":10,"completion_tokens":5}})).unwrap(),delay:Duration::ZERO,retry_after_ms:None,failure:None}
}
fn host(replies: Vec<MockReply>) -> (ChatHost, Arc<MockTransport>) {
    let t = Arc::new(MockTransport::new(replies));
    let mut h = ChatHost::new(t.clone(), Box::new(Credential));
    h.enabled = true;
    h.handle("new", Command::New {}).unwrap();
    (h, t)
}
fn send(h: &mut ChatHost, text: &str) {
    h.handle(
        "send",
        Command::Send {
            session_id: h.selected.clone(),
            text: text.into(),
        },
    )
    .unwrap();
}
#[test]
fn genuine_multiturn_context_and_fixed_route() {
    let (mut h, t) = host(vec![reply("Hello"), reply("Blue")]);
    send(&mut h, "Remember blue");
    assert!(h.poll().unwrap()["ok"].as_bool().unwrap());
    send(&mut h, "What colour?");
    h.poll().unwrap();
    let sent = t.sent.lock().unwrap();
    let v: Value = serde_json::from_slice(&sent[1]).unwrap();
    assert_eq!(v["messages"][1]["content"], "Remember blue");
    assert_eq!(v["messages"][2]["role"], "assistant");
    assert_eq!(v["messages"][3]["content"], "What colour?");
    assert_eq!(v["provider"]["only"], json!(["nvidia"]));
    assert_eq!(v["provider"]["allow_fallbacks"], false);
    assert_eq!(v["provider"]["max_price"]["request"], 0);
    assert_eq!(v["tools"], json!([]));
    assert_eq!(v["max_tokens"], 1024);
}
#[test]
fn no_consent_no_calls_and_empty_or_oversized_rejected() {
    let (mut h, t) = host(vec![]);
    h.enabled = false;
    assert!(matches!(
        h.start("t", h.selected.clone(), "hi".into()),
        Err(Failure::ConsentRequired)
    ));
    h.enabled = true;
    for text in [String::new(), "x".repeat(MAX_USER + 1)] {
        assert!(h.start("t", h.selected.clone(), text).is_err());
    }
    assert_eq!(t.attempts.load(Ordering::SeqCst), 0);
}
#[test]
fn cancelled_and_failed_messages_never_enter_context() {
    let mut delayed = reply("late");
    delayed.delay = Duration::from_secs(60);
    let (mut h, t) = host(vec![delayed, reply("okay")]);
    send(&mut h, "cancelled private text");
    h.cancel();
    assert!(h.poll().is_none());
    assert_eq!(t.aborted.load(Ordering::SeqCst), 1);
    send(&mut h, "fresh");
    h.poll().unwrap();
    let sent = t.sent.lock().unwrap();
    assert!(!String::from_utf8_lossy(&sent[1]).contains("cancelled private text"));
}
#[test]
fn switching_chats_isolates_context_and_cancels_pending() {
    let (mut h, t) = host(vec![reply("one"), reply("two")]);
    send(&mut h, "first chat only");
    h.poll().unwrap();
    h.handle("n", Command::New {}).unwrap();
    send(&mut h, "second");
    h.poll().unwrap();
    let sent = t.sent.lock().unwrap();
    assert!(!String::from_utf8_lossy(&sent[1]).contains("first chat only"));
    drop(sent);
    h.handle(
        "s",
        Command::Select {
            session_id: "chat-1".into(),
        },
    )
    .unwrap();
    assert_eq!(h.state()["messages"][0]["text"], "first chat only");
}
#[test]
fn strict_untrusted_responses_and_typed_errors() {
    for (status, code) in [
        (401, Failure::AuthFailed),
        (403, Failure::AuthFailed),
        (429, Failure::RateLimited),
        (503, Failure::Unavailable),
    ] {
        assert!(matches!(parse(status,b"{}"),Err(e)if e==code));
    }
    for value in [
        json!({"choices":[{"finish_reason":"tool_calls","message":{"role":"assistant","content":"x","tool_calls":[{"function":{"name":"run"}}]}}]}),
        json!({"choices":[{"finish_reason":"stop","message":{"role":"system","content":"x"}}]}),
    ] {
        assert!(matches!(
            parse(200, &serde_json::to_vec(&value).unwrap()),
            Err(Failure::SchemaMismatch)
        ));
    }
    assert!(strict_request(br#"{"type":"new","type":"quit"}"#).is_err());
    assert!(parse(200, br#"{"choices":[],"choices":[]}"#).is_err());
    let mut v: Value = serde_json::from_slice(&reply("x").body).unwrap();
    v["choices"][0]["finish_reason"] = json!("length");
    assert!(matches!(
        parse(200, &serde_json::to_vec(&v).unwrap()),
        Err(Failure::Truncated)
    ));
    v["choices"][0]["message"]["refusal"] = json!("no");
    assert!(matches!(
        parse(200, &serde_json::to_vec(&v).unwrap()),
        Err(Failure::Refused)
    ));
}
#[test]
fn secrets_are_rejected_before_send_or_display() {
    let (mut h, t) = host(vec![reply("synthetic-chat-test-only")]);
    assert!(h
        .start("x", h.selected.clone(), "synthetic-chat-test-only".into())
        .is_err());
    assert_eq!(t.attempts.load(Ordering::SeqCst), 0);
    send(&mut h, "hi");
    assert_eq!(h.poll().unwrap()["code"], "SCHEMA_MISMATCH");
    assert_eq!(h.state()["messages"].as_array().unwrap().len(), 1);
}
#[test]
fn context_bounded_without_destroying_transcript() {
    let (mut h, t) = host((0..14).map(|_| reply("response")).collect());
    for n in 0..14 {
        send(&mut h, &format!("turn{n}"));
        h.poll().unwrap();
    }
    assert_eq!(h.state()["messages"].as_array().unwrap().len(), 28);
    let sent = t.sent.lock().unwrap();
    let v: Value = serde_json::from_slice(sent.last().unwrap()).unwrap();
    assert_eq!(v["messages"].as_array().unwrap().len(), 26);
    assert_eq!(v["messages"][1]["content"], "turn1");
}

#[test]
fn failure_and_disable_do_not_contaminate_future_turns() {
    let mut failure = reply("no");
    failure.status = 429;
    let (mut h, t) = host(vec![failure, reply("okay")]);
    send(&mut h, "failed private turn");
    assert_eq!(h.poll().unwrap()["code"], "RATE_LIMITED");
    send(&mut h, "next turn");
    h.poll().unwrap();
    let sent = t.sent.lock().unwrap();
    assert!(!String::from_utf8_lossy(&sent[1]).contains("failed private turn"));
}
#[test]
fn timeout_and_new_chat_drop_exchange_without_committing_late_answer() {
    let mut delayed = reply("late");
    delayed.delay = Duration::from_secs(60);
    let (mut h, t) = host(vec![delayed.clone(), delayed]);
    send(&mut h, "timed out");
    h.pending.as_mut().unwrap().started = Instant::now() - Duration::from_secs(31);
    assert_eq!(h.poll().unwrap()["code"], "TIMEOUT");
    assert_eq!(t.aborted.load(Ordering::SeqCst), 1);
    send(&mut h, "switch away");
    h.handle("new", Command::New {}).unwrap();
    assert!(!h.busy());
    assert_eq!(t.aborted.load(Ordering::SeqCst), 2);
    assert_eq!(h.state()["messages"], json!([]));
    assert!(h.poll().is_none());
}
#[test]
fn caller_cannot_supply_roles_history_or_unknown_commands() {
    for raw in [
        r#"{"type":"send","sessionId":"chat-1","text":"hi","role":"system"}"#,
        r#"{"type":"send","sessionId":"chat-1","text":"hi","messages":[]}"#,
        r#"{"type":"execute","command":"anything"}"#,
        r#"{"type":"new","execute":"anything"}"#,
        r#"{"type":"cancel","extra":true}"#,
        r#"{"type":"quit","extra":true}"#,
    ] {
        assert!(serde_json::from_str::<Command>(raw).is_err());
    }
}
#[test]
fn context_bytes_and_response_bounds_are_enforced() {
    let messages: Vec<_> = (0..24)
        .map(|n| Message {
            role: if n % 2 == 0 { "user" } else { "assistant" },
            text: "x".repeat(8000),
            status: "complete",
            error: None,
        })
        .collect();
    let (body, turns) = request(&messages, "hello").unwrap();
    assert!(body.len() < 65536);
    assert!(turns < MAX_TURNS);
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["messages"][1]["role"], "user");
    assert_eq!(v["messages"].as_array().unwrap().len(), turns * 2 + 2);
    let value = reply(&"x".repeat(MAX_ASSISTANT + 1));
    assert!(matches!(
        parse(200, &value.body),
        Err(Failure::BudgetExceeded)
    ));
}

#[test]
fn disabling_with_invalid_replacement_source_revokes_and_cancels() {
    let mut delayed = reply("late");
    delayed.delay = Duration::from_secs(60);
    let (mut h, t) = host(vec![delayed]);
    send(&mut h, "pending");
    assert!(h
        .handle(
            "off",
            Command::Configure {
                enabled: false,
                credential_path: Some("relative-invalid".into())
            }
        )
        .is_err());
    assert!(!h.enabled);
    assert!(!h.busy());
    assert_eq!(t.aborted.load(Ordering::SeqCst), 1);
    assert_eq!(h.state()["messages"][0]["status"], "cancelled");
    assert!(h.poll().is_none());
}
#[test]
fn escaped_transcript_quota_is_recoverable_before_next_network_call() {
    let (mut h, t) = host(vec![]);
    let s = h.session_mut("chat-1");
    for n in 0..100 {
        s.messages.push(Message {
            role: if n % 2 == 0 { "user" } else { "assistant" },
            text: "\u{1}".repeat(1600),
            status: "complete",
            error: None,
        });
    }
    let before = h.state();
    assert!(serde_json::to_vec(&before).unwrap().len() < 4 * 1024 * 1024);
    assert!(matches!(
        h.start("x", "chat-1".into(), "next".into()),
        Err(Failure::BudgetExceeded)
    ));
    assert_eq!(h.state(), before);
    assert_eq!(t.attempts.load(Ordering::SeqCst), 0);
    h.handle("new", Command::New {}).unwrap();
    assert_eq!(h.state()["selected"], "chat-2");
}
#[test]
fn request_serialization_failure_does_not_store_turn_or_spend_call() {
    let (mut h, t) = host(vec![]);
    let s = h.session_mut("chat-1");
    for n in 0..8 {
        s.messages.push(Message {
            role: if n % 2 == 0 { "user" } else { "assistant" },
            text: "\u{1}".repeat(3000),
            status: "complete",
            error: None,
        });
    }
    let before = h.state();
    assert!(matches!(
        h.start("x", "chat-1".into(), "next".into()),
        Err(Failure::BudgetExceeded)
    ));
    assert_eq!(h.state(), before);
    assert_eq!(t.attempts.load(Ordering::SeqCst), 0);
}

#[test]
fn new_chat_at_capacity_still_revokes_pending_request() {
    let mut delayed = reply("late");
    delayed.delay = Duration::from_secs(60);
    let (mut h, t) = host(vec![delayed]);
    for _ in 1..16 {
        h.handle("new", Command::New {}).unwrap();
    }
    send(&mut h, "stop at capacity");
    assert!(h.handle("new", Command::New {}).is_err());
    assert!(!h.busy());
    assert_eq!(t.aborted.load(Ordering::SeqCst), 1);
    assert_eq!(h.state()["messages"][0]["status"], "cancelled");
    assert!(h.poll().is_none());
}
