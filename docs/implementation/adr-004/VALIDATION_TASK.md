# ADR-004 implementation and validation task

## Prerequisites

Start from a clean branch containing merged accepted ADR-001, ADR-002, ADR-003 and VSLICE-001 changes. Read `AGENTS.md`, actual current repository DTOs/interfaces and evidence; avoid assuming the original bootstrap scaffold is current. If PR #6 has not merged, obtain a merge/rebase strategy first. Copy this kit to `docs/adr/ADR-004.md` and `docs/implementation/adr-004/` as appropriate; ensure there is no unresolved placeholder ADR-004 before replacing it.

## Work scope

1. Inspect actual mortgage manifest, RDF, input/output schemas, package identity and existing UI example. Document differences from the kit's illustrative UI; do not change accepted WIT or mortgage numerical semantics to align an example.
2. Implement canonical UI schema/strict parser, semantic/manifest reconciliation, and typed bindings. Add positive and malicious negative fixtures.
3. Implement a bounded, thread-safe Rust-to-Qt bridge and deterministic trusted renderer. Choose/record exact Qt and Rust bridge versions, platform setup, licences and headless test strategy.
4. Add bounded host-owned draft ingress and revision/generation-fenced committed projections through the *real* room state path.
5. Implement separate UI-bearing registry admission extension, ensuring the old headless admission contract remains valid; publish a new immutable UI-bearing mortgage fixture.
6. Implement UI-001–020 with non-ignored, non-mocked tests and collect logs. Obtain independent reviews and fresh CI evidence.

## Hard boundaries

- Do not modify accepted `wit/component.wit`, ABI/IPC/REG semantics or numerical domain policy.
- Do not inject QML/JS/HTML from package files; no dynamic `Qt.createQmlObject` over untrusted text.
- Do not add guest imports, WASI privileges, network stores or Operator inference.
- Do not bypass registry admission to load package files, hard-code a mortgage UI or update view from unvalidated guest output.
- No silent headless fallbacks when UI tests are impossible; keep ADR proposed and report blocker.

## Acceptance procedure

1. Produce test fixtures and an indexed UI-001–020 matrix with test command and source file path.
2. Run local Rust, Python contract, package admission and real Qt integration tests, with formatting and Clippy.
3. Run security/Qt review and independent persistence/revision review; resolve issues and rerun affected tests.
4. Run Linux GitHub Actions from commit SHA; archive CI URL/artifacts, OS/Qt/Rust/Wasmtime versions, checksums, results and review findings.
5. Mark ADR Accepted only with required hosted evidence, **no skipped mandatory tests**, and no regression of accepted contracts. If an architecture decision must change, stop and propose a separate ADR.
