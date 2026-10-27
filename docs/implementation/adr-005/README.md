# Workbench ADR-005 validation kit v0.1

**Status:** Accepted for the bounded Linux offline profile. The bounded implementation, executable gate map and independent reviews are present; see [acceptance evidence](evidence/RESULTS.md).

## Contents

- `ADR-005.md`: proposed normative decision and security model.
- `TEST_MATRIX.md`: 24 mandatory gates, OP-001–OP-024.
- `VALIDATION_TASK.md`: Codex implementation/review/hosted CI procedure.
- `CODEX_PROMPT.md`: ready-to-paste assignment.
- `schemas/operator-suggestion.schema.json`: narrow, untrusted provider output shape.
- `schemas/operator-request.schema.json`: minimal host→provider request shape.
- `examples/operator-request.json`, `examples/operator-suggestion.json`: *illustrative* fixture records; keys are opaque, not real installed pins.
- `REVIEW_CHECKLISTS.md`: independent review requirements.
- `evidence/RESULTS.template.md`: explicit pass/fail and source evidence ledger.

## Repository integration

Put ADR in `docs/adr/ADR-005.md`, remaining files in `docs/implementation/adr-005/` (schemas may be copied to canonical contracts directory following review). **Do not overwrite an accepted composition-plan schema:** the schema files here describe the *agent boundary only*. Check actual accepted mortgage fields and pinned package identity before implementation. Do not infer success from illustrative JSON fixtures.

Run `bash apps/desktop-qt/validate-operator.sh` for the full locked validation chain. See [executable conformance](CONFORMANCE_MAP.md), [implementation boundaries](IMPLEMENTATION_MAP.md), and [acceptance evidence](evidence/RESULTS.md). Illustrative kit examples remain non-executable; the product tests build and admit real compiled packages.
