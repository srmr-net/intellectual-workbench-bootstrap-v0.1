# Codex Task — ADR-003 Implementation and Validation

## Goal
Implement `wb-registry` local directory admission and exact pinned resolution according to `docs/adr/ADR-003.md`, keeping accepted ADR-001 and ADR-002 unchanged. Use `AGENTS.md` and the existing contracts in the repository as authorities.

## Preparation
1. Inspect current checkout, CI and accepted ABI/IPC implementations. Do not assume this ZIP reflects the latest source.
2. Check `docs/adr/ADR-003.md`, `docs/specs` or equivalent for the canonical component package schema and semantic ontology. Do not rewrite the manifest/ontology to make fixtures pass.
3. Record the exact `component.schema.json` and ontology SHA-256 in the evidence report. Determine whether UI schema and SHACL runner are implemented. An unsupported feature must reject packages rather than silently skip validation.
4. Inspect current error DTOs and preserve ABI/IPC contracts. Set ADR status PROPOSED throughout development.

## Implementation stages

**Stage A — Contract and fixtures.** Create a real compiled pure WASM fixture and a matching v0.1 directory package; add strict JSON duplicate-key parse, manifest schema validation, local path/schema-ref checks, raw digest validation, RDF/SHACL exact reconciliation and signature-free assurance policy. Never use metadata-only mortgage example as a positive install fixture.

**Stage B — ABI and policy admission.** Use ADR-001 Wasmtime Component Model inspection without guest invocation, check the accepted `compute` type and forbidden imports, handle unsupported/unknown fields and nonempty permission requests conservatively.

**Stage C — Registry storage.** Implement safe staging of directory package with race-safe file opening and no links/nonregular files; immutable content store, commit journal/marker, atomic index visibility, startup recovery, idempotence, same-version conflict, exact pin resolution and read-time integrity verification. Implement concurrency and interruption injection tests.

**Stage D — Integration.** Adapt `wb-kernel` only as required to consume registry-issued exact pinned handles. Do not alter lifecycle or messaging semantics. Keep GUI/Qt out of scope.

**Stage E — Independent review, hosted verification.** Run REG-001–REG-020, existing ABI-001–010, IPC-001–015, all workspace checks and GitHub-hosted Linux CI. Archive complete logs/hashes and evidence. Obtain independent review of untrusted filesystem handling and persistence write ordering.

## Non-negotiable invariants
- Source package stays untrusted until every admission check passes.
- Registry discovery does not run component code.
- JSON manifest remains authoritative; RDF cannot expand executable permissions.
- No float/semver-range resolution; no silent upgrade; no remote fetching.
- No arbitrary QML/JS execution.
- No new WASI/Workbench guest imports or WIT modifications.
- No automatic trust elevation from a package's evidence assertions.
- Never mark ADR Accepted based on local-only tests.

## PR strategy
Recommend small reviewable PRs: (1) parser/fixtures/semantic reconciliation, (2) staged immutable install/recovery/pinning, (3) integration tests and evidence. Each PR retains formatting, Clippy and existing ABI/IPC suites. If the schema/SHACL/UI contracts have gaps, document blockers and propose a narrow follow-up contract task rather than guessing.

## Required outputs
- Implementation with tests for every `REG-001`–`REG-020` gate (map exact test identifiers to commands).
- `docs/adr/evidence/ADR-003-results.md` filled using the included template.
- Updated `ADR-003.md` status only after hosted success and artifact verification.
- Exact Git commit, PR and hosted GitHub Actions URL with uploaded logs.
- Known limitations and explicitly deferred work.

## Ready-to-paste Codex prompt
> Read AGENTS.md, accepted ADR-001 and ADR-002, docs/adr/ADR-003.md and the Component Package and Manifest Specification v0.1. Implement ADR-003 using incremental reviewable changes. Preserve accepted WIT and IPC contracts. Produce a compiled pure-WASM positive package and implement safe local admission, strict manifest/RDF/SHACL/ABI validation, immutable atomic installation, exact pins, conflict detection and crash recovery. Implement and run REG-001–REG-020 plus existing ABI/IPC regressions. Obtain independent review of filesystem and durability invariants. Report local results and obtain hosted Linux CI proof before changing status to Accepted. Do not invent support for signatures, remote registries, external dependencies, or unapproved UI/permissions contracts.
