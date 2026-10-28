use crate::{
    policy::{Budget, CredentialSource, Failure, Kind, Profile},
    response,
    transport::{Exchange, Transport},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, VecDeque},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};
use wb_operator::{
    provider::{Cancellation, OperatorProvider, ProviderFuture, ProviderRequest},
    suggestion::OperatorError,
};

/// Non-serializable host permit for exactly one queued request, room and invocation.
#[derive(Clone)]
pub struct Permit {
    generation: u64,
    expires: Instant,
    cancel: Cancellation,
    input_digest: [u8; 32],
    binding: Arc<Mutex<PermitBinding>>,
}
struct PermitBinding {
    room: Option<String>,
    request: Option<String>,
    owner: Option<Arc<()>>,
    spent: bool,
}
impl Permit {
    fn bind_room(&self, room: &str) -> Result<(), Failure> {
        let mut b = self.binding.lock().unwrap();
        if b.room.as_ref().is_some_and(|r| r != room) {
            return Err(Failure::ConsentRequired);
        }
        b.room = Some(room.into());
        Ok(())
    }
    fn scope(
        &self,
        room: &str,
        request: &ProviderRequest,
        cancel: &Cancellation,
    ) -> Result<(), Failure> {
        if !self.cancel.same_request(cancel)
            || self.input_digest != <[u8; 32]>::from(Sha256::digest(request.utterance.as_bytes()))
        {
            return Err(Failure::ConsentRequired);
        }
        if Instant::now() >= self.expires {
            return Err(Failure::Timeout);
        }
        let b = self.binding.lock().unwrap();
        if b.spent || b.room.as_deref() != Some(room) {
            return Err(Failure::ConsentRequired);
        }
        Ok(())
    }
    fn claim(&self, request: &str, owner: &Arc<()>) -> Result<(), Failure> {
        let mut b = self.binding.lock().unwrap();
        if b.owner.is_some() || b.spent {
            return Err(Failure::ConsentRequired);
        }
        b.request = Some(request.into());
        b.owner = Some(owner.clone());
        Ok(())
    }
    fn owned(&self, request: &str, owner: &Arc<()>) -> bool {
        let b = self.binding.lock().unwrap();
        !b.spent
            && b.request.as_deref() == Some(request)
            && b.owner.as_ref().is_some_and(|o| Arc::ptr_eq(o, owner))
    }
    fn finish(&self, owner: &Arc<()>) {
        let mut b = self.binding.lock().unwrap();
        if b.owner.as_ref().is_some_and(|o| Arc::ptr_eq(o, owner)) {
            b.spent = true;
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    observation_version: &'static str,
    correlation_id: String,
    provider: &'static str,
    model: String,
    pub status: String,
    duration_ms: u64,
    catalog_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_tokens: Option<u64>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    schema_id: &'static str,
    pub sequence: u64,
    room: String,
    generation: u64,
    budget: Budget,
    pub attempts: u8,
    pub observation: Observation,
}
struct Active {
    request_id: String,
    cancel: Cancellation,
}
struct State {
    profile: Profile,
    generation: u64,
    active: BTreeMap<String, Active>,
    audit: VecDeque<Audit>,
    sequence: u64,
}
pub struct Control {
    transaction: Mutex<()>,
    state: Mutex<State>,
    pub network_calls: AtomicU64,
}
impl Default for Control {
    fn default() -> Self {
        Self {
            transaction: Mutex::new(()),
            state: Mutex::new(State {
                profile: Profile::default(),
                generation: 0,
                active: BTreeMap::new(),
                audit: VecDeque::new(),
                sequence: 0,
            }),
            network_calls: 0.into(),
        }
    }
}
impl Observation {
    pub fn correlation(&self) -> &str {
        &self.correlation_id
    }
}
impl Control {
    pub fn generation(&self) -> u64 {
        self.state.lock().unwrap().generation
    }
    /// Admission linearizes before transport start; revocation cannot un-send
    /// an already admitted request, cancel server inference or guarantee no billing.
    fn admit(&self, generation: u64) -> Result<(), Failure> {
        let s = self.state.lock().unwrap();
        if s.generation != generation {
            return Err(Failure::Cancelled);
        }
        self.network_calls
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_add(1))
            .map_err(|_| Failure::BudgetExceeded)?;
        Ok(())
    }
    pub fn consume_consent(&self, utterance: &str, cancel: Cancellation) -> Option<Permit> {
        let mut s = self.state.lock().unwrap();
        if s.profile.provider != Kind::OpenaiResponses
            || !s.profile.enabled
            || !s.profile.remote_consent
        {
            return None;
        }
        s.profile.remote_consent = false;
        Some(Permit {
            generation: s.generation,
            expires: Instant::now() + Duration::from_secs(30),
            cancel,
            input_digest: Sha256::digest(utterance.as_bytes()).into(),
            binding: Arc::new(Mutex::new(PermitBinding {
                room: None,
                request: None,
                owner: None,
                spent: false,
            })),
        })
    }
    pub fn with_generation<T>(&self, generation: u64, f: impl FnOnce() -> T) -> Result<T, Failure> {
        let _transaction = self.transaction.lock().unwrap();
        if self.generation() != generation {
            return Err(Failure::Cancelled);
        }
        Ok(f())
    }
    pub fn profile(&self) -> Profile {
        self.state.lock().unwrap().profile.clone()
    }
    /// Only trusted UI/configuration code calls this. Consent is not a composition grant.
    pub fn configure(&self, profile: Profile) -> Result<(), Failure> {
        profile.validate()?;
        let _transaction = self.transaction.lock().unwrap();
        let mut s = self.state.lock().unwrap();
        s.generation = s.generation.checked_add(1).ok_or(Failure::BudgetExceeded)?;
        for active in s.active.values() {
            active.cancel.cancel();
        }
        s.profile = profile;
        Ok(())
    }
    pub fn last_sequence(&self) -> u64 {
        self.state.lock().unwrap().sequence
    }
    pub fn latest(&self, room: &str) -> Option<Audit> {
        self.state
            .lock()
            .unwrap()
            .audit
            .iter()
            .rev()
            .find(|a| a.room == room)
            .cloned()
    }
    pub fn audits(&self, room: &str) -> Vec<Audit> {
        self.state
            .lock()
            .unwrap()
            .audit
            .iter()
            .filter(|a| a.room == room)
            .cloned()
            .collect()
    }
    fn snapshot(&self) -> (Profile, u64) {
        let s = self.state.lock().unwrap();
        (s.profile.clone(), s.generation)
    }
    fn authorize(&self, generation: u64) -> Result<(), Failure> {
        let s = self.state.lock().unwrap();
        if s.generation != generation {
            return Err(Failure::Cancelled);
        }
        s.profile.validate()?;
        if s.profile.provider != Kind::OpenaiResponses || !s.profile.enabled {
            return Err(Failure::NotConfigured);
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct LiveProvider {
    control: Arc<Control>,
    credentials: Arc<dyn CredentialSource>,
    transport: Arc<dyn Transport>,
    room: String,
    permit: Option<Permit>,
    pub retry_transient: bool,
}
impl LiveProvider {
    pub fn new(
        control: Arc<Control>,
        credentials: Arc<dyn CredentialSource>,
        transport: Arc<dyn Transport>,
        room: String,
        permit: Option<Permit>,
    ) -> Result<Self, Failure> {
        if !wb_storage::vs2::identifier(&room) {
            return Err(Failure::NotConfigured);
        }
        if let Some(p) = &permit {
            p.bind_room(&room)?;
        }
        Ok(Self {
            permit,
            control,
            credentials,
            transport,
            room,
            retry_transient: false,
        })
    }
}
impl OperatorProvider for LiveProvider {
    fn kind(&self) -> &'static str {
        "openai-responses"
    }
    fn suggest<'a>(&'a self, request: ProviderRequest, cancel: Cancellation) -> ProviderFuture<'a> {
        Box::pin(Invocation::new(self.clone(), request, cancel))
    }
}
fn legacy(f: Failure) -> OperatorError {
    match f {
        Failure::Cancelled => OperatorError::Cancelled,
        Failure::Timeout => OperatorError::Timeout,
        Failure::SchemaMismatch | Failure::MalformedResponse => OperatorError::InvalidSuggestion,
        Failure::BudgetExceeded => OperatorError::RequestTooLarge,
        _ => OperatorError::Unavailable,
    }
}
struct Invocation {
    credential: Option<crate::policy::Secret>,
    redact_correlation: bool,
    owner: Arc<()>,
    claimed: bool,
    provider: LiveProvider,
    request: ProviderRequest,
    cancel: Cancellation,
    profile: Profile,
    generation: u64,
    started: Instant,
    deadline: Instant,
    retry_at: Instant,
    attempts: u8,
    exchange: Option<Box<dyn Exchange>>,
    body: Vec<u8>,
    leased: bool,
    done: bool,
}
impl Invocation {
    fn new(provider: LiveProvider, request: ProviderRequest, cancel: Cancellation) -> Self {
        let (profile, generation) = provider.control.snapshot();
        let now = Instant::now();
        let deadline = now + Duration::from_millis(profile.budget.timeout_ms.min(30000));
        Self {
            credential: None,
            redact_correlation: false,
            owner: Arc::new(()),
            claimed: false,
            provider,
            request,
            cancel,
            profile,
            generation,
            started: now,
            deadline,
            retry_at: now,
            attempts: 0,
            exchange: None,
            body: vec![],
            leased: false,
            done: false,
        }
    }
    fn check(&self) -> Result<(), Failure> {
        if self.cancel.is_cancelled() {
            return Err(Failure::Cancelled);
        }
        if Instant::now() >= self.deadline {
            return Err(Failure::Timeout);
        }
        self.provider.control.authorize(self.generation)?;
        let p = self
            .provider
            .permit
            .as_ref()
            .ok_or(Failure::ConsentRequired)?;
        if p.generation != self.generation {
            return Err(Failure::Cancelled);
        }
        p.scope(&self.provider.room, &self.request, &self.cancel)?;
        if self.claimed && !p.owned(&self.request.request_id, &self.owner) {
            return Err(Failure::ConsentRequired);
        }
        Ok(())
    }
    fn record(&mut self, result: Result<&response::Parsed, Failure>) {
        if self.done {
            return;
        }
        self.done = true;
        if let Some(p) = &self.provider.permit {
            p.finish(&self.owner);
        }
        let (status, input_tokens, output_tokens) = match result {
            Ok(p) => ("OK".into(), p.input_tokens, p.output_tokens),
            Err(f) => (f.code().into(), None, None),
        };
        let mut audit = Audit {
            schema_id: "OperatorSuggestionV0.1",
            sequence: 0,
            room: self.provider.room.clone(),
            generation: self.generation,
            budget: self.profile.budget.clone(),
            attempts: self.attempts,
            observation: Observation {
                observation_version: "0.1",
                correlation_id: if self.redact_correlation {
                    "<redacted>".into()
                } else {
                    self.request.request_id.clone()
                },
                provider: "openai-responses",
                model: self.profile.model.clone(),
                status,
                duration_ms: self.started.elapsed().as_millis().min(u64::MAX as u128) as u64,
                catalog_count: self.request.candidates.len().min(16),
                input_tokens,
                output_tokens,
            },
        };
        let mut s = self.provider.control.state.lock().unwrap();
        if s.audit.len() == 128 {
            s.audit.pop_front();
        }
        s.sequence = s.sequence.saturating_add(1);
        audit.sequence = s.sequence;
        s.audit.push_back(audit);
    }
    fn fail(&mut self, f: Failure) -> Poll<Result<Vec<u8>, OperatorError>> {
        self.exchange.take();
        self.record(Err(f));
        Poll::Ready(Err(legacy(f)))
    }
    fn lease(&mut self) -> Result<(), Failure> {
        let mut s = self.provider.control.state.lock().unwrap();
        if s.generation != self.generation {
            return Err(Failure::Cancelled);
        }
        if s.active.contains_key(&self.provider.room) || s.active.len() >= 4 {
            return Err(Failure::BudgetExceeded);
        }
        s.active.insert(
            self.provider.room.clone(),
            Active {
                request_id: self.request.request_id.clone(),
                cancel: self.cancel.clone(),
            },
        );
        self.leased = true;
        Ok(())
    }
}
impl Future for Invocation {
    type Output = Result<Vec<u8>, OperatorError>;
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if let Err(f) = self.check() {
            return self.fail(f);
        }
        if !self.leased {
            if !self.claimed {
                if let Err(f) = self
                    .provider
                    .permit
                    .as_ref()
                    .unwrap()
                    .claim(&self.request.request_id, &self.owner)
                {
                    return self.fail(f);
                }
                self.claimed = true;
            }
            match response::request_body(&self.request, &self.profile.model, &self.profile.budget) {
                Ok(body) => self.body = body,
                Err(f) => return self.fail(f),
            }
            if let Err(f) = self.lease() {
                return self.fail(f);
            }
        }
        if Instant::now() < self.retry_at {
            return Poll::Pending;
        }
        if self.exchange.is_none() {
            if self.attempts >= self.profile.budget.max_calls.min(2) {
                return self.fail(Failure::BudgetExceeded);
            }
            if !self.provider.transport.available() {
                return self.fail(Failure::NotConfigured);
            }
            let key = match self.provider.credentials.load() {
                Ok(k) => k,
                Err(f) => return self.fail(f),
            };
            let request_bytes =
                serde_json::to_vec(&self.request).map_err(|_| OperatorError::InvalidSuggestion)?;
            if crate::policy::contains_secret(&request_bytes, &key)
                || crate::policy::contains_secret(&self.body, &key)
            {
                self.redact_correlation = true;
                return self.fail(Failure::SchemaMismatch);
            }
            self.credential = Some(key);
            if let Err(f) = self.check() {
                return self.fail(f);
            }
            if let Err(f) = self.provider.control.admit(self.generation) {
                return self.fail(f);
            }
            self.attempts += 1;
            let timeout_ms = self
                .deadline
                .saturating_duration_since(Instant::now())
                .as_millis()
                .clamp(1, 30000) as u64;
            match self.provider.transport.start(
                &self.body,
                self.credential.as_ref().unwrap().copy_for_transport(),
                self.profile.budget.max_response_bytes,
                timeout_ms,
            ) {
                Ok(exchange) => self.exchange = Some(exchange),
                Err(f) => return self.fail(f),
            }
        }
        if let Err(f) = self.check() {
            return self.fail(f);
        }
        let result = match self.exchange.as_mut().unwrap().poll() {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(r) => r,
        };
        self.exchange.take();
        if let Err(f) = self.check() {
            return self.fail(f);
        }
        let raw = match result {
            Ok(r) => r,
            Err(f) => return self.fail(f),
        };
        if self
            .credential
            .as_ref()
            .is_some_and(|key| crate::policy::contains_secret(&raw.body, key))
        {
            return self.fail(if raw.status == 401 || raw.status == 403 {
                Failure::AuthFailed
            } else {
                Failure::SchemaMismatch
            });
        }
        if raw.body.len() > self.profile.budget.max_response_bytes {
            return self.fail(Failure::BudgetExceeded);
        }
        if self.provider.retry_transient
            && self.attempts < self.profile.budget.max_calls.min(2)
            && (raw.status == 429 || (500..=599).contains(&raw.status))
        {
            let delay = raw.retry_after_ms.unwrap_or(0);
            if delay <= 2000 && Instant::now() + Duration::from_millis(delay) < self.deadline {
                self.retry_at = Instant::now() + Duration::from_millis(delay);
                return Poll::Pending;
            }
        }
        match response::parse(
            raw.status,
            &raw.body,
            &self.profile.budget,
            &self.request,
            &self.profile.model,
        ) {
            Err(f) => self.fail(f),
            Ok(parsed) => {
                if let Err(f) = self.check() {
                    return self.fail(f);
                }
                self.record(Ok(&parsed));
                Poll::Ready(Ok(parsed.suggestion))
            }
        }
    }
}
impl Drop for Invocation {
    fn drop(&mut self) {
        self.exchange.take();
        if !self.done {
            self.record(Err(if self.cancel.is_cancelled() {
                Failure::Cancelled
            } else if Instant::now() >= self.deadline {
                Failure::Timeout
            } else {
                Failure::Cancelled
            }));
        }
        if self.leased {
            let mut s = self.provider.control.state.lock().unwrap();
            if s.active.get(&self.provider.room).is_some_and(|a| {
                a.request_id == self.request.request_id && a.cancel.same_request(&self.cancel)
            }) {
                s.active.remove(&self.provider.room);
            }
        }
        self.body.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{Secret, TESTED_MODEL};
    use crate::transport::{MockReply, MockTransport};
    use serde_json::json;
    use std::task::Waker;
    use wb_operator::provider::Candidate;
    struct Key(AtomicU64);
    impl CredentialSource for Key {
        fn load(&self) -> Result<Secret, Failure> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Secret::from_bytes(b"synthetic-concurrency-key".to_vec())
        }
    }
    #[test]
    fn llm_014_020_concurrent_room_and_global_leases_are_bounded_and_isolated() {
        let utterance = "I need a mortgage calculator for a £250,000 loan over 25 years at 5%.";
        let control = Arc::new(Control::default());
        control
            .configure(Profile {
                provider: Kind::OpenaiResponses,
                model: TESTED_MODEL.into(),
                ..Profile::default()
            })
            .unwrap();
        let suggestion = json!({"suggestionVersion":"0.1","candidateKey":"one","inputValues":wb_operator::suggestion::mortgage_terms(utterance).unwrap(),"explanation":"Explicit"});
        let body=serde_json::to_vec(&json!({"model":TESTED_MODEL,"status":"completed","output":[{"type":"message","role":"assistant","status":"completed","content":[{"type":"output_text","text":suggestion.to_string()}]}]})).unwrap();
        let transport = Arc::new(MockTransport::new(
            (0..4)
                .map(|_| MockReply {
                    status: 200,
                    body: body.clone(),
                    delay: Duration::from_millis(30),
                    retry_after_ms: None,
                    failure: None,
                })
                .collect(),
        ));
        let key = Arc::new(Key(0.into()));
        // Test-only independent trusted consent issuance at one generation allows
        // exercising every lease bound. The UI's configure path is stricter: it
        // revokes prior-generation requests whenever the next consent is armed.
        let rooms = ["room-a", "room-b", "room-c", "room-d", "room-e", "room-a"];
        let cancellations: Vec<_> = (0..6).map(|_| Cancellation::default()).collect();
        let providers: Vec<_> = rooms
            .iter()
            .enumerate()
            .map(|(i, room)| {
                let permit = Permit {
                    generation: control.generation(),
                    expires: Instant::now() + Duration::from_secs(30),
                    cancel: cancellations[i].clone(),
                    input_digest: Sha256::digest(utterance.as_bytes()).into(),
                    binding: Arc::new(Mutex::new(PermitBinding {
                        room: None,
                        request: None,
                        owner: None,
                        spent: false,
                    })),
                };
                LiveProvider::new(
                    control.clone(),
                    key.clone(),
                    transport.clone(),
                    (*room).into(),
                    Some(permit),
                )
                .unwrap()
            })
            .collect();
        let mut futures: Vec<_> = providers
            .iter()
            .enumerate()
            .map(|(i, p)| {
                p.suggest(
                    ProviderRequest {
                        request_version: "0.1".into(),
                        request_id: format!("request-{i}"),
                        utterance: utterance.into(),
                        candidates: vec![Candidate {
                            candidate_key: "one".into(),
                            capability: "https://workbench.example/ns/0.1#CalculateMortgagePayment"
                                .into(),
                            inputs: vec![
                                "principal".into(),
                                "annual_rate_percent".into(),
                                "term_months".into(),
                            ],
                            has_semantic_ui: true,
                        }],
                    },
                    cancellations[i].clone(),
                )
            })
            .collect();
        let mut cx = Context::from_waker(Waker::noop());
        assert!(futures[0].as_mut().poll(&mut cx).is_pending());
        // Prove the room bound independently, while the global cap is not full.
        assert!(matches!(
            futures[5].as_mut().poll(&mut cx),
            Poll::Ready(Err(_))
        ));
        assert_eq!(key.0.load(Ordering::SeqCst), 1);
        assert_eq!(transport.attempts.load(Ordering::SeqCst), 1);
        for f in futures.iter_mut().take(4).skip(1) {
            assert!(f.as_mut().poll(&mut cx).is_pending());
        }
        assert!(matches!(
            futures[4].as_mut().poll(&mut cx),
            Poll::Ready(Err(_))
        ));
        assert_eq!(key.0.load(Ordering::SeqCst), 4);
        assert_eq!(transport.attempts.load(Ordering::SeqCst), 4);
        assert_eq!(
            control.latest("room-e").unwrap().observation.status,
            "BUDGET_EXCEEDED"
        );
        assert_eq!(
            control.latest("room-a").unwrap().observation.status,
            "BUDGET_EXCEEDED"
        );
        cancellations[0].cancel();
        assert!(matches!(
            futures[0].as_mut().poll(&mut cx),
            Poll::Ready(Err(_))
        ));
        std::thread::sleep(Duration::from_millis(40));
        for (i, f) in futures.iter_mut().enumerate().take(4).skip(1) {
            assert!(matches!(f.as_mut().poll(&mut cx), Poll::Ready(Ok(_))));
            let audit = control.latest(rooms[i]).unwrap();
            assert_eq!(audit.observation.status, "OK");
            assert_eq!(audit.observation.correlation(), format!("request-{i}"));
            assert!(!cancellations[i].is_cancelled());
        }
        assert_eq!(
            control.latest("room-a").unwrap().observation.status,
            "CANCELLED"
        );
        drop(futures);
        assert!(control.state.lock().unwrap().active.is_empty());
        assert_eq!(transport.aborted.load(Ordering::SeqCst), 1);
    }
}
