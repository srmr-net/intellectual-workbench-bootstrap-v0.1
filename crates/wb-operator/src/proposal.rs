//! Trusted canonical construction and single-use explicit approval. None of these
//! host capabilities implement Deserialize; providers return only Suggestion.
use crate::{
    catalog::{Catalog, Selection},
    suggestion::{mortgage_terms, OperatorError, Suggestion},
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};
use wb_contracts::{ipc::*, ComponentRef};
use wb_kernel::{room::Room, Kernel};
use wb_registry::Registry;

#[derive(Debug)]
pub enum ProposalError {
    Operator(OperatorError),
    Host(HostError),
    ApprovalRejected,
    Expired,
    Persistence,
}
impl From<OperatorError> for ProposalError {
    fn from(e: OperatorError) -> Self {
        Self::Operator(e)
    }
}
impl From<HostError> for ProposalError {
    fn from(e: HostError) -> Self {
        Self::Host(e)
    }
}
type Result<T> = std::result::Result<T, ProposalError>;
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub proposal_digest: String,
    pub initial_input_digest: String,
    pub plan: CompositionPlan,
    pub initial_inputs: Value,
    pub untrusted_explanation: String,
    pub effect: String,
    pub requires_approval: bool,
    pub expires_in_seconds: u64,
    pub proposal_id: String,
}
pub struct ApplyContext<'a> {
    pub kernel: &'a Kernel,
    pub registry: &'a Registry,
    pub catalog: &'a Catalog,
    pub correlation: &'a str,
}
pub struct Proposal {
    preview: Preview,
    selection: Selection,
    suggestion: Suggestion,
    deadline: Instant,
    spent: bool,
}
#[derive(Debug)]
pub struct ApplyOutcome {
    pub topology_committed: bool,
    pub initialization_succeeded: bool,
    pub initialization_error: Option<HostError>,
}
fn digest(value: &impl Serialize) -> std::result::Result<String, OperatorError> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(|_| OperatorError::InvalidSuggestion)?)
    ))
}
impl Proposal {
    pub fn prepare(
        kernel: &Kernel,
        room: &Room,
        registry: &Registry,
        catalog: &Catalog,
        utterance: &str,
        suggestion: Suggestion,
        lifetime: Duration,
    ) -> Result<Self> {
        if suggestion.input_values != mortgage_terms(utterance)? {
            return Err(OperatorError::InvalidSuggestion.into());
        }
        let selection = catalog.validate(registry, &suggestion)?;
        let handle = registry
            .resolve_exact(&selection.pin)
            .map_err(|_| OperatorError::Unavailable)?;
        let component = kernel.cache_registry_release(registry, &handle)?;
        let plan = CompositionPlan {
            plan_version: "0.1".into(),
            room_id: room.id().into(),
            expected_room_revision: room.revision(),
            instances: vec![InstanceSpec {
                instance_id: "component".into(),
                component,
            }],
            connections: vec![],
            ui: UiReference {
                definition_ref: format!("{}/{}", selection.pin.id, selection.ui_path),
            },
            requested_permissions: vec![],
        };
        kernel.validate_operator_plan(&room.snapshot(), &plan)?;
        let inputs = serde_json::to_value(&suggestion.input_values)
            .map_err(|_| OperatorError::InvalidSuggestion)?;
        let lifetime = lifetime.min(Duration::from_secs(300));
        // Include exact input values and the action alongside the canonical plan;
        // consent for topology alone must not authorize different initialization.
        use std::io::Read;
        let mut nonce = [0u8; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut nonce))
            .map_err(|_| OperatorError::Unavailable)?;
        let proposal_id = digest(&nonce)?;
        let proposal_digest = digest(&(
            "apply-and-initialize",
            &proposal_id,
            lifetime.as_nanos(),
            &plan,
            &inputs,
            &selection.ui_sha256,
            &selection.ui_schema_sha256,
        ))?;
        Ok(Self { preview: Preview { proposal_digest, initial_input_digest: digest(&inputs)?, plan, initial_inputs: inputs, untrusted_explanation: suggestion.explanation.clone(), effect: "Add one installed component to this empty room, then initialize through typed UI ingress.".into(), requires_approval: true, expires_in_seconds: lifetime.as_secs(), proposal_id }, selection, suggestion, deadline: Instant::now() + lifetime, spent: false })
    }
    pub fn preview(&self) -> &Preview {
        &self.preview
    }
    pub fn decline(&mut self) {
        self.spent = true;
    }
    /// Only an explicit trusted UI confirmation may invoke this method. The
    /// digest is comparison data, never an execution credential from a provider.
    /// Persist the topology checkpoint in `checkpoint` before initialization.
    /// Failure after apply is partial completion, never a claimed rollback.
    pub fn approve<F>(
        &mut self,
        confirmed_digest: &str,
        room: &mut Room,
        context: ApplyContext<'_>,
        mut checkpoint: F,
    ) -> Result<ApplyOutcome>
    where
        F: FnMut(&Room, &Selection) -> std::result::Result<(), ProposalError>,
    {
        let ApplyContext {
            kernel,
            registry,
            catalog,
            correlation,
        } = context;
        if self.spent {
            return Err(ProposalError::ApprovalRejected);
        }
        self.spent = true;
        if confirmed_digest != self.preview.proposal_digest {
            return Err(ProposalError::ApprovalRejected);
        }
        if Instant::now() >= self.deadline {
            return Err(ProposalError::Expired);
        }
        let selection = catalog.validate(registry, &self.suggestion)?;
        if selection.pin != self.selection.pin || selection.ui_sha256 != self.selection.ui_sha256 {
            return Err(OperatorError::Unavailable.into());
        }
        kernel.validate_operator_plan(&room.snapshot(), &self.preview.plan)?;
        let ComponentRef {
            id,
            version,
            sha256,
        } = &self.preview.plan.instances[0].component;
        let pin = ComponentRef {
            id: id.clone(),
            version: version.clone(),
            sha256: sha256.clone(),
        };
        let preexisting = kernel.operator_execution_approved(room.id(), "component", &pin);
        kernel.approve_execution(room.id(), "component", &pin)?;
        if let Err(error) =
            room.apply_plan(self.preview.plan.clone(), "operator.approved", correlation)
        {
            if !preexisting {
                kernel.revoke_execution(room.id(), "component", &pin);
            }
            return Err(error.into());
        }
        let initialize = (|| -> std::result::Result<(), HostError> {
            checkpoint(room, &self.selection)
                .map_err(|_| HostError::new(ErrorCode::RestoreFailed, "operator.checkpoint"))?;
            room.prepare("component", "operator.host", correlation)?;
            room.start("component", "operator.host", correlation)?;
            let destination = Endpoint::new("component", &self.selection.input_port);
            let adapter =
                room.authorize_ui("operator.host", BTreeSet::from([destination.clone()]))?;
            let accepted = room.submit_ui(
                &adapter,
                correlation,
                destination,
                self.preview.initial_inputs.clone(),
                correlation,
            );
            if let Err(error) = accepted {
                room.revoke_ui(&adapter);
                return Err(error);
            }
            let errors = room.run_until_idle();
            room.revoke_ui(&adapter);
            if let Some(error) = errors.into_iter().next() {
                return Err(error);
            }
            if self.selection.output_ports.is_empty()
                || self.selection.output_ports.iter().any(|port| {
                    room.output("component", port).is_none_or(|output| {
                        !output.valid
                            || output.value.is_none()
                            || output.revision != room.revision()
                    })
                })
            {
                return Err(HostError::new(ErrorCode::StateConflict, "operator.commit"));
            }
            Ok(())
        })();
        Ok(ApplyOutcome {
            topology_committed: true,
            initialization_succeeded: initialize.is_ok(),
            initialization_error: initialize.err(),
        })
    }
}
