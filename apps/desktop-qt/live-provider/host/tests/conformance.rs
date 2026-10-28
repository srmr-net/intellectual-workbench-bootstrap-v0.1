use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    task::{Context, Poll, Waker},
    time::Duration,
};
use wb_live_provider::{policy::*, provider::*, response, transport::*};
use wb_operator::{
    provider::*,
    suggestion::{mortgage_terms, Suggestion},
};
const INTENT: &str = "I need a mortgage calculator for a £250,000 loan over 25 years at 5%.";
fn request() -> ProviderRequest {
    ProviderRequest {
        request_version: "0.1".into(),
        request_id: "request-one".into(),
        utterance: INTENT.into(),
        candidates: vec![Candidate {
            candidate_key: "opaque-one".into(),
            capability: "https://workbench.example/ns/0.1#CalculateMortgagePayment".into(),
            inputs: vec![
                "principal".into(),
                "annual_rate_percent".into(),
                "term_months".into(),
            ],
            has_semantic_ui: true,
        }],
    }
}
fn suggestion() -> Value {
    json!({"suggestionVersion":"0.1","candidateKey":"opaque-one","inputValues":mortgage_terms(INTENT).unwrap(),"explanation":"Explicit terms"})
}
fn envelope(s: Value) -> Value {
    json!({"model":TESTED_MODEL,"status":"completed","error":null,"incomplete_details":null,"output":[{"type":"message","role":"assistant","status":"completed","content":[{"type":"output_text","text":s.to_string()}]}],"usage":{"input_tokens":100,"output_tokens":60}})
}
fn reply(status: u16, v: Value) -> MockReply {
    MockReply {
        status,
        body: serde_json::to_vec(&v).unwrap(),
        delay: Duration::ZERO,
        retry_after_ms: None,
        failure: None,
    }
}
fn profile(consent: bool) -> Profile {
    Profile {
        provider: Kind::OpenaiResponses,
        model: TESTED_MODEL.into(),
        remote_consent: consent,
        ..Profile::default()
    }
}
struct Credential(AtomicU64);
impl CredentialSource for Credential {
    fn load(&self) -> Result<Secret, Failure> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Secret::from_bytes(b"synthetic-fixture-secret".to_vec())
    }
}
struct Harness {
    control: Arc<Control>,
    key: Arc<Credential>,
    transport: Arc<MockTransport>,
    provider: LiveProvider,
    cancel: Cancellation,
}
impl Harness {
    fn new(replies: Vec<MockReply>, consent: bool) -> Self {
        Self::budget(replies, consent, Budget::default())
    }
    fn budget(replies: Vec<MockReply>, consent: bool, budget: Budget) -> Self {
        let control = Arc::new(Control::default());
        control
            .configure(Profile {
                budget,
                ..profile(consent)
            })
            .unwrap();
        let key = Arc::new(Credential(0.into()));
        let transport = Arc::new(MockTransport::new(replies));
        let cancel = Cancellation::default();
        let permit = control.consume_consent(INTENT, cancel.clone());
        let provider = LiveProvider::new(
            control.clone(),
            key.clone(),
            transport.clone(),
            "room-one".into(),
            permit,
        )
        .unwrap();
        Self {
            control,
            key,
            transport,
            provider,
            cancel,
        }
    }
    fn run(&self) -> Result<Vec<u8>, wb_operator::suggestion::OperatorError> {
        await_provider(
            &self.provider,
            request(),
            self.cancel.clone(),
            Duration::from_secs(2),
        )
    }
    fn status(&self) -> String {
        self.control.latest("room-one").unwrap().observation.status
    }
}
fn parse(v: Value) -> Result<response::Parsed, Failure> {
    response::parse(
        200,
        &serde_json::to_vec(&v).unwrap(),
        &Budget::default(),
        &request(),
        TESTED_MODEL,
    )
}
#[test]
fn llm_001_offline_without_credentials() {
    let r = await_provider(
        &OfflineFixture,
        request(),
        Cancellation::default(),
        Duration::from_secs(1),
    )
    .unwrap();
    assert_eq!(
        Suggestion::parse(&r).unwrap().input_values,
        mortgage_terms(INTENT).unwrap()
    );
    assert_eq!(Control::default().profile().provider, Kind::OfflineFixture);
}
#[test]
fn llm_002_consent_before_credential_or_transport() {
    let h = Harness::new(vec![], false);
    assert!(h.run().is_err());
    assert_eq!(h.status(), "CONSENT_REQUIRED");
    assert_eq!(h.key.0.load(Ordering::SeqCst), 0);
    assert_eq!(h.transport.attempts.load(Ordering::SeqCst), 0);
    let mut p = profile(true);
    p.model = "unvetted-model".into();
    assert_eq!(p.validate(), Err(Failure::NotConfigured));
}
#[test]
fn llm_003_minimized_payload() {
    let body = response::request_body(&request(), TESTED_MODEL, &Budget::default()).unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let input: Value = serde_json::from_str(v["input"][0]["content"].as_str().unwrap()).unwrap();
    assert_eq!(input.as_object().unwrap().len(), 2);
    assert_eq!(input["utterance"], INTENT);
    assert_eq!(
        input["candidates"],
        serde_json::to_value(request().candidates).unwrap()
    );
    for absent in [
        "request-one",
        "room-one",
        "/home/",
        "credential",
        "snapshot",
    ] {
        assert!(!String::from_utf8_lossy(&body).contains(absent));
    }
}
#[test]
fn llm_004_closed_host_configuration() {
    let mut p = serde_json::to_value(profile(true)).unwrap();
    p["endpoint"] = json!("http://127.0.0.1/");
    assert!(serde_json::from_value::<Profile>(p).is_err());
    assert_eq!(ENDPOINT, "https://api.openai.com/v1/responses");
    assert_eq!(
        response::parse(302, b"", &Budget::default(), &request(), TESTED_MODEL).err(),
        Some(Failure::Unavailable)
    );
}
#[test]
fn llm_005_credentials_excluded_and_echo_blocked() {
    let mut v = envelope(suggestion());
    v["output"][0]["content"][0]["text"] = json!(suggestion()
        .to_string()
        .replace("Explicit terms", "synthetic-fixture-secret"));
    let h = Harness::new(vec![reply(200, v)], true);
    assert!(h.run().is_err());
    assert_eq!(h.status(), "SCHEMA_MISMATCH");
    let logs = serde_json::to_string(&h.control.audits("room-one")).unwrap();
    assert!(!logs.contains("synthetic-fixture-secret"));
    assert!(h
        .transport
        .sent
        .lock()
        .unwrap()
        .iter()
        .all(|b| !String::from_utf8_lossy(b).contains("synthetic-fixture-secret")));
}
#[test]
fn llm_006_strict_api_schema_and_no_tools() {
    let v: Value = serde_json::from_slice(
        &response::request_body(&request(), TESTED_MODEL, &Budget::default()).unwrap(),
    )
    .unwrap();
    assert_eq!(v["model"], TESTED_MODEL);
    for k in ["store", "stream", "background"] {
        assert_eq!(v[k], false);
    }
    assert_eq!(v["tools"], json!([]));
    assert_eq!(v["tool_choice"], "none");
    assert_eq!(v["text"]["format"]["strict"], true);
    let schema = &v["text"]["format"]["schema"];
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        schema["properties"]["inputValues"]["additionalProperties"],
        false
    );
    assert_eq!(
        schema["properties"]["candidateKey"]["enum"],
        json!(["opaque-one"])
    );
    assert_eq!(schema["required"].as_array().unwrap().len(), 4);
}
#[test]
fn llm_007_valid_canonical_suggestion() {
    let h = Harness::new(vec![reply(200, envelope(suggestion()))], true);
    assert_eq!(
        Suggestion::parse(&h.run().unwrap()).unwrap().candidate_key,
        "opaque-one"
    );
    assert_eq!(h.status(), "OK");
    assert_eq!(h.transport.attempts.load(Ordering::SeqCst), 1);
}
#[test]
fn llm_008_strict_json_and_size() {
    for raw in [b"{\"model\":1,\"model\":2}".as_slice(), b"\xff", b"{"] {
        assert_eq!(
            response::parse(200, raw, &Budget::default(), &request(), TESTED_MODEL).err(),
            Some(Failure::MalformedResponse)
        );
    }
    let mut s = suggestion();
    s["unknown"] = json!(1);
    assert_eq!(parse(envelope(s)).err(), Some(Failure::SchemaMismatch));
    assert_eq!(
        response::parse(
            200,
            &vec![0; 16385],
            &Budget::default(),
            &request(),
            TESTED_MODEL
        )
        .err(),
        Some(Failure::BudgetExceeded)
    );
}
#[test]
fn llm_009_invented_authority() {
    for key in ["invented", "https://evil.example/", "component-id"] {
        let mut s = suggestion();
        s["candidateKey"] = json!(key);
        assert_eq!(parse(envelope(s)).err(), Some(Failure::SchemaMismatch));
    }
    for key in ["port", "permissions", "planVersion", "url"] {
        let mut s = suggestion();
        s[key] = json!("forbidden");
        assert!(parse(envelope(s)).is_err());
    }
    let mut s = suggestion();
    s["inputValues"]["inventedPort"] = json!(1);
    assert!(parse(envelope(s)).is_err());
}
#[test]
fn llm_010_refusal() {
    let mut v = envelope(suggestion());
    v["output"][0]["content"] = json!([{"type":"refusal","refusal":"sensitive diagnostic"}]);
    assert_eq!(parse(v).err(), Some(Failure::Refused));
    let mut v = envelope(suggestion());
    v["status"] = json!("incomplete");
    v["incomplete_details"] = json!({"reason":"content_filter"});
    assert_eq!(parse(v).err(), Some(Failure::Refused));
}
#[test]
fn llm_011_incomplete_and_schema_separate() {
    let mut v = envelope(suggestion());
    v["status"] = json!("incomplete");
    v["incomplete_details"] = json!({"reason":"max_output_tokens"});
    assert_eq!(parse(v).err(), Some(Failure::Truncated));
    assert_eq!(
        response::parse(
            400,
            b"{\"error\":{\"code\":\"invalid_json_schema\"}}",
            &Budget::default(),
            &request(),
            TESTED_MODEL
        )
        .err(),
        Some(Failure::SchemaMismatch)
    );
}
#[test]
fn llm_012_auth_no_retry() {
    for status in [401, 403] {
        let mut h = Harness::budget(
            vec![reply(status, json!({"error":"synthetic-fixture-secret"}))],
            true,
            Budget {
                max_calls: 3,
                ..Budget::default()
            },
        );
        h.provider.retry_transient = true;
        assert!(h.run().is_err());
        assert_eq!(h.status(), "AUTH_FAILED");
        assert_eq!(h.transport.attempts.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn llm_013_retry_bounds() {
    for status in [429, 503] {
        let mut h = Harness::budget(
            vec![reply(status, json!({})), reply(200, envelope(suggestion()))],
            true,
            Budget {
                max_calls: 3,
                ..Budget::default()
            },
        );
        h.provider.retry_transient = true;
        assert!(h.run().is_ok());
        assert_eq!(h.transport.attempts.load(Ordering::SeqCst), 2);
        let mut r = reply(status, json!({}));
        r.retry_after_ms = Some(2001);
        let mut h = Harness::budget(
            vec![r],
            true,
            Budget {
                max_calls: 3,
                ..Budget::default()
            },
        );
        h.provider.retry_transient = true;
        assert!(h.run().is_err());
        assert_eq!(h.transport.attempts.load(Ordering::SeqCst), 1);
    }
    let h = Harness::new(vec![reply(429, json!({}))], true);
    assert!(h.run().is_err());
    assert_eq!(h.status(), "RATE_LIMITED");
}
#[test]
fn llm_014_budgets() {
    let b = Budget {
        max_request_bytes: 1,
        ..Budget::default()
    };
    assert_eq!(
        response::request_body(&request(), TESTED_MODEL, &b).err(),
        Some(Failure::BudgetExceeded)
    );
    let mut r = request();
    r.candidates = vec![r.candidates[0].clone(); 9];
    assert_eq!(
        response::request_body(&r, TESTED_MODEL, &Budget::default()).err(),
        Some(Failure::BudgetExceeded)
    );
    let mut v = envelope(suggestion());
    v["usage"]["output_tokens"] = json!(513);
    assert_eq!(parse(v).err(), Some(Failure::BudgetExceeded));
    let mut r = reply(200, envelope(suggestion()));
    r.delay = Duration::from_secs(5);
    let h = Harness::new(vec![r], true);
    assert!(await_provider(
        &h.provider,
        request(),
        h.cancel.clone(),
        Duration::from_millis(10)
    )
    .is_err());
    assert_eq!(h.transport.aborted.load(Ordering::SeqCst), 1);
}
#[test]
fn llm_015_cancel_before_and_pending() {
    let h = Harness::new(vec![], true);
    h.cancel.cancel();
    assert!(h.run().is_err());
    assert_eq!(h.key.0.load(Ordering::SeqCst), 0);
    let mut r = reply(200, envelope(suggestion()));
    r.delay = Duration::from_secs(5);
    let h = Harness::new(vec![r], true);
    let mut f = h.provider.suggest(request(), h.cancel.clone());
    let mut cx = Context::from_waker(Waker::noop());
    assert!(f.as_mut().poll(&mut cx).is_pending());
    h.cancel.cancel();
    assert!(matches!(f.as_mut().poll(&mut cx), Poll::Ready(Err(_))));
    drop(f);
    assert_eq!(h.transport.aborted.load(Ordering::SeqCst), 1);
    assert_eq!(h.status(), "CANCELLED");
}
#[test]
fn llm_020_single_use_room_and_switch_fences() {
    let h = Harness::new(vec![reply(200, envelope(suggestion()))], true);
    assert!(h.run().is_ok());
    assert!(h.run().is_err());
    assert_eq!(h.transport.attempts.load(Ordering::SeqCst), 1);
    assert!(h
        .control
        .consume_consent(INTENT, Cancellation::default())
        .is_none());
    let h = Harness::new(vec![], true);
    assert_eq!(h.provider.kind(), "openai-responses");
    h.control.configure(Profile::default()).unwrap();
    assert_eq!(h.provider.kind(), "openai-responses");
    assert!(LiveProvider::new(
        h.control.clone(),
        h.key.clone(),
        h.transport.clone(),
        "room-two".into(),
        None
    )
    .is_err());
    assert!(h.run().is_err());
    assert_eq!(h.transport.attempts.load(Ordering::SeqCst), 0);
    let c = Arc::new(Control::default());
    c.configure(profile(true)).unwrap();
    let cancel = Cancellation::default();
    let p = c.consume_consent(INTENT, cancel.clone()).unwrap();
    LiveProvider::new(
        c.clone(),
        Arc::new(Credential(0.into())),
        Arc::new(MockTransport::new(vec![])),
        "room-one".into(),
        Some(p.clone()),
    )
    .unwrap();
    assert!(LiveProvider::new(
        c,
        Arc::new(Credential(0.into())),
        Arc::new(MockTransport::new(vec![])),
        "room-two".into(),
        Some(p)
    )
    .is_err());
}
#[test]
fn llm_021_redacted_observation() {
    let h = Harness::new(vec![reply(200, envelope(suggestion()))], true);
    h.run().unwrap();
    let a = serde_json::to_value(h.control.latest("room-one").unwrap()).unwrap();
    assert_eq!(a["schemaId"], "OperatorSuggestionV0.1");
    assert_eq!(a["observation"]["correlationId"], "request-one");
    assert_eq!(a["observation"]["inputTokens"], 100);
    assert_eq!(a["attempts"], 1);
    let text = a.to_string();
    for forbidden in [
        INTENT,
        "synthetic-fixture-secret",
        "Explicit terms",
        "opaque-one",
    ] {
        assert!(!text.contains(forbidden));
    }
}
#[test]
fn llm_022_transport_failures_and_tools() {
    for failure in [
        Failure::Unavailable,
        Failure::Timeout,
        Failure::MalformedResponse,
    ] {
        let mut r = reply(200, json!({}));
        r.failure = Some(failure);
        let h = Harness::new(vec![r], true);
        assert!(h.run().is_err());
        assert_eq!(h.status(), failure.code());
    }
    let mut v = envelope(suggestion());
    v["output"]
        .as_array_mut()
        .unwrap()
        .push(json!({"type":"function_call","name":"execute","arguments":"{}"}));
    assert_eq!(parse(v).err(), Some(Failure::ProviderError));
    for status in [429, 503] {
        let mut h = Harness::budget(
            vec![reply(status, json!("x".repeat(17000)))],
            true,
            Budget {
                max_calls: 3,
                ..Budget::default()
            },
        );
        h.provider.retry_transient = true;
        assert!(h.run().is_err());
        assert_eq!(h.status(), "BUDGET_EXCEEDED");
        assert_eq!(h.transport.attempts.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn generation_transaction_switch_before_apply() {
    let c = Control::default();
    let gen = c.generation();
    c.configure(profile(false)).unwrap();
    let applied = AtomicU64::new(0);
    assert_eq!(
        c.with_generation(gen, || applied.fetch_add(1, Ordering::SeqCst)),
        Err(Failure::Cancelled)
    );
    assert_eq!(applied.load(Ordering::SeqCst), 0);
}
#[test]
fn generation_transaction_apply_before_switch() {
    use std::sync::mpsc;
    let c = Arc::new(Control::default());
    let (entered, wait) = mpsc::channel();
    let (release, hold) = mpsc::channel();
    let t = c.clone();
    let apply = std::thread::spawn(move || {
        t.with_generation(0, || {
            entered.send(()).unwrap();
            hold.recv().unwrap();
        })
    });
    wait.recv().unwrap();
    let t = c.clone();
    let (done, rx) = mpsc::channel();
    let switch = std::thread::spawn(move || {
        t.configure(profile(false)).unwrap();
        done.send(()).unwrap();
    });
    assert!(rx.recv_timeout(Duration::from_millis(20)).is_err());
    release.send(()).unwrap();
    apply.join().unwrap().unwrap();
    rx.recv_timeout(Duration::from_secs(1)).unwrap();
    switch.join().unwrap();
}
#[test]
fn slot_publication_cannot_escape_invalidation() {
    use std::sync::{mpsc, Mutex};
    let slot = Arc::new(Mutex::new(None));
    let (entered, wait) = mpsc::channel();
    let (release, hold) = mpsc::channel();
    let s = slot.clone();
    let creator = std::thread::spawn(move || {
        wb_live_provider::slot::get_or_try_create(
            &s,
            |_: &u8| true,
            |_| {},
            || {
                entered.send(()).unwrap();
                hold.recv().unwrap();
                Ok::<_, ()>(7)
            },
        )
        .unwrap()
    });
    wait.recv().unwrap();
    let s = slot.clone();
    let (done, rx) = mpsc::channel();
    let invalidator = std::thread::spawn(move || {
        let value = s.lock().unwrap().take();
        done.send(value).unwrap();
    });
    assert!(rx.recv_timeout(Duration::from_millis(20)).is_err());
    release.send(()).unwrap();
    assert_eq!(creator.join().unwrap(), 7);
    assert_eq!(rx.recv_timeout(Duration::from_secs(1)).unwrap(), Some(7));
    invalidator.join().unwrap();
    assert!(slot.lock().unwrap().is_none());
}
#[test]
fn private_file_credentials_reject_links_and_permissions() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let path = std::env::temp_dir().join(format!("wb-llm-key-fixture-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let file = path.join("fixture");
    std::fs::write(&file, b"not-a-real-key\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(HostCredential::PrivateFile(file.clone()).load().is_ok());
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(HostCredential::PrivateFile(file.clone()).load().is_err());
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    let link = path.join("symlink");
    symlink(&file, &link).unwrap();
    assert!(HostCredential::PrivateFile(link).load().is_err());
    let hard = path.join("hardlink");
    std::fs::hard_link(&file, &hard).unwrap();
    assert!(HostCredential::PrivateFile(file).load().is_err());
    std::fs::remove_dir_all(path).unwrap();
}
#[test]
fn consent_binds_exact_input_and_host_ticket() {
    let h = Harness::new(vec![], true);
    let mut r = request();
    r.utterance.push(' ');
    assert!(await_provider(&h.provider, r, h.cancel.clone(), Duration::from_secs(1)).is_err());
    assert_eq!(h.status(), "CONSENT_REQUIRED");
    assert_eq!(h.key.0.load(Ordering::SeqCst), 0);
    let h = Harness::new(vec![], true);
    assert!(await_provider(
        &h.provider,
        request(),
        Cancellation::default(),
        Duration::from_secs(1)
    )
    .is_err());
    assert_eq!(h.status(), "CONSENT_REQUIRED");
    assert_eq!(h.key.0.load(Ordering::SeqCst), 0);
}
struct RevokingTransport {
    control: Arc<Control>,
    during_start: bool,
    polls: Arc<AtomicU64>,
}
struct RevokingExchange {
    control: Arc<Control>,
    revoke: bool,
    polls: Arc<AtomicU64>,
}
impl Transport for RevokingTransport {
    fn start(&self, _: &[u8], _: Secret, _: usize, _: u64) -> Result<Box<dyn Exchange>, Failure> {
        if self.during_start {
            self.control.configure(Profile::default()).unwrap();
        }
        Ok(Box::new(RevokingExchange {
            control: self.control.clone(),
            revoke: !self.during_start,
            polls: self.polls.clone(),
        }))
    }
}
impl Exchange for RevokingExchange {
    fn poll(&mut self) -> Poll<Result<HttpResponse, Failure>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        if self.revoke {
            self.control.configure(Profile::default()).unwrap();
        }
        Poll::Ready(Ok(HttpResponse {
            status: 200,
            body: serde_json::to_vec(&envelope(suggestion())).unwrap(),
            retry_after_ms: None,
        }))
    }
}
#[test]
fn revoke_inside_start_and_ready_poll_cannot_publish() {
    for during_start in [true, false] {
        let c = Arc::new(Control::default());
        c.configure(profile(true)).unwrap();
        let cancel = Cancellation::default();
        let permit = c.consume_consent(INTENT, cancel.clone());
        let polls = Arc::new(AtomicU64::new(0));
        let t = Arc::new(RevokingTransport {
            control: c.clone(),
            during_start,
            polls: polls.clone(),
        });
        let p = LiveProvider::new(
            c.clone(),
            Arc::new(Credential(0.into())),
            t,
            "room-one".into(),
            permit,
        )
        .unwrap();
        assert!(await_provider(&p, request(), cancel, Duration::from_secs(1)).is_err());
        assert_eq!(
            c.latest("room-one").unwrap().observation.status,
            "CANCELLED"
        );
        assert_eq!(
            polls.load(Ordering::SeqCst),
            if during_start { 0 } else { 1 }
        );
    }
}

fn chat(s: Value) -> Value {
    json!({"model":OPENROUTER_MODEL,"provider":"Nvidia","choices":[{"index":0,"finish_reason":"stop","message":{"role":"assistant","content":s.to_string()}}],"usage":{"prompt_tokens":100,"completion_tokens":60}})
}
fn parse_chat(v: Value) -> Result<response::Parsed, Failure> {
    response::parse_for(
        &Kind::OpenrouterChat,
        200,
        &serde_json::to_vec(&v).unwrap(),
        &Budget::default(),
        &request(),
        OPENROUTER_MODEL,
    )
}
#[test]
fn openrouter_explicit_profile_payload_and_no_schema_facility_claim() {
    let p = Profile {
        provider: Kind::OpenrouterChat,
        model: OPENROUTER_MODEL.into(),
        ..profile(false)
    };
    p.validate().unwrap();
    let v: Value = serde_json::from_slice(
        &response::request_body_for(&p.provider, &request(), &p.model, &p.budget).unwrap(),
    )
    .unwrap();
    assert_eq!(v["model"], OPENROUTER_MODEL);
    assert_eq!(v["provider"]["only"], json!(["nvidia"]));
    assert_eq!(v["provider"]["allow_fallbacks"], false);
    assert_eq!(v["provider"]["require_parameters"], true);
    assert_eq!(
        v["provider"]["max_price"],
        json!({"prompt":0,"completion":0,"request":0})
    );
    assert_eq!(v["provider"]["data_collection"], "allow");
    assert!(v.get("response_format").is_none());
    assert_eq!(v["tools"], json!([]));
    assert_eq!(v["tool_choice"], "none");
    assert_eq!(v["max_tokens"], 512);
    assert_eq!(v["reasoning"]["enabled"], false);
    assert_eq!(v["reasoning"]["exclude"], true);
    let user: Value = serde_json::from_str(v["messages"][1]["content"].as_str().unwrap()).unwrap();
    assert_eq!(user.as_object().unwrap().len(), 2);
    assert_eq!(user["utterance"], INTENT);
    assert!(!v.to_string().contains("request-one"));
    assert!(v["messages"][0]["content"]
        .as_str()
        .unwrap()
        .contains("does not enforce JSON Schema"));
    assert_eq!(Kind::OpenrouterChat.endpoint(), OPENROUTER_ENDPOINT);
}
#[test]
fn openrouter_strict_canonical_chat_and_failure_taxonomy() {
    assert_eq!(
        Suggestion::parse(&parse_chat(chat(suggestion())).unwrap().suggestion)
            .unwrap()
            .candidate_key,
        "opaque-one"
    );
    for (field, value, failure) in [
        ("finish_reason", json!("length"), Failure::Truncated),
        ("finish_reason", json!("content_filter"), Failure::Refused),
        ("finish_reason", json!("tool_calls"), Failure::ProviderError),
    ] {
        let mut v = chat(suggestion());
        v["choices"][0][field] = value;
        assert_eq!(parse_chat(v).err(), Some(failure));
    }
    for field in ["tool_calls", "function_call"] {
        let mut v = chat(suggestion());
        v["choices"][0]["message"][field] = json!({"name":"execute"});
        assert_eq!(parse_chat(v).err(), Some(Failure::ProviderError));
    }
    let mut v = chat(suggestion());
    v["choices"][0]["message"]["refusal"] = json!("Never log raw refusal");
    assert_eq!(parse_chat(v).err(), Some(Failure::Refused));
    for field in ["model", "provider"] {
        let mut v = chat(suggestion());
        v[field] = json!("unexpected");
        assert!(parse_chat(v).is_err());
    }
    let mut v = chat(suggestion());
    v["choices"][0]["message"]["content"] = json!(format!("```json\n{}\n```", suggestion()));
    assert_eq!(parse_chat(v).err(), Some(Failure::SchemaMismatch));
    let mut s = suggestion();
    s["candidateKey"] = json!("invented");
    assert_eq!(parse_chat(chat(s)).err(), Some(Failure::SchemaMismatch));
    let mut s = suggestion();
    s["inputValues"]["principal"] = json!("42");
    assert_eq!(parse_chat(chat(s)).err(), Some(Failure::SchemaMismatch));
    let mut v = chat(suggestion());
    v["choices"][0]["message"]["content"] =
        json!("{\"candidateKey\":\"a\",\"candidateKey\":\"b\"}");
    assert_eq!(parse_chat(v).err(), Some(Failure::SchemaMismatch));
    for (status, error) in [
        (402, Failure::BudgetExceeded),
        (408, Failure::Timeout),
        (401, Failure::AuthFailed),
        (429, Failure::RateLimited),
        (503, Failure::Unavailable),
    ] {
        assert_eq!(
            response::parse_for(
                &Kind::OpenrouterChat,
                status,
                b"{}",
                &Budget::default(),
                &request(),
                OPENROUTER_MODEL
            )
            .err(),
            Some(error)
        );
    }
}
#[test]
fn openrouter_same_permit_and_canonical_provider_boundary() {
    let c = Arc::new(Control::default());
    c.configure(Profile {
        provider: Kind::OpenrouterChat,
        model: OPENROUTER_MODEL.into(),
        ..profile(true)
    })
    .unwrap();
    let cancel = Cancellation::default();
    let permit = c.consume_consent(INTENT, cancel.clone());
    let key = Arc::new(Credential(0.into()));
    let t = Arc::new(MockTransport::new(vec![reply(200, chat(suggestion()))]));
    let p = LiveProvider::new(c.clone(), key, t.clone(), "room-router".into(), permit).unwrap();
    let result = await_provider(&p, request(), cancel.clone(), Duration::from_secs(1)).unwrap();
    assert_eq!(
        Suggestion::parse(&result).unwrap().input_values,
        mortgage_terms(INTENT).unwrap()
    );
    assert!(await_provider(&p, request(), cancel, Duration::from_secs(1)).is_err());
    assert_eq!(t.attempts.load(Ordering::SeqCst), 1);
    let audit = serde_json::to_value(c.audits("room-router")[0].clone()).unwrap();
    assert_eq!(audit["observation"]["provider"], "openrouter-chat");
    assert_eq!(audit["observation"]["model"], OPENROUTER_MODEL);
    assert_eq!(audit["observation"]["outputTokens"], 60);
    c.configure(Profile::default()).unwrap();
    assert_eq!(p.kind(), "openrouter-chat");
}
