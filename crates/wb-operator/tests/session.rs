use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    task::Poll,
    time::{Duration, Instant},
};
use wb_operator::{
    proposal::ProposalError,
    provider::*,
    session::{Session, SessionError},
};
use wb_registry::{Policy, Registry};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn setup(provider: Arc<dyn OperatorProvider>) -> (tempfile::TempDir, Arc<Session>) {
    let tmp = tempfile::tempdir().unwrap();
    let mut p = Policy::local(
        vec![root().join("target/ui-fixtures")],
        std::env::var_os("REGISTRY_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| root().join("target/vs2-venv/bin/python")),
        root().join("tools/registry_validate.py"),
        root().join("target/debug/wb-registry-inspect"),
    );
    p.semantic_ui_v01 = true;
    let registry = Registry::open(&tmp.path().join("registry"), p).unwrap();
    registry
        .admit_directory(&root().join("target/ui-fixtures/mortgage-ui"))
        .unwrap();
    (
        tmp,
        Arc::new(
            Session::new(
                registry,
                provider,
                wb_wasm_host::Limits {
                    fuel: 20_000_000,
                    ..Default::default()
                },
            )
            .unwrap(),
        ),
    )
}
const INTENT: &str = "I need a mortgage calculator for a £250,000 loan over 25 years at 5%.";
#[test]
fn private_session_policy_epoch_revoke_before_grant_and_real_commit() {
    let (_tmp, session) = setup(Arc::new(OfflineFixture));
    session.create_room("room-a", "A").unwrap();
    let old = session
        .prepare("room-a", INTENT, Duration::from_secs(30))
        .unwrap();
    assert_eq!(old.policy_revision, "0");
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/implementation/adr-005/schemas/operator-preview.schema.json"
    ))
    .unwrap();
    let composition: serde_json::Value =
        serde_json::from_str(include_str!("../../../schemas/composition.schema.json")).unwrap();
    let validator = jsonschema::options()
        .with_resource(
            composition["$id"].as_str().unwrap(),
            jsonschema::Resource::from_contents(composition.clone()).unwrap(),
        )
        .build(&schema)
        .unwrap();
    assert!(validator.is_valid(&serde_json::to_value(&old).unwrap()));
    session.revoke("room-a").unwrap();
    assert_eq!(
        session
            .approve("room-a", &old.proposal_digest, |_, _| Ok(()))
            .unwrap_err()
            .code(),
        "STATE_CONFLICT"
    );
    assert_eq!(session.snapshot("room-a").unwrap().revision, 0);
    assert!(matches!(
        session.prepare("room-a", INTENT, Duration::from_secs(30)),
        Err(SessionError::PolicyDenied)
    ));
    session.allow("room-a").unwrap();
    let preview = session
        .prepare("room-a", INTENT, Duration::from_secs(30))
        .unwrap();
    assert_eq!(preview.policy_revision, "2");
    assert_ne!(preview.proposal_digest, old.proposal_digest);
    let outcome = session
        .approve("room-a", &preview.proposal_digest, |_, _| Ok(()))
        .unwrap();
    assert!(outcome.initialization_succeeded);
    assert_eq!(
        session.outputs("room-a").unwrap()["payment"]["monthly_payment"],
        "1461.48"
    );
    assert!(session
        .approve("room-a", &preview.proposal_digest, |_, _| Ok(()))
        .is_err());
    assert_eq!(session.provider_calls("room-a").unwrap(), 2);
    let audit = serde_json::to_string(&session.audit()).unwrap();
    assert!(
        !audit.contains("250,000") && !audit.contains("principal") && !audit.contains("I need")
    );
    assert!(audit.contains("initialization.committed"));
}
struct Gated {
    calls: AtomicUsize,
    ready: AtomicBool,
}
impl OperatorProvider for Gated {
    fn kind(&self) -> &'static str {
        "gated-test"
    }
    fn suggest<'a>(&'a self, request: ProviderRequest, cancel: Cancellation) -> ProviderFuture<'a> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            std::future::poll_fn(|_| {
                if self.ready.load(Ordering::SeqCst) {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
            OfflineFixture.suggest(request, cancel).await
        })
    }
}
fn wait_calls(provider: &Gated, n: usize) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while provider.calls.load(Ordering::SeqCst) < n {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn pending_cancel_queue_release_policy_race_and_room_isolation() {
    let provider = Arc::new(Gated {
        calls: AtomicUsize::new(0),
        ready: AtomicBool::new(false),
    });
    let (_tmp, session) = setup(provider.clone());
    session.create_room("room-a", "A").unwrap();
    session.create_room("room-b", "B").unwrap();
    std::thread::scope(|scope| {
        let pending = scope.spawn(|| session.prepare("room-a", INTENT, Duration::from_secs(30)));
        wait_calls(&provider, 1);
        assert_eq!(
            session
                .prepare("room-a", INTENT, Duration::from_secs(1))
                .unwrap_err()
                .code(),
            "QUEUE_FULL"
        );
        session.revoke("room-a").unwrap();
        assert!(pending.join().unwrap().is_err());
    });
    assert_eq!(session.snapshot("room-a").unwrap().revision, 0);
    assert_eq!(session.provider_calls("room-b").unwrap(), 0);
    session.allow("room-a").unwrap();
    assert_eq!(
        session
            .prepare("room-a", INTENT, Duration::from_millis(5))
            .unwrap_err()
            .code(),
        "PROVIDER_TIMEOUT"
    );
    assert_eq!(session.snapshot("room-a").unwrap().revision, 0);
    provider.ready.store(true, Ordering::SeqCst);
    let preview = session
        .prepare("room-a", INTENT, Duration::from_secs(30))
        .unwrap();
    let other = session
        .prepare("room-b", INTENT, Duration::from_secs(30))
        .unwrap();
    assert!(session
        .approve("room-b", &preview.proposal_digest, |_, _| Ok(()))
        .is_err());
    assert_eq!(session.snapshot("room-b").unwrap().revision, 0);
    assert!(session
        .approve("room-b", &other.proposal_digest, |_, _| Ok(()))
        .is_err());
    assert!(session.cancel("room-a"));
    assert!(session
        .approve("room-a", &preview.proposal_digest, |_, _| Ok(()))
        .is_err());
    let preview = session
        .prepare("room-a", INTENT, Duration::from_secs(30))
        .unwrap();
    let outcome = session
        .approve("room-a", &preview.proposal_digest, |_, _| {
            Err(ProposalError::Persistence)
        })
        .unwrap();
    assert!(outcome.topology_committed && !outcome.initialization_succeeded);
    let audit = session.audit();
    let last = audit.last().unwrap();
    assert_eq!(last.status, "initialization.failed");
    assert_eq!(last.topology_committed, Some(true));
    assert!(last.error_code.is_some());
    assert!(session.outputs("room-a").unwrap().is_empty());
}

#[test]
fn committed_guest_then_real_final_write_failure_preserves_topology_checkpoint() {
    use std::os::unix::fs::PermissionsExt;
    use wb_contracts::ComponentRef;
    use wb_storage::vs2::{Checkpoint, FileStore, UiPin, Workspace};
    let (tmp, session) = setup(Arc::new(OfflineFixture));
    session.create_room("room-a", "Room").unwrap();
    let directory = tmp.path().join("workspace");
    let store = FileStore::open(&directory).unwrap();
    let mut workspace = Workspace::default();
    let preview = session
        .prepare("room-a", INTENT, Duration::from_secs(30))
        .unwrap();
    let result = session
        .approve("room-a", &preview.proposal_digest, |room, s| {
            let cp = Checkpoint {
                room_id: room.id().into(),
                label: room.snapshot().name.clone(),
                pinned: false,
                component: ComponentRef {
                    id: s.pin.id.clone(),
                    version: s.pin.version.clone(),
                    sha256: s.pin.sha256.clone(),
                },
                ui: UiPin {
                    path: s.ui_path.clone(),
                    sha256: s.ui_sha256.clone(),
                    schema_sha256: s.ui_schema_sha256.clone(),
                },
                room: room.snapshot(),
                outputs: Default::default(),
            };
            workspace.rooms = vec![cp];
            workspace.selected_room = Some("room-a".into());
            workspace = store
                .commit(workspace.revision, &workspace)
                .map_err(|_| ProposalError::Persistence)?;
            Ok(())
        })
        .unwrap();
    assert!(result.initialization_succeeded);
    let before = std::fs::read(directory.join("workspace.json")).unwrap();
    let mut full = workspace.clone();
    full.rooms[0].room = session.snapshot("room-a").unwrap();
    full.rooms[0].outputs = session.outputs("room-a").unwrap();
    assert_eq!(
        full.rooms[0].outputs["payment"]["monthly_payment"],
        "1461.48"
    );
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o500)).unwrap();
    let failure = store.commit(workspace.revision, &full);
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(failure.unwrap_err(), "SNAPSHOT_IO");
    assert_eq!(
        std::fs::read(directory.join("workspace.json")).unwrap(),
        before
    );
    let saved = store.load().unwrap().unwrap();
    assert!(saved.rooms[0].room.state.is_empty());
    assert!(saved.rooms[0].outputs.is_empty());
}
#[test]
fn op_022_concurrent_rooms_keep_intent_approval_and_cancellation_isolated() {
    let provider = Arc::new(Gated {
        calls: AtomicUsize::new(0),
        ready: AtomicBool::new(false),
    });
    let (_tmp, session) = setup(provider.clone());
    session.create_room("room-a", "A").unwrap();
    session.create_room("room-b", "B").unwrap();
    let preview = std::thread::scope(|scope| {
        let a = scope.spawn(|| session.prepare("room-a", INTENT, Duration::from_secs(30)));
        wait_calls(&provider, 1);
        let b = scope.spawn(|| {
            session.prepare(
                "room-b",
                "I need a mortgage calculator for a £300,000 loan over 25 years at 5%.",
                Duration::from_secs(30),
            )
        });
        wait_calls(&provider, 2);
        assert!(session.cancel("room-a"));
        assert!(a.join().unwrap().is_err());
        provider.ready.store(true, Ordering::SeqCst);
        b.join().unwrap().unwrap()
    });
    assert!(session
        .approve("room-a", &preview.proposal_digest, |_, _| Ok(()))
        .is_err());
    assert_eq!(session.snapshot("room-a").unwrap().revision, 0);
    let result = session
        .approve("room-b", &preview.proposal_digest, |_, _| Ok(()))
        .unwrap();
    assert!(result.initialization_succeeded);
    assert_eq!(
        session.outputs("room-b").unwrap()["payment"]["monthly_payment"],
        "1753.77"
    );
    assert_eq!(session.provider_calls("room-a").unwrap(), 1);
    assert_eq!(session.provider_calls("room-b").unwrap(), 1);
    let audit = serde_json::to_string(&session.audit()).unwrap();
    assert!(
        !audit.contains("300,000") && !audit.contains("250,000") && !audit.contains("principal")
    );
}
#[test]
fn op_012_scope_quota_creates_no_guest_or_execution_authority() {
    let (_tmp, session) = setup(Arc::new(OfflineFixture));
    for n in 0..32 {
        let id = format!("quota-{n}");
        session.create_room(&id, "Quota").unwrap();
        assert!(session.snapshot(&id).unwrap().components.is_empty());
        assert!(session.execution_audit(&id).unwrap().is_empty());
        assert_eq!(session.provider_calls(&id).unwrap(), 0);
    }
    assert_eq!(
        session
            .create_room("quota-extra", "Quota")
            .unwrap_err()
            .code(),
        "QUEUE_FULL"
    );
}
