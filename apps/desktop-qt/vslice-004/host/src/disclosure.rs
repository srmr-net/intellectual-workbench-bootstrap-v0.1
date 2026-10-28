//! Read-only preflight and one-use consent binding. No alternative plan/executor.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    os::fd::{AsRawFd, FromRawFd},
    os::unix::fs::MetadataExt,
    path::{Component, Path},
    sync::Arc,
    time::{Duration, Instant},
};
use wb_live_provider::{
    policy::{Budget, Kind, OPENROUTER_MODEL},
    response::request_body_for,
};
use wb_operator::{
    catalog::Catalog,
    provider::{Cancellation, OperatorProvider, ProviderFuture, ProviderRequest},
    suggestion::{mortgage_terms, OperatorError, Suggestion},
};
use wb_registry::Registry;
use wb_storage::vs2::{Workspace, MAX_BYTES};
const OPAQUE: &str = "<fresh host-issued opaque selector generated at send>";
fn hash(v: &Value) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(v).expect("JSON value"))
    )
}

/// Read the current durable CAS while accepted Desktop retains its writer lock.
/// No writer/migration/recovery is added; accepted Workspace::decode is authority.
pub struct ReadView {
    dir: File,
}
impl ReadView {
    pub fn open(path: &Path) -> Result<Self, &'static str> {
        if !path.is_absolute() {
            return Err("SNAPSHOT_PATH");
        }
        let mut dir = File::open("/").map_err(|_| "SNAPSHOT_IO")?;
        for part in path.components() {
            match part {
                Component::RootDir => {}
                Component::Normal(name) => {
                    use std::os::unix::ffi::OsStrExt;
                    let name =
                        std::ffi::CString::new(name.as_bytes()).map_err(|_| "SNAPSHOT_PATH")?;
                    let fd = unsafe {
                        libc::openat(
                            dir.as_raw_fd(),
                            name.as_ptr(),
                            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                        )
                    };
                    if fd < 0 {
                        return Err("SNAPSHOT_IO");
                    }
                    dir = unsafe { File::from_raw_fd(fd) };
                }
                _ => return Err("SNAPSHOT_PATH"),
            }
        }
        let m = dir.metadata().map_err(|_| "SNAPSHOT_IO")?;
        if m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o077 != 0 {
            return Err("SNAPSHOT_PRIVATE");
        }
        Ok(Self { dir })
    }
    pub fn current(&self) -> Result<(u64, String), &'static str> {
        let fd = unsafe {
            libc::openat(
                self.dir.as_raw_fd(),
                c"workspace.json".as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::NotFound {
                return Ok(stamp(&Workspace::default()));
            }
            return Err("SNAPSHOT_IO");
        }
        let file = unsafe { File::from_raw_fd(fd) };
        let m = file.metadata().map_err(|_| "SNAPSHOT_IO")?;
        if !m.is_file()
            || m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o077 != 0
            || m.nlink() != 1
            || m.len() > MAX_BYTES as u64
        {
            return Err("INVALID_SNAPSHOT");
        }
        let mut bytes = Vec::new();
        file.take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "SNAPSHOT_IO")?;
        let workspace = Workspace::decode(&bytes)?;
        Ok(stamp(&workspace))
    }
}
pub fn stamp(workspace: &Workspace) -> (u64, String) {
    (workspace.revision, workspace.content_sha256.clone())
}
fn catalog_state(
    registry: &Registry,
    utterance: &str,
) -> Result<(ProviderRequest, Value), OperatorError> {
    let catalog = Catalog::discover(registry)?;
    let candidates = catalog.candidates();
    let inputs = mortgage_terms(utterance)?;
    let mut pins = Vec::new();
    for c in &candidates {
        let s = catalog.validate(
            registry,
            &Suggestion {
                suggestion_version: "0.1".into(),
                candidate_key: c.candidate_key.clone(),
                input_values: inputs.clone(),
                explanation: String::new(),
            },
        )?;
        pins.push(json!({"pin":s.pin,"uiPath":s.ui_path,"uiSha256":s.ui_sha256,"schemaSha256":s.ui_schema_sha256}));
    }
    // Catalog order is deterministic; retain multiplicity, not only a set/count.
    Ok((
        ProviderRequest {
            request_version: "0.1".into(),
            request_id: "outbound-review".into(),
            utterance: utterance.into(),
            candidates,
        },
        Value::Array(pins),
    ))
}
/// Only keys in their declared fields and fixed host-schema context are normalized.
/// User utterance and all other dynamic data must compare byte-for-byte as JSON.
pub fn normalized_body(request: &ProviderRequest, budget: &Budget) -> Result<Value, OperatorError> {
    let bytes = request_body_for(&Kind::OpenrouterChat, request, OPENROUTER_MODEL, budget)
        .map_err(|_| OperatorError::RequestTooLarge)?;
    let mut body: Value =
        serde_json::from_slice(&bytes).map_err(|_| OperatorError::InvalidSuggestion)?;
    let mut selectors = BTreeSet::new();
    for c in &request.candidates {
        if c.candidate_key.len() != 64
            || !c
                .candidate_key
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !selectors.insert(c.candidate_key.clone())
        {
            return Err(OperatorError::CatalogChanged);
        }
    }
    let mut instructions = body["messages"][0]["content"]
        .as_str()
        .ok_or(OperatorError::InvalidSuggestion)?
        .to_owned();
    for key in &selectors {
        instructions = instructions.replace(key, OPAQUE);
    }
    body["messages"][0]["content"] = json!(instructions);
    let mut input: Value = serde_json::from_str(
        body["messages"][1]["content"]
            .as_str()
            .ok_or(OperatorError::InvalidSuggestion)?,
    )
    .map_err(|_| OperatorError::InvalidSuggestion)?;
    for candidate in input["candidates"]
        .as_array_mut()
        .ok_or(OperatorError::InvalidSuggestion)?
    {
        candidate["candidateKey"] = json!(OPAQUE);
    }
    body["messages"][1]["content"] =
        json!(serde_json::to_string(&input).map_err(|_| OperatorError::InvalidSuggestion)?);
    Ok(body)
}
#[derive(Clone)]
pub struct Review {
    pub id: String,
    pub digest: String,
    pub utterance: String,
    pub generation: u64,
    pub workspace: (u64, String),
    pub view: Value,
    body: Value,
    pins: Value,
    expires: Instant,
    budget: Budget,
}
impl Review {
    pub fn create(
        registry: &Registry,
        read: &ReadView,
        workspace: &Workspace,
        utterance: &str,
        generation: u64,
        budget: Budget,
    ) -> Result<Self, String> {
        if read.current().map_err(str::to_string)? != stamp(workspace) {
            return Err("STATE_CONFLICT".into());
        }
        let (request, pins) =
            catalog_state(registry, utterance).map_err(|e| e.code().to_string())?;
        let body = normalized_body(&request, &budget).map_err(|e| e.code().to_string())?;
        let id = hash(
            &json!({"selectors":request.candidates.iter().map(|c|&c.candidate_key).collect::<Vec<_>>(),"generation":generation,"workspace":stamp(workspace)}),
        );
        let digest = hash(
            &json!({"id":id,"body":body,"pins":pins,"workspace":stamp(workspace),"generation":generation}),
        );
        let mut settings = body.clone();
        let messages = settings
            .as_object_mut()
            .unwrap()
            .remove("messages")
            .unwrap();
        let input: Value = serde_json::from_str(messages[1]["content"].as_str().unwrap()).unwrap();
        let view = json!({"reviewId":id,"reviewDigest":digest,"expiresInSeconds":120,"outboundData":input,"fixedHostContext":messages[0]["content"],"requestSettings":settings,"selectorPolicy":"Each candidateKey is a fresh host-issued opaque selector generated at send, derived from a host nonce and local identity. Displayed placeholders are not the future token bytes.","exclusions":"No prior room documents, history, paths or credential values are model input. Authentication uses the separate approved host source.","providerDataUse":"OpenRouter routes only to NVIDIA's free endpoint; NVIDIA logs requests for security and product improvement. No provider JSON Schema mode; canonical host validation remains mandatory.","workspaceRevision":workspace.revision.to_string(),"candidateCount":request.candidates.len()});
        Ok(Self {
            id,
            digest,
            utterance: utterance.into(),
            generation,
            workspace: stamp(workspace),
            view,
            body,
            pins,
            expires: Instant::now() + Duration::from_secs(120),
            budget,
        })
    }
    pub fn authorize(&self, id: &str, digest: &str, generation: u64) -> bool {
        id == self.id
            && digest == self.digest
            && generation == self.generation
            && Instant::now() < self.expires
    }
    fn validate_request(
        &self,
        request: &ProviderRequest,
        registry: &Registry,
        read: &ReadView,
    ) -> Result<(), OperatorError> {
        if Instant::now() >= self.expires {
            return Err(OperatorError::Timeout);
        }
        if request.utterance != self.utterance
            || normalized_body(request, &self.budget)? != self.body
        {
            return Err(OperatorError::CatalogChanged);
        }
        if read.current().map_err(|_| OperatorError::Unavailable)? != self.workspace {
            return Err(OperatorError::CatalogChanged);
        }
        let (_, pins) = catalog_state(registry, &request.utterance)?;
        if pins != self.pins {
            return Err(OperatorError::CatalogChanged);
        }
        Ok(())
    }
}
pub struct GuardedProvider {
    pub inner: Arc<dyn OperatorProvider>,
    pub review: Review,
    pub registry: Registry,
    pub read: Arc<ReadView>,
}
impl OperatorProvider for GuardedProvider {
    fn kind(&self) -> &'static str {
        self.inner.kind()
    }
    fn suggest<'a>(&'a self, request: ProviderRequest, cancel: Cancellation) -> ProviderFuture<'a> {
        Box::pin(async move {
            if cancel.is_cancelled() {
                return Err(OperatorError::Cancelled);
            }
            self.review
                .validate_request(&request, &self.registry, &self.read)?;
            if cancel.is_cancelled() {
                return Err(OperatorError::Cancelled);
            }
            self.inner.suggest(request, cancel).await
        })
    }
}
pub fn presentation(registry: &Registry, preview: &Value) -> Option<Value> {
    let pin: wb_registry::ReleasePin =
        serde_json::from_value(preview["plan"]["instances"][0]["component"].clone()).ok()?;
    let handle = registry.resolve_exact(&pin).ok()?;
    let ui = registry.read_verified_ui(&handle).ok()?;
    let fields:Vec<_>=ui.document.root.children.iter().filter(|c|c.bind.direction==wb_contracts::ui::Direction::Input).map(|c|{let field=wb_semantic::ui::field_name(&c.bind.field).ok()?;Some(json!({"field":field,"label":c.label,"unit":c.unit,"value":preview["initialInputs"][&field]}))}).collect::<Option<Vec<_>>>()?;
    let capabilities = registry
        .discover(None)
        .ok()?
        .into_iter()
        .find(|s| s.pin == pin)?
        .capabilities;
    Some(
        json!({"fields":fields,"capabilities":capabilities,"labelAuthority":"Admitted labels/units are presentation hints; host schema and semantic checks are authoritative."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use wb_operator::provider::Candidate;
    #[test]
    fn only_opaque_selectors_may_change() {
        let candidate = |key: char| Candidate {
            candidate_key: key.to_string().repeat(64),
            capability: "urn:example:capability".into(),
            inputs: vec![
                "principal".into(),
                "annual_rate_percent".into(),
                "term_months".into(),
            ],
            has_semantic_ui: true,
        };
        let a = ProviderRequest {
            request_version: "0.1".into(),
            request_id: "a".into(),
            utterance: "I need a mortgage calculator for a £250,000 loan over 25 years at 5%."
                .into(),
            candidates: vec![candidate('a')],
        };
        let mut b = a.clone();
        b.request_id = "b".into();
        b.candidates = vec![candidate('b')];
        let budget = Budget::default();
        assert_eq!(
            normalized_body(&a, &budget).unwrap(),
            normalized_body(&b, &budget).unwrap()
        );
        b.utterance = b.utterance.replace("250,000", "300,000");
        assert_ne!(
            normalized_body(&a, &budget).unwrap(),
            normalized_body(&b, &budget).unwrap()
        );
        b = a.clone();
        b.candidates.push(candidate('b'));
        assert_ne!(
            normalized_body(&a, &budget).unwrap(),
            normalized_body(&b, &budget).unwrap()
        );
        b.candidates[1] = candidate('a');
        assert!(normalized_body(&b, &budget).is_err());
    }
    #[test]
    fn durable_read_is_private_bounded_and_rejects_links() {
        let path = std::env::temp_dir().join(format!("vs4-read-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let read = ReadView::open(&path).unwrap();
        assert_eq!(read.current().unwrap(), stamp(&Workspace::default()));
        std::os::unix::fs::symlink("/dev/null", path.join("workspace.json")).unwrap();
        assert!(read.current().is_err());
        std::fs::remove_file(path.join("workspace.json")).unwrap();
        std::fs::write(path.join("workspace.json"), b"invalid").unwrap();
        assert!(read.current().is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
}
