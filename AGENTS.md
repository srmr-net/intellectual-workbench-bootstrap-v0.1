# Agent Development Instructions — v0.1

Read `README.md`, `docs/CONTRACT_STATUS.md`, the relevant upstream spec and your task file **before editing**. The target is a component-based intellectual workbench, **not** an LLM-code-generator wrapper.

## Non-negotiable invariants

1. Native Rust trusted kernel enforces runtime lifecycle, isolation boundary, permissions and storage adapters. WASM components never receive ambient filesystem/network/process privileges.
2. Manifest `component.json` is canonical for executable identity, artifacts, declared ports and permission requests. Turtle metadata can augment discovery but cannot grant permission or override manifest data.
3. Operator proposes topology; host validates it. Operator does not run reactive state changes or issue arbitrary QML.
4. UI metadata is declarative and validated; no execution of QML strings from component bundles.
5. State propagation is deterministic and revision aware. Never commit an older computation over newer state.
6. Pinned component versions/digests must be restored or explicitly marked missing; never auto-substitute.
7. Test failures may not be 'fixed' by silently weakening checks. No agent may amend its own acceptance criteria unilaterally.
8. Provisional ADRs are **not** approved decisions; flag their blockers rather than assume implementations.

## Workflow

- Select one task in `docs/implementation/BACKLOG.md`; record owner, paths, dependencies, acceptance commands in an issue/task file based on `docs/implementation/TASK_TEMPLATE.md`.
- Work in a dedicated branch/worktree; don't edit outside allocated paths without handoff.
- Update API README for each public crate change, tests/fixtures for every contract change, and an ADR for any changed trust boundary or public protocol.
- Commit atomic changes and submit a PR with: intent, changed contracts, verification commands/results, compatibility, risks, remaining work.
- Treat `references/` as read-only upstream material. Any conflicts between references must be surfaced in a new ADR and test case, not hidden by rewriting source docs.
- Run `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `python3 tools/check_contracts.py` where tools are available. Report skipped tools honestly.
- Independent review is required for the kernel, permission grants, Wasm execution, installer, and development-tool privileges.

## Work ownership

- `wb-contracts`: shared versioned DTOs; changing them is a cross-team coordination event.
- `wb-wasm-host`: Wasmtime/WIT adapter **only after ADR-001**.
- `wb-bus`: message envelopes and routing.
- `wb-capabilities`: policy/grant boundary (deny by default).
- `wb-semantic`: RDF parser, SHACL/compatibility later.
- `wb-registry`: verify before install, never use mutable versions for restore.
- `wb-reactive`: deterministic DAG and state revision semantics.
- `wb-storage`: durable room snapshots and recovery abstraction.
- `wb-kernel`: orchestration across boundaries.
- `wb-testkit`: conformance fixtures, test harness.
- `apps/desktop-qt`: render **validated** semantic UI only.

## Banned shortcuts

Do not embed secret credentials; do not provide dummy 'allow all' broker; do not execute untrusted QML; do not claim test pass when tool missing; do not substitute a generic arbitrary shell agent for sandboxed components; do not publish metadata-only mortgage example as installable.
