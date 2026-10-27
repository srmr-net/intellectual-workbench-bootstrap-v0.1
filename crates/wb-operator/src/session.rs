//! Host-owned Operator authority. Kernel and mutable rooms never escape this
//! controller; all policy changes have an observable monotonic revision.
use crate::{
    catalog::{Catalog, Selection},
    proposal::{ApplyContext, ApplyOutcome, Preview, Proposal, ProposalError},
    provider::{await_provider, Cancellation, OperatorProvider, ProviderRequest},
    suggestion::{OperatorError, Suggestion},
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
    time::Duration,
};
use wb_contracts::{
    ipc::{HostError, RoomSnapshot, UiReference},
    ComponentRef,
};
use wb_kernel::{room::Room, Kernel};
use wb_registry::Registry;

#[derive(Debug)]
pub enum SessionError {
    Operator(OperatorError),
    Proposal(ProposalError),
    Host(HostError),
    UnknownRoom,
    StateConflict,
    PolicyDenied,
    NoProposal,
}
impl SessionError {
    pub fn native_code(&self) -> Option<String> {
        match self {
            Self::Host(e) | Self::Proposal(ProposalError::Host(e)) => serde_json::to_value(e.code)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned)),
            Self::Operator(OperatorError::PinUnavailable(code))
            | Self::Proposal(ProposalError::Operator(OperatorError::PinUnavailable(code))) => {
                Some(code.clone())
            }
            _ => None,
        }
    }
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownRoom => "UNSUPPORTED_REQUEST",
            Self::StateConflict => "STATE_CONFLICT",
            Self::PolicyDenied => "APPROVAL_REJECTED",
            Self::NoProposal => "APPROVAL_REQUIRED",
            Self::Operator(e) => e.code(),
            Self::Proposal(ProposalError::Host(e)) | Self::Host(e)
                if e.code == wb_contracts::ipc::ErrorCode::StateConflict =>
            {
                "STATE_CONFLICT"
            }
            Self::Proposal(ProposalError::Operator(e)) => e.code(),
            Self::Proposal(ProposalError::ApprovalRejected) => "APPROVAL_REJECTED",
            Self::Proposal(ProposalError::Expired) => "APPROVAL_EXPIRED",
            Self::Proposal(_) => "PLAN_REJECTED",
            Self::Host(_) => "PLAN_REJECTED",
        }
    }
}
impl From<OperatorError> for SessionError {
    fn from(e: OperatorError) -> Self {
        Self::Operator(e)
    }
}
impl From<ProposalError> for SessionError {
    fn from(e: ProposalError) -> Self {
        Self::Proposal(e)
    }
}
impl From<HostError> for SessionError {
    fn from(e: HostError) -> Self {
        Self::Host(e)
    }
}
type Result<T> = std::result::Result<T, SessionError>;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    pub status: &'static str,
    pub room_id: String,
    pub revision: u64,
    pub provider_kind: &'static str,
    pub proposal_digest: Option<String>,
    pub pin: Option<ComponentRef>,
    pub correlation: String,
    pub error_code: Option<String>,
    pub native_code: Option<String>,
    pub topology_committed: Option<bool>,
    pub initialization_succeeded: Option<bool>,
}
struct Prepared {
    epoch: u64,
    cancel: Cancellation,
    catalog: Catalog,
    proposal: Proposal,
}
struct Entry {
    room: Room,
    epoch: u64,
    denied: bool,
    prepared: Option<Prepared>,
    provider_calls: u64,
}
struct Inner {
    kernel: Kernel,
    rooms: BTreeMap<String, Entry>,
    audit: VecDeque<Audit>,
    request_counter: u64,
}
type Active = Arc<Mutex<BTreeMap<String, Cancellation>>>;
struct Lease {
    active: Active,
    room: String,
    cancel: Cancellation,
}
impl Drop for Lease {
    fn drop(&mut self) {
        let mut active = self.active.lock().unwrap();
        if active
            .get(&self.room)
            .is_some_and(|c| c.same_request(&self.cancel))
        {
            active.remove(&self.room);
        }
    }
}
/// Host-only service, not a provider tool. No room/kernel authority is serialized.
pub struct Session {
    registry: Registry,
    provider: Arc<dyn OperatorProvider>,
    inner: Mutex<Inner>,
    active: Active,
}
impl Session {
    pub fn new(
        registry: Registry,
        provider: Arc<dyn OperatorProvider>,
        limits: wb_wasm_host::Limits,
    ) -> Result<Self> {
        Ok(Self {
            registry,
            provider,
            inner: Mutex::new(Inner {
                kernel: Kernel::new(limits, 32, 32)?,
                rooms: BTreeMap::new(),
                audit: VecDeque::new(),
                request_counter: 0,
            }),
            active: Arc::default(),
        })
    }
    pub fn create_room(&self, id: &str, label: &str) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if inner.rooms.contains_key(id) || inner.rooms.len() >= 32 {
            return Err(OperatorError::QueueFull.into());
        }
        let room = inner.kernel.create_room(
            id,
            label,
            UiReference {
                definition_ref: "host.operator/room/ui.json".into(),
            },
            32,
        )?;
        inner.rooms.insert(
            id.into(),
            Entry {
                room,
                epoch: 0,
                denied: false,
                prepared: None,
                provider_calls: 0,
            },
        );
        Ok(())
    }
    pub fn snapshot(&self, id: &str) -> Result<RoomSnapshot> {
        self.inner
            .lock()
            .unwrap()
            .rooms
            .get(id)
            .map(|e| e.room.snapshot())
            .ok_or(SessionError::UnknownRoom)
    }
    pub fn provider_calls(&self, id: &str) -> Result<u64> {
        self.inner
            .lock()
            .unwrap()
            .rooms
            .get(id)
            .map(|e| e.provider_calls)
            .ok_or(SessionError::UnknownRoom)
    }
    pub fn execution_audit(&self, id: &str) -> Result<Vec<wb_contracts::ipc::AuditEvent>> {
        self.inner
            .lock()
            .unwrap()
            .rooms
            .get(id)
            .map(|e| e.room.audit().to_vec())
            .ok_or(SessionError::UnknownRoom)
    }
    pub fn audit(&self) -> Vec<Audit> {
        self.inner.lock().unwrap().audit.iter().cloned().collect()
    }
    fn record(
        &self,
        inner: &mut Inner,
        id: &str,
        status: &'static str,
        correlation: &str,
        preview: Option<&Preview>,
    ) {
        let revision = inner.rooms.get(id).map_or(0, |e| e.room.revision());
        if inner.audit.len() == 128 {
            inner.audit.pop_front();
        }
        inner.audit.push_back(Audit {
            status,
            room_id: if inner.rooms.contains_key(id) {
                id.into()
            } else {
                "unknown".into()
            },
            revision,
            provider_kind: self.provider.kind(),
            proposal_digest: preview.map(|p| p.proposal_digest.clone()),
            pin: preview.and_then(|p| p.plan.instances.first().map(|i| i.component.clone())),
            correlation: correlation.into(),
            error_code: None,
            native_code: None,
            topology_committed: None,
            initialization_succeeded: None,
        });
    }
    pub fn cancel(&self, id: &str) -> bool {
        let cancel = self.active.lock().unwrap().get(id).cloned();
        if let Some(c) = &cancel {
            c.cancel();
        }
        let mut inner = self.inner.lock().unwrap();
        let pending = inner
            .rooms
            .get_mut(id)
            .is_some_and(|e| e.prepared.take().is_some());
        if cancel.is_some() || pending {
            self.record(&mut inner, id, "proposal.cancelled", "host.cancel", None);
        }
        cancel.is_some() || pending
    }
    /// Revocation is meaningful even before execution consent exists. The epoch
    /// belongs to this sole host controller; callers cannot bypass it via Kernel.
    pub fn revoke(&self, id: &str) -> Result<()> {
        if let Some(cancel) = self.active.lock().unwrap().get(id).cloned() {
            cancel.cancel();
        }
        let mut inner = self.inner.lock().unwrap();
        let Inner { kernel, rooms, .. } = &mut *inner;
        let e = rooms.get_mut(id).ok_or(SessionError::UnknownRoom)?;
        e.epoch = e.epoch.checked_add(1).ok_or(SessionError::StateConflict)?;
        e.denied = true;
        if let Some(prepared) = &e.prepared {
            kernel.revoke_execution(
                id,
                "component",
                &prepared.proposal.preview().plan.instances[0].component,
            );
        }
        for component in e.room.snapshot().components {
            kernel.revoke_execution(
                id,
                &component.instance_id,
                &ComponentRef {
                    id: component.id,
                    version: component.version,
                    sha256: component.sha256,
                },
            );
        }
        self.record(&mut inner, id, "policy.revoked", "host.policy", None);
        Ok(())
    }
    /// A trusted host policy action, never part of provider or intent RPC.
    pub fn allow(&self, id: &str) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        let e = inner.rooms.get_mut(id).ok_or(SessionError::UnknownRoom)?;
        e.epoch = e.epoch.checked_add(1).ok_or(SessionError::StateConflict)?;
        e.denied = false;
        self.record(&mut inner, id, "policy.allowed", "host.policy", None);
        Ok(())
    }
    fn correlation(&self) -> Result<String> {
        let mut inner = self.inner.lock().unwrap();
        inner.request_counter = inner
            .request_counter
            .checked_add(1)
            .ok_or(SessionError::StateConflict)?;
        Ok(format!("operator-{}", inner.request_counter))
    }
    pub fn prepare(&self, id: &str, utterance: &str, budget: Duration) -> Result<Preview> {
        self.prepare_with_cancel(id, utterance, budget, Cancellation::default())
    }
    pub fn prepare_with_cancel(
        &self,
        id: &str,
        utterance: &str,
        budget: Duration,
        cancel: Cancellation,
    ) -> Result<Preview> {
        let correlation = self.correlation()?;
        let result = self.prepare_inner(id, utterance, budget, &correlation, cancel);
        if let Err(error) = &result {
            self.record_error(id, "proposal.rejected", &correlation, error);
        }
        result
    }
    fn record_error(
        &self,
        id: &str,
        status: &'static str,
        correlation: &str,
        error: &SessionError,
    ) {
        let mut inner = self.inner.lock().unwrap();
        self.record(&mut inner, id, status, correlation, None);
        if let Some(event) = inner.audit.back_mut() {
            event.error_code = Some(error.code().into());
            event.native_code = error.native_code();
        }
    }
    fn prepare_inner(
        &self,
        id: &str,
        utterance: &str,
        budget: Duration,
        request_id: &str,
        cancel: Cancellation,
    ) -> Result<Preview> {
        let request_id = request_id.to_owned();
        if utterance.len() > 4096 {
            return Err(OperatorError::RequestTooLarge.into());
        }
        if cancel.is_cancelled() {
            return Err(OperatorError::Cancelled.into());
        }
        {
            let mut active = self.active.lock().unwrap();
            if active.contains_key(id) || active.len() >= 32 {
                return Err(OperatorError::QueueFull.into());
            }
            active.insert(id.into(), cancel.clone());
        }
        let _lease = Lease {
            active: self.active.clone(),
            room: id.into(),
            cancel: cancel.clone(),
        };
        let (revision, epoch) = {
            let mut inner = self.inner.lock().unwrap();
            let e = inner.rooms.get_mut(id).ok_or(SessionError::UnknownRoom)?;
            if e.denied {
                return Err(SessionError::PolicyDenied);
            }
            if !e.room.snapshot().components.is_empty() {
                return Err(SessionError::StateConflict);
            }
            e.prepared = None;
            (e.room.revision(), e.epoch)
        };
        let result = (|| {
            let catalog = Catalog::discover(&self.registry)?;
            let request = ProviderRequest {
                request_version: "0.1".into(),
                request_id: request_id.clone(),
                utterance: utterance.into(),
                candidates: catalog.candidates(),
            };
            {
                let mut inner = self.inner.lock().unwrap();
                let e = inner.rooms.get_mut(id).ok_or(SessionError::UnknownRoom)?;
                if cancel.is_cancelled() {
                    return Err(OperatorError::Cancelled.into());
                }
                if e.denied || e.epoch != epoch || e.room.revision() != revision {
                    return Err(SessionError::StateConflict);
                }
                e.provider_calls = e
                    .provider_calls
                    .checked_add(1)
                    .ok_or(SessionError::StateConflict)?;
            }
            let bytes = await_provider(self.provider.as_ref(), request, cancel.clone(), budget)?;
            let suggestion = Suggestion::parse(&bytes)?;
            let active = self.active.lock().unwrap();
            if !active.get(id).is_some_and(|c| c.same_request(&cancel)) {
                return Err(SessionError::StateConflict);
            }
            drop(active);
            let mut inner = self.inner.lock().unwrap();
            let Inner { kernel, rooms, .. } = &mut *inner;
            let e = rooms.get_mut(id).ok_or(SessionError::UnknownRoom)?;
            if cancel.is_cancelled() {
                return Err(OperatorError::Cancelled.into());
            }
            if e.room.revision() != revision || e.epoch != epoch || e.denied {
                return Err(SessionError::StateConflict);
            }
            let mut proposal = Proposal::prepare(
                kernel,
                &e.room,
                &self.registry,
                &catalog,
                utterance,
                suggestion,
                Duration::from_secs(120),
            )?;
            proposal.bind_policy_revision(epoch)?;
            let preview = proposal.preview().clone();
            if cancel.is_cancelled() {
                return Err(OperatorError::Cancelled.into());
            }
            e.prepared = Some(Prepared {
                epoch,
                cancel: cancel.clone(),
                catalog,
                proposal,
            });
            self.record(
                &mut inner,
                id,
                "proposal.prepared",
                &request_id,
                Some(&preview),
            );
            Ok(preview)
        })();
        result
    }
    pub fn decline(&self, id: &str) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner
            .rooms
            .get_mut(id)
            .ok_or(SessionError::UnknownRoom)?
            .prepared
            .take()
            .ok_or(SessionError::NoProposal)?;
        self.record(&mut inner, id, "proposal.declined", "host.decline", None);
        Ok(())
    }
    /// Explicit native UI confirmation only. Preview comparison is single-use;
    /// wrong/expired/conflicting attempts also consume the stored proposal.
    /// Approval is the commit boundary; later cancellation/revocation serializes
    /// after it and does not claim interruption of an already approved apply.
    /// `checkpoint` must perform only bounded persistence I/O: NEVER reenter this
    /// Session or wait for another thread using it while these locks are held.
    pub fn approve<F>(&self, id: &str, digest: &str, checkpoint: F) -> Result<ApplyOutcome>
    where
        F: FnMut(&Room, &Selection) -> std::result::Result<(), ProposalError>,
    {
        let correlation = self.correlation()?;
        let result = self.approve_inner(id, digest, &correlation, checkpoint);
        if let Err(error) = &result {
            self.record_error(id, "approval.rejected", &correlation, error);
        }
        result
    }
    fn approve_inner<F>(
        &self,
        id: &str,
        digest: &str,
        correlation: &str,
        checkpoint: F,
    ) -> Result<ApplyOutcome>
    where
        F: FnMut(&Room, &Selection) -> std::result::Result<(), ProposalError>,
    {
        let active = self.active.lock().unwrap();
        if active.contains_key(id) {
            return Err(OperatorError::QueueFull.into());
        }
        let mut inner = self.inner.lock().unwrap();
        let Inner { kernel, rooms, .. } = &mut *inner;
        let e = rooms.get_mut(id).ok_or(SessionError::UnknownRoom)?;
        let mut prepared = e.prepared.take().ok_or(SessionError::NoProposal)?;
        if prepared.cancel.is_cancelled() {
            return Err(OperatorError::Cancelled.into());
        }
        if e.denied || e.epoch != prepared.epoch {
            return Err(SessionError::StateConflict);
        }
        let preview = prepared.proposal.preview().clone();
        let outcome = prepared.proposal.approve(
            digest,
            &mut e.room,
            ApplyContext {
                kernel,
                registry: &self.registry,
                catalog: &prepared.catalog,
                correlation,
            },
            checkpoint,
        );
        self.record(
            &mut inner,
            id,
            match &outcome {
                Ok(result) if result.initialization_succeeded => "initialization.committed",
                Ok(_) => "initialization.failed",
                Err(_) => "topology.failed",
            },
            correlation,
            Some(&preview),
        );
        if let (Some(event), Ok(result)) = (inner.audit.back_mut(), &outcome) {
            event.topology_committed = Some(result.topology_committed);
            event.initialization_succeeded = Some(result.initialization_succeeded);
            event.error_code = result
                .initialization_error
                .as_ref()
                .map(|_| "INGRESS_FAILED".into());
            event.native_code = result.initialization_error.as_ref().map(|e| {
                serde_json::to_value(e.code)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned()
            });
        }
        outcome.map_err(Into::into)
    }
    /// Validated committed projections only; never a provider-generated result.
    pub fn outputs(&self, id: &str) -> Result<BTreeMap<String, serde_json::Value>> {
        let inner = self.inner.lock().unwrap();
        let e = inner.rooms.get(id).ok_or(SessionError::UnknownRoom)?;
        let snapshot = e.room.snapshot();
        let Some(component) = snapshot.components.first() else {
            return Ok(BTreeMap::new());
        };
        let handle = self
            .registry
            .resolve_exact(&wb_registry::ReleasePin {
                id: component.id.clone(),
                version: component.version.clone(),
                sha256: component.sha256.clone(),
            })
            .map_err(|e| OperatorError::PinUnavailable(e.code))?;
        let (metadata, _) = self
            .registry
            .read_verified_bundle(&handle)
            .map_err(|e| OperatorError::PinUnavailable(e.code))?;
        let ports = metadata.manifest["ports"]["outputs"]
            .as_array()
            .ok_or(OperatorError::Unavailable)?;
        Ok(ports
            .iter()
            .filter_map(|p| {
                let port = p["name"].as_str()?;
                let output = e.room.output(&component.instance_id, port)?;
                if output.valid && output.revision == e.room.revision() {
                    Some((port.into(), output.value.clone()?))
                } else {
                    None
                }
            })
            .collect())
    }
}
