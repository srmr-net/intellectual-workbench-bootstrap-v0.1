//! Replaceable asynchronous provider boundary. Providers receive no host handles.
use crate::suggestion::{mortgage_terms, OperatorError, Result, Suggestion};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Candidate {
    pub candidate_key: String,
    pub capability: String,
    pub inputs: Vec<String>,
    pub has_semantic_ui: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderRequest {
    pub request_version: String,
    pub request_id: String,
    pub utterance: String,
    pub candidates: Vec<Candidate>,
}
impl ProviderRequest {
    pub fn validate(&self) -> Result<()> {
        if self.utterance.len() > 4096 {
            return Err(OperatorError::RequestTooLarge);
        }
        let value = serde_json::to_value(self).map_err(|_| OperatorError::InvalidSuggestion)?;
        let schema = serde_json::from_str(include_str!(
            "../../../docs/implementation/adr-005/schemas/operator-request.schema.json"
        ))
        .expect("bundled schema");
        if !jsonschema::is_valid(&schema, &value)
            || serde_json::to_vec(&self.candidates)
                .map_err(|_| OperatorError::InvalidSuggestion)?
                .len()
                > 32 * 1024
        {
            return Err(OperatorError::InvalidSuggestion);
        }
        Ok(())
    }
}
#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
pub type ProviderFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;
pub trait OperatorProvider: Send + Sync {
    fn kind(&self) -> &'static str;
    fn suggest<'a>(&'a self, request: ProviderRequest, cancel: Cancellation) -> ProviderFuture<'a>;
}
pub struct OfflineFixture;
impl OperatorProvider for OfflineFixture {
    fn kind(&self) -> &'static str {
        "offline-fixture"
    }
    fn suggest<'a>(&'a self, request: ProviderRequest, cancel: Cancellation) -> ProviderFuture<'a> {
        Box::pin(async move {
            request.validate()?;
            if cancel.is_cancelled() {
                return Err(OperatorError::Cancelled);
            }
            let inputs = mortgage_terms(&request.utterance)?;
            let candidates: Vec<_> = request
                .candidates
                .iter()
                .filter(|c| {
                    c.has_semantic_ui
                        && c.capability
                            == "https://workbench.example/ns/0.1#CalculateMortgagePayment"
                        && inputs.keys().all(|k| c.inputs.contains(k))
                })
                .collect();
            let candidate = match candidates.as_slice() {
                [one] => one,
                [] => return Err(OperatorError::ClarificationRequired),
                _ => return Err(OperatorError::Ambiguous),
            };
            serde_json::to_vec(&Suggestion { suggestion_version: "0.1".into(), candidate_key: candidate.candidate_key.clone(), input_values: inputs, explanation: "Explicit GBP amount, annual percentage and years converted to months. Host validation and approval are required.".into() }).map_err(|_| OperatorError::InvalidSuggestion)
        })
    }
}
pub struct DisabledProvider;
impl OperatorProvider for DisabledProvider {
    fn kind(&self) -> &'static str {
        "disabled"
    }
    fn suggest<'a>(&'a self, _: ProviderRequest, _: Cancellation) -> ProviderFuture<'a> {
        Box::pin(async { Err(OperatorError::Unavailable) })
    }
}
struct ThreadWake(std::thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}
/// Host adapters must yield from poll; cancellation drops the pending future.
/// This is a cooperative deadline, not isolation for arbitrary native code.
pub fn await_provider(
    provider: &dyn OperatorProvider,
    request: ProviderRequest,
    cancel: Cancellation,
    budget: Duration,
) -> Result<Vec<u8>> {
    request.validate()?;
    let deadline = Instant::now() + budget.min(Duration::from_secs(30));
    let mut future = provider.suggest(request, cancel.clone());
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    loop {
        if cancel.is_cancelled() {
            return Err(OperatorError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(OperatorError::Timeout);
        }
        if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
            if cancel.is_cancelled() {
                return Err(OperatorError::Cancelled);
            }
            if Instant::now() >= deadline {
                return Err(OperatorError::Timeout);
            }
            let bytes = result?;
            Suggestion::parse(&bytes)?;
            return Ok(bytes);
        }
        std::thread::park_timeout(Duration::from_millis(2));
    }
}
