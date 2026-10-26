# VSLICE-001 — Installable Mortgage Calculator (Headless)

**Status:** PROPOSED — product-level acceptance requires all mandatory VSLICE gates, independent reviews, successful hosted Linux CI and archived evidence.  
**Version:** 0.1 · 2026-10-26  
**Depends on:** accepted ADR-001, ADR-002 and ADR-003; Component Package and Manifest Specification v0.1 and ontology v0.1.

## Contents
- `SPECIFICATION.md` — normative scope, cross-subsystem contract, financial behaviour, security and completion criteria.
- `VALIDATION_TASK.md` — agent-friendly implementation sequence and merge/CI instructions.
- `TEST_MATRIX.md` — VSLICE-001–020 acceptance cases, including positive and negative assertions.
- `fixtures/mortgage-vectors.json` — reference financial inputs, expected results, and invalid cases. These are reference test data, **not an installable WASM package**.
- `review/INDEPENDENT_REVIEW.md` — reviews and required evidence.
- `evidence/VSLICE-001-results-template.md` — evidence report to fill from real runs, no fabricated passes.
- `CODEX_PROMPT.md` — ready-to-copy agent task.

## Repository integration
1. Ensure accepted ADR-003 PR #5 is merged (or base the branch on its exact validated commit); never copy an unverified implementation from this kit.
2. Copy `SPECIFICATION.md`, `TEST_MATRIX.md`, `VALIDATION_TASK.md` and `CODEX_PROMPT.md` to `docs/implementation/vslice-001/` (or equivalent), and review/evidence templates under `docs/implementation/vslice-001/`.
3. Keep `wit/component.wit`, `component.schema.json`, ontology and accepted ABI/IPC/REG tests unchanged.
4. Ask Codex to implement in a feature branch, attach evidence, and use hosted GitHub CI before changing status.

## Acceptance definition
A *real* compiled mortgage WASM component must be admitted to the local registry from a v0.1 directory package; then resolved by `(id,version,digest)`, instantiated through the accepted host and lifecycle, invoked by the room controller/bus, and produce schema-checked numerical results. Negative gates must not leave installations, route invalid messages, or corrupt room state. No GUI, Operator or new permissions are needed.
