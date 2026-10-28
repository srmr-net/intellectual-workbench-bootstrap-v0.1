use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::PathBuf,
};

pub const ENDPOINT: &str = "https://api.openai.com/v1/responses";
pub const TESTED_MODEL: &str = "gpt-5.4-mini-2026-03-17";
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Failure {
    NotConfigured,
    ConsentRequired,
    AuthFailed,
    RateLimited,
    Timeout,
    Cancelled,
    Unavailable,
    Refused,
    Truncated,
    MalformedResponse,
    SchemaMismatch,
    BudgetExceeded,
    ProviderError,
}
impl Failure {
    pub fn code(self) -> &'static str {
        match self {
            Self::NotConfigured => "NOT_CONFIGURED",
            Self::ConsentRequired => "CONSENT_REQUIRED",
            Self::AuthFailed => "AUTH_FAILED",
            Self::RateLimited => "RATE_LIMITED",
            Self::Timeout => "TIMEOUT",
            Self::Cancelled => "CANCELLED",
            Self::Unavailable => "UNAVAILABLE",
            Self::Refused => "REFUSED",
            Self::Truncated => "TRUNCATED",
            Self::MalformedResponse => "MALFORMED_RESPONSE",
            Self::SchemaMismatch => "SCHEMA_MISMATCH",
            Self::BudgetExceeded => "BUDGET_EXCEEDED",
            Self::ProviderError => "PROVIDER_ERROR",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Budget {
    pub timeout_ms: u64,
    pub max_output_tokens: u32,
    pub max_request_bytes: usize,
    pub max_response_bytes: usize,
    pub max_catalog_candidates: usize,
    pub max_calls: u8,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            timeout_ms: 20_000,
            max_output_tokens: 512,
            max_request_bytes: 4096,
            max_response_bytes: 16384,
            max_catalog_candidates: 8,
            max_calls: 1,
        }
    }
}
impl Budget {
    pub fn validate(&self) -> Result<(), Failure> {
        if !(1000..=45000).contains(&self.timeout_ms)
            || !(1..=1024).contains(&self.max_output_tokens)
            || !(1..=8192).contains(&self.max_request_bytes)
            || !(1..=32768).contains(&self.max_response_bytes)
            || !(1..=16).contains(&self.max_catalog_candidates)
            || !(1..=3).contains(&self.max_calls)
        {
            return Err(Failure::BudgetExceeded);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    OfflineFixture,
    OpenaiResponses,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub profile_version: String,
    pub provider: Kind,
    pub model: String,
    pub enabled: bool,
    pub remote_consent: bool,
    pub budget: Budget,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            profile_version: "0.1".into(),
            provider: Kind::OfflineFixture,
            model: "deterministic-fixture".into(),
            enabled: true,
            remote_consent: false,
            budget: Budget::default(),
        }
    }
}
impl Profile {
    pub fn validate(&self) -> Result<(), Failure> {
        self.budget.validate()?;
        if self.profile_version != "0.1" || self.model.len() > 120 || self.model.is_empty() {
            return Err(Failure::NotConfigured);
        }
        match self.provider {
            Kind::OfflineFixture if self.model != "deterministic-fixture" => {
                Err(Failure::NotConfigured)
            }
            Kind::OpenaiResponses if self.model != TESTED_MODEL => Err(Failure::NotConfigured),
            _ => Ok(()),
        }
    }
}
/// Intentionally no Debug/Serialize/Clone: credentials are not telemetry or DTOs.
pub struct Secret(Vec<u8>);
impl Secret {
    pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self, Failure> {
        if bytes.is_empty() || bytes.len() > 4096 || !bytes.iter().all(|b| (33..=126).contains(b)) {
            bytes.fill(0);
            return Err(Failure::NotConfigured);
        }
        Ok(Self(bytes))
    }
    pub(crate) fn copy_for_transport(&self) -> Self {
        Self(self.0.clone())
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.0
    }
}
impl Drop for Secret {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}
pub trait CredentialSource: Send + Sync {
    fn load(&self) -> Result<Secret, Failure>;
}
pub enum HostCredential {
    Environment,
    PrivateFile(PathBuf),
    Missing,
}
impl CredentialSource for HostCredential {
    fn load(&self) -> Result<Secret, Failure> {
        let mut bytes = match self {
            Self::Environment => std::env::var_os("OPENAI_API_KEY")
                .ok_or(Failure::NotConfigured)?
                .into_encoded_bytes(),
            Self::PrivateFile(path) => {
                let mut f = OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                    .open(path)
                    .map_err(|_| Failure::NotConfigured)?;
                let m = f.metadata().map_err(|_| Failure::NotConfigured)?;
                if !path.is_absolute()
                    || !m.is_file()
                    || m.nlink() != 1
                    || m.uid() != unsafe { libc::geteuid() }
                    || m.mode() & 0o077 != 0
                    || m.len() > 4098
                {
                    return Err(Failure::NotConfigured);
                }
                let mut b = Vec::new();
                f.by_ref()
                    .take(4099)
                    .read_to_end(&mut b)
                    .map_err(|_| Failure::NotConfigured)?;
                b
            }
            Self::Missing => return Err(Failure::NotConfigured),
        };
        while bytes.last().is_some_and(|b| b.is_ascii_whitespace()) {
            bytes.pop();
        }
        Secret::from_bytes(bytes)
    }
}

/// Detect the credential in raw JSON or decoded string leaves (including the
/// nested suggestion JSON string). Fail closed instead of rewriting user intent.
pub(crate) fn contains_secret(bytes: &[u8], secret: &Secret) -> bool {
    fn visit(value: &serde_json::Value, key: &str, depth: usize) -> bool {
        match value {
            serde_json::Value::String(s) => {
                s.contains(key)
                    || (depth < 3
                        && wb_contracts::ui::strict_json(s.as_bytes())
                            .ok()
                            .is_some_and(|v| visit(&v, key, depth + 1)))
            }
            serde_json::Value::Array(a) => a.iter().any(|v| visit(v, key, depth)),
            serde_json::Value::Object(o) => o
                .iter()
                .any(|(k, v)| k.contains(key) || visit(v, key, depth)),
            _ => false,
        }
    }
    if bytes
        .windows(secret.bytes().len())
        .any(|w| w == secret.bytes())
    {
        return true;
    }
    let key = std::str::from_utf8(secret.bytes()).expect("validated ASCII credential");
    wb_contracts::ui::strict_json(bytes)
        .ok()
        .is_some_and(|v| visit(&v, key, 0))
}
