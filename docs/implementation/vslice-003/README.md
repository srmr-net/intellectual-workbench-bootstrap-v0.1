# VSLICE-003 — Operator-Driven Component Discovery and Assembly

**Version 0.1 · Status PROPOSED · 2026-10-27**

This is the product-level acceptance kit for the bounded **Linux offline Operator** implementation from accepted ADR-005. It reuses admitted immutable components, trusted canonical `planVersion: "0.1"` planning, proposal-bound approval, accepted Qt renderer, ingress, WASM, and room persistence. **No new ABI**.

## Install into the repository

1. Ensure accepted ADR-005 and VSLICE-002 implementations are on the branch; do not overwrite newer tracked files.
2. Copy these files into `docs/implementation/vslice-003/`.
3. Follow `VALIDATION_TASK.md` and `CODEX_PROMPT.md` on a dedicated branch.
4. Produce named executable VS3 tests, reviews, and hosted GitHub CI evidence in `evidence/RESULTS.md`.
5. Keep status **PROPOSED** until every mandatory VS3 gate passes.

The included fixture is a **scenario description**, not a substitute for tests against the actual installed component. Test names, commands and implementation paths must be filled in from the checked-out repository; don't claim they have run.

## Source alignment

- Accepted ADR-001: WIT compute ABI and Wasmtime.
- Accepted ADR-002: lifecycle, reactive bus, revision/generation fences.
- Accepted ADR-003: immutable local package admission and exact pin resolution.
- Accepted ADR-004: trusted semantic Qt rendering and UI binding.
- Accepted ADR-005: bounded Operator provider/discovery/suggestion/approval policy.
- Accepted VSLICE-001 and VSLICE-002: mortgage computation and durable desktop canvas.

## Implementation and current validation

The separate [desktop entry](../../../apps/desktop-qt/vslice-003/README.md) reuses
accepted libraries/renderer contracts and supports a fresh empty room beside existing
canvases. See [boundaries](IMPLEMENTATION_MAP.md) and [executable test map](CONFORMANCE_MAP.md).
All gates remain Proposed until independent reviews and full local/hosted archive
verification finish. Optional external-model trial: NOT RUN (optional; no adapter).
