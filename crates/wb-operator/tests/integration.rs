use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use wb_contracts::ipc::UiReference;
use wb_kernel::Kernel;
use wb_operator::{catalog::Catalog, proposal::*, provider::*, suggestion::Suggestion};
use wb_registry::{Policy, Registry};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn setup() -> (tempfile::TempDir, Registry, Kernel) {
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
    let kernel = Kernel::new(
        wb_wasm_host::Limits {
            fuel: 20_000_000,
            ..Default::default()
        },
        8,
        8,
    )
    .unwrap();
    (tmp, registry, kernel)
}
const INTENT: &str = "I need a mortgage calculator for a £250,000 loan over 25 years at 5%.";
fn suggestion(catalog: &Catalog) -> Suggestion {
    Suggestion::parse(
        &await_provider(
            &OfflineFixture,
            ProviderRequest {
                request_version: "0.1".into(),
                request_id: "r1".into(),
                utterance: INTENT.into(),
                candidates: catalog.candidates(),
            },
            Cancellation::default(),
            Duration::from_secs(1),
        )
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn real_admission_preview_approval_ingress_and_replay() {
    let (_tmp, registry, kernel) = setup();
    let catalog = Catalog::discover(&registry).unwrap();
    assert_eq!(catalog.candidates().len(), 1);
    let mut room = kernel
        .create_room(
            "operator-room",
            "Operator",
            UiReference {
                definition_ref: "host.operator/room/ui.json".into(),
            },
            32,
        )
        .unwrap();
    let mut proposal = Proposal::prepare(
        &kernel,
        &room,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::from_secs(60),
    )
    .unwrap();
    assert_eq!(room.revision(), 0);
    assert_eq!(kernel.reserved_instances(), 0);
    let digest = proposal.preview().proposal_digest.clone();
    let context = || ApplyContext {
        kernel: &kernel,
        registry: &registry,
        catalog: &catalog,
        correlation: "op-real",
    };
    let mut checkpointed = false;
    let result = proposal
        .approve(&digest, &mut room, context(), |r, _| {
            assert_eq!(r.revision(), 1);
            assert!(r.snapshot().state.is_empty());
            checkpointed = true;
            Ok(())
        })
        .unwrap();
    assert!(
        checkpointed && result.topology_committed && result.initialization_succeeded,
        "{result:?}"
    );
    assert_eq!(
        room.output("component", "payment")
            .unwrap()
            .value
            .as_ref()
            .unwrap()["monthly_payment"],
        "1461.48"
    );
    let before = room.snapshot();
    assert!(proposal
        .approve(&digest, &mut room, context(), |_, _| Ok(()))
        .is_err());
    assert_eq!(room.snapshot(), before);
}
#[test]
fn checkpoint_failure_is_honest_partial_completion() {
    let (_tmp, registry, kernel) = setup();
    let catalog = Catalog::discover(&registry).unwrap();
    let mut room = kernel
        .create_room(
            "partial-room",
            "Partial",
            UiReference {
                definition_ref: "host.operator/room/ui.json".into(),
            },
            32,
        )
        .unwrap();
    let mut proposal = Proposal::prepare(
        &kernel,
        &room,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::from_secs(60),
    )
    .unwrap();
    let digest = proposal.preview().proposal_digest.clone();
    let result = proposal
        .approve(
            &digest,
            &mut room,
            ApplyContext {
                kernel: &kernel,
                registry: &registry,
                catalog: &catalog,
                correlation: "op-partial",
            },
            |_, _| Err(ProposalError::Persistence),
        )
        .unwrap();
    assert!(result.topology_committed && !result.initialization_succeeded);
    assert_eq!(room.revision(), 1);
    assert!(room.snapshot().state.is_empty());
    assert!(room.output("component", "payment").is_none());
}
#[test]
fn fresh_preview_identity_stale_revision_and_expiry() {
    let (_tmp, registry, kernel) = setup();
    let catalog = Catalog::discover(&registry).unwrap();
    let mut room = kernel
        .create_room(
            "fenced-room",
            "Fenced",
            UiReference {
                definition_ref: "host.operator/room/ui.json".into(),
            },
            32,
        )
        .unwrap();
    let mut first = Proposal::prepare(
        &kernel,
        &room,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::from_secs(60),
    )
    .unwrap();
    let mut second = Proposal::prepare(
        &kernel,
        &room,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::from_secs(60),
    )
    .unwrap();
    let old_digest = first.preview().proposal_digest.clone();
    assert_ne!(old_digest, second.preview().proposal_digest);
    let context = || ApplyContext {
        kernel: &kernel,
        registry: &registry,
        catalog: &catalog,
        correlation: "op-fence",
    };
    assert!(second
        .approve(&old_digest, &mut room, context(), |_, _| Ok(()))
        .is_err());
    let mut expired = Proposal::prepare(
        &kernel,
        &room,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::ZERO,
    )
    .unwrap();
    let expired_digest = expired.preview().proposal_digest.clone();
    assert!(matches!(
        expired.approve(&expired_digest, &mut room, context(), |_, _| Ok(())),
        Err(ProposalError::Expired)
    ));
    assert_eq!(room.revision(), 0);
    // A different explicitly approved proposal changes the revision.
    let mut winner = Proposal::prepare(
        &kernel,
        &room,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::from_secs(60),
    )
    .unwrap();
    let digest = winner.preview().proposal_digest.clone();
    winner
        .approve(&digest, &mut room, context(), |_, _| {
            Err(ProposalError::Persistence)
        })
        .unwrap();
    let before = room.snapshot();
    assert!(first
        .approve(&old_digest, &mut room, context(), |_, _| Ok(()))
        .is_err());
    assert_eq!(room.snapshot(), before);
}
#[test]
fn failed_apply_removes_only_new_execution_consent() {
    let (_tmp, registry, _kernel) = setup();
    let kernel = Kernel::new(wb_wasm_host::Limits::default(), 1, 1).unwrap();
    let catalog = Catalog::discover(&registry).unwrap();
    let make_room = |id| {
        kernel
            .create_room(
                id,
                "Quota",
                UiReference {
                    definition_ref: "host.operator/room/ui.json".into(),
                },
                32,
            )
            .unwrap()
    };
    let mut a = make_room("quota-a");
    let mut b = make_room("quota-b");
    let mut pa = Proposal::prepare(
        &kernel,
        &a,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::from_secs(60),
    )
    .unwrap();
    let mut pb = Proposal::prepare(
        &kernel,
        &b,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::from_secs(60),
    )
    .unwrap();
    let pin = pb.preview().plan.instances[0].component.clone();
    let context = || ApplyContext {
        kernel: &kernel,
        registry: &registry,
        catalog: &catalog,
        correlation: "op-quota",
    };
    let da = pa.preview().proposal_digest.clone();
    let db = pb.preview().proposal_digest.clone();
    pa.approve(&da, &mut a, context(), |_, _| {
        Err(ProposalError::Persistence)
    })
    .unwrap();
    assert!(pb.approve(&db, &mut b, context(), |_, _| Ok(())).is_err());
    assert_eq!(b.revision(), 0);
    assert!(!kernel.operator_execution_approved(b.id(), "component", &pin));
    let mut pc = Proposal::prepare(
        &kernel,
        &b,
        &registry,
        &catalog,
        INTENT,
        suggestion(&catalog),
        Duration::from_secs(60),
    )
    .unwrap();
    kernel.approve_execution(b.id(), "component", &pin).unwrap();
    let dc = pc.preview().proposal_digest.clone();
    assert!(pc.approve(&dc, &mut b, context(), |_, _| Ok(())).is_err());
    assert!(kernel.operator_execution_approved(b.id(), "component", &pin));
}
