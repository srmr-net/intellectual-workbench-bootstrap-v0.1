# Codex implementation / validation assignment — VSLICE-002

## Preconditions

1. Confirm PR #7/ADR-004 and PR #6/VSLICE-001 are on the integration base or explicitly choose a branch containing their accepted implementations. Avoid silently developing against stale master.
2. Read `AGENTS.md`, relevant canonical schemas, accepted ADR-001–004 and VSLICE-001 evidence; read this kit.
3. Inventory real exported APIs and current desktop application. Record a short `IMPLEMENTATION_MAP.md` noting any differences between specifications and implementation. Do not recreate competing DTOs or modify accepted WIT.

## Work sequence

1. Validate launcher packaging/development command and basic three-column shell; no Operator required.
2. Wire generic registry component discovery/selection, exact pin, validated semantic metadata and renderer. Assert no mortgage-only UI branch.
3. Connect UI edit drafts to accepted room input/compute/commit/projection path. Add pending and error states; test actual WASM numerical cases.
4. Implement minimal **versioned durable room snapshot** behind a Rust storage interface. Prefer already established persistence abstraction; implement transactional atomic write/SQLite storage and strict loader with size/path/version validation, not a custom DB.
5. Restore selected and pinned rooms, exact artifact/UI pins and committed view state on cold process launch. Handle missing pin/corrupt snapshots without implicit repair.
6. Build repeatable offscreen Qt process integration suite with temporary local registry and temporary storage. Capture logs and process IDs; test teardown, room isolation and offline restart.
7. Implement VS2-001–024 individually; ensure existing regression suites remain green.
8. Request independent reviewers with separate execution context. Address all findings with retest evidence.
9. Run GitHub-hosted Linux CI; archive and verify evidence/artifact checksums. Only then update milestone status to Accepted.

## Implementation boundaries

**Do not:** change WIT, existing mortgage math/contracts, accepted ontology, registry pin semantics, immutable artifact bytes; add arbitrary QML/JS, new WASI permissions, Operator dependency, remote component registries or cross-platform deliverables. Do not weaken previous test suites to pass new CI. Do not silently paper over existing recovery bugs; report limitations.

**May:** add a narrowly scoped Linux launcher, typed host-owned room snapshot DTO/storage adapter, UI selection view and genuine integration tests. Any new snapshot persistence format is an **internal v0.1 contract** and must be versioned/documented with negative fixtures.

## Required evidence

Populate `evidence/RESULTS.md` from the template with exact commit, branch, pinned toolchain/Qt versions, commands, VS2 matrix, hosted workflow URL, artifact hashes, numerical fixtures, Qt instrumentation and separate process IDs, reviewer outcomes, unresolved limitations and explicit acceptance reasoning. Preserve original outputs and links to logs. A configured workflow or local green tests are not hosted success.
