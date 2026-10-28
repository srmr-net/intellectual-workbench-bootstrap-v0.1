# Intellectual Workbench — ADR-006 Validation Kit v0.1

**Status:** PROPOSED. No implementation, independent review, live-provider run or hosted CI evidence is asserted by this document package.

This kit extends accepted ADR-005 for **opt-in real LLM suggestion generation only**. Accepted WIT v0.1, canonical `planVersion: "0.1"`, operator suggestion format, permissions, registry, semantic UI and persisted rooms remain normative. The supplied JSON Schemas are **supplemental configuration and redacted-observation fixtures**; they do not redefine the existing operator contracts.

## Files
- `ADR-006.md` — proposed normative architectural decision and limits.
- `TEST_MATRIX.md` — LLM-001–024 deterministic CI gates and LIVE-001–003 separate live gates.
- `VALIDATION_TASK.md` — implementation, review and evidence steps.
- `REVIEW_CHECKLISTS.md` — two independent reviewer scopes.
- `CODEX_PROMPT.md` — agent-ready development assignment.
- `schemas/` — provider profile / redacted observation JSON Schemas.
- `fixtures/` — no-secret profiles and observation.
- `evidence/RESULTS.template.md` — reproducible acceptance ledger.

## Repository placement
Place the ADR under `docs/adr/ADR-006.md` and other kit files under `docs/implementation/adr-006/` (adjust imports to the existing repository conventions). Do not overwrite previously accepted schemas or the deterministic fixture provider.

## Acceptance
1. All LLM-001–024 tests pass locally and in a real hosted Linux run, with archived, verified artifacts.
2. Both independent reviews approve documented results.
3. A separate opt-in real provider LIVE-001–003 smoke is recorded and passes.
4. Existing acceptance suites remain passing and accepted contracts remain unchanged.
5. Only then mark Accepted, explicitly indicating provider(s)/platform actually validated.

No credentials are included. Production model availability, API parameters and prices must be checked against current provider documentation when implementation occurs.
