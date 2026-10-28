//! Provider selection belongs to the host, not the model or room snapshot.
#[cfg(feature = "test-transport")]
use crate::policy::Secret;
use crate::{
    policy::{
        Budget, CredentialSource, Failure, HostCredential, Kind, Profile, ENDPOINT, TESTED_MODEL,
    },
    provider::{Control, LiveProvider, Permit},
    transport::{TlsTransport, Transport},
};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc};
use wb_operator::provider::{DisabledProvider, OfflineFixture, OperatorProvider};

pub struct Host {
    pub control: Arc<Control>,
    credentials: Arc<dyn CredentialSource>,
    transport: Arc<dyn Transport>,
    model: Option<String>,
    budget: Option<Budget>,
    pub transport_kind: &'static str,
}
impl Host {
    pub fn from_env(python: PathBuf) -> Result<Self, Failure> {
        let model = match std::env::var("WB_LLM_MODEL") {
            Ok(m) if m == TESTED_MODEL => Some(m),
            Err(std::env::VarError::NotPresent) => Some(TESTED_MODEL.into()),
            _ => None,
        };
        let budget = match std::env::var("WB_LLM_BUDGET") {
            Ok(s) => wb_contracts::ui::strict_json(s.as_bytes())
                .ok()
                .and_then(|v| serde_json::from_value::<Budget>(v).ok())
                .filter(|b| b.validate().is_ok()),
            Err(_) => Some(Budget::default()),
        };
        let credentials: Arc<dyn CredentialSource> =
            Arc::new(match std::env::var_os("WB_LLM_SECRET_FILE") {
                Some(p) => HostCredential::PrivateFile(p.into()),
                None => HostCredential::Environment,
            });
        let control = Arc::new(Control::default());
        let transport: Arc<dyn Transport> = Arc::new(TlsTransport::new(python)?);
        let result = Self {
            control,
            credentials,
            transport,
            model,
            budget,
            transport_kind: "https-production",
        };
        if let Ok(mode) = std::env::var("WB_LLM_TEST_TRANSPORT") {
            #[cfg(feature = "test-transport")]
            {
                let transport = match FixtureTransport::new(mode) {
                    Ok(t) => t,
                    Err(_) => {
                        return Ok(Self {
                            model: None,
                            budget: None,
                            transport_kind: "not-configured",
                            ..result
                        })
                    }
                };
                return Ok(Self {
                    transport: Arc::new(transport),
                    credentials: Arc::new(FixtureCredential),
                    transport_kind: "deterministic-mock",
                    ..result
                });
            }
            #[cfg(not(feature = "test-transport"))]
            {
                let _ = mode;
                return Ok(Self {
                    model: None,
                    budget: None,
                    transport_kind: "not-configured",
                    ..result
                });
            }
        }
        Ok(result)
    }
    #[cfg(feature = "test-transport")]
    pub fn fixture(mode: &str, budget: Budget) -> Result<Self, Failure> {
        budget.validate()?;
        Ok(Self {
            control: Arc::new(Control::default()),
            credentials: Arc::new(FixtureCredential),
            transport: Arc::new(FixtureTransport::new(mode.into())?),
            model: Some(TESTED_MODEL.into()),
            budget: Some(budget),
            transport_kind: "deterministic-mock",
        })
    }
    pub fn configure(&self, provider: Kind, consent: bool) -> Result<(), Failure> {
        let model = match provider {
            Kind::OfflineFixture => "deterministic-fixture".into(),
            Kind::OpenaiResponses => self.model.clone().ok_or(Failure::NotConfigured)?,
        };
        self.control.configure(Profile {
            profile_version: "0.1".into(),
            remote_consent: provider == Kind::OpenaiResponses && consent,
            enabled: true,
            model,
            budget: if provider == Kind::OfflineFixture {
                Budget::default()
            } else {
                self.budget.clone().ok_or(Failure::NotConfigured)?
            },
            provider,
        })
    }
    pub fn provider(
        &self,
        room: &str,
        permit: Option<Permit>,
    ) -> Result<Arc<dyn OperatorProvider>, Failure> {
        match self.control.profile().provider {
            Kind::OfflineFixture => Ok(
                if std::env::var("WB_OPERATOR_PROVIDER").as_deref() == Ok("disabled") {
                    Arc::new(DisabledProvider)
                } else {
                    Arc::new(OfflineFixture)
                },
            ),
            Kind::OpenaiResponses => Ok(Arc::new(LiveProvider::new(
                self.control.clone(),
                self.credentials.clone(),
                self.transport.clone(),
                room.into(),
                permit,
            )?)),
        }
    }
    pub fn status(&self, token: &str, result: Result<(), Failure>) -> Value {
        json!({"protocol":"provider.1","kind":"reply","token":token,"ok":result.is_ok(),"code":result.err().map_or("OK",Failure::code),"profile":self.control.profile(),"endpoint":ENDPOINT,"remoteModel":self.model.as_deref().unwrap_or("NOT_CONFIGURED"),"remoteAttempts":self.control.network_calls.load(std::sync::atomic::Ordering::SeqCst).to_string(),"transport":self.transport_kind})
    }
}
#[cfg(feature = "test-transport")]
struct FixtureCredential;
#[cfg(feature = "test-transport")]
impl CredentialSource for FixtureCredential {
    fn load(&self) -> Result<Secret, Failure> {
        Secret::from_bytes(b"fixture-only-not-a-real-key".to_vec())
    }
}
#[cfg(feature = "test-transport")]
struct FixtureTransport {
    mode: String,
    started: std::sync::atomic::AtomicU64,
}
#[cfg(feature = "test-transport")]
impl FixtureTransport {
    fn new(mode: String) -> Result<Self, Failure> {
        if ![
            "success",
            "refused",
            "incomplete",
            "auth",
            "tool",
            "invalid",
            "slow",
            "unavailable",
        ]
        .contains(&mode.as_str())
        {
            return Err(Failure::NotConfigured);
        }
        Ok(Self {
            mode,
            started: 0.into(),
        })
    }
}
#[cfg(feature = "test-transport")]
impl Transport for FixtureTransport {
    fn start(
        &self,
        body: &[u8],
        _: Secret,
        _: usize,
        _: u64,
    ) -> Result<Box<dyn crate::transport::Exchange>, Failure> {
        use crate::transport::{MockReply, MockTransport};
        self.started
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let body: Value = serde_json::from_slice(body).map_err(|_| Failure::MalformedResponse)?;
        let input: Value = serde_json::from_str(
            body["input"][0]["content"]
                .as_str()
                .ok_or(Failure::MalformedResponse)?,
        )
        .map_err(|_| Failure::MalformedResponse)?;
        let utterance = input["utterance"]
            .as_str()
            .ok_or(Failure::MalformedResponse)?;
        let values = wb_operator::suggestion::mortgage_terms(utterance)
            .map_err(|_| Failure::SchemaMismatch)?;
        let candidates: Vec<wb_operator::provider::Candidate> =
            serde_json::from_value(input["candidates"].clone())
                .map_err(|_| Failure::SchemaMismatch)?;
        let candidates: Vec<_> = candidates
            .iter()
            .filter(|c| {
                c.has_semantic_ui
                    && c.capability == "https://workbench.example/ns/0.1#CalculateMortgagePayment"
                    && values.keys().all(|k| c.inputs.contains(k))
            })
            .collect();
        if candidates.len() != 1 {
            return Err(Failure::SchemaMismatch);
        }
        let suggestion = json!({"suggestionVersion":"0.1","candidateKey":candidates[0].candidate_key,"inputValues":values,"explanation":"Mocked transport only. Host validation and user approval are required."});
        let mut response = json!({"model":TESTED_MODEL,"status":"completed","error":null,"incomplete_details":null,"tools":[],"output":[{"type":"message","role":"assistant","status":"completed","content":[{"type":"output_text","text":serde_json::to_string(&suggestion).unwrap()}]}],"usage":{"input_tokens":100,"output_tokens":100}});
        let mut status = 200;
        let mut delay = std::time::Duration::ZERO;
        let mut failure = None;
        match self.mode.as_str() {
            "refused" => {
                response["output"][0]["content"] =
                    json!([{"type":"refusal","refusal":"Never log this raw text"}])
            }
            "incomplete" => {
                response["status"] = json!("incomplete");
                response["incomplete_details"] = json!({"reason":"max_output_tokens"});
            }
            "auth" => {
                status = 401;
                response = json!({"error":{"message":"fixture-only-not-a-real-key"}});
            }
            "tool" => response["output"]
                .as_array_mut()
                .unwrap()
                .push(json!({"type":"function_call","name":"install"})),
            "invalid" => response["output"][0]["content"][0]["text"] = json!("{}"),
            "slow" => delay = std::time::Duration::from_secs(5),
            "unavailable" => failure = Some(Failure::Unavailable),
            _ => {}
        }
        let t = MockTransport::new(vec![MockReply {
            status,
            body: serde_json::to_vec(&response).unwrap(),
            delay,
            retry_after_ms: None,
            failure,
        }]);
        t.start(&[], Secret::from_bytes(b"mock".to_vec())?, 32768, 20000)
    }
}
