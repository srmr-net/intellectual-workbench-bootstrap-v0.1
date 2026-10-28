# VSLICE-004 — Live Operator: End-to-End User Experience

**Version:** 0.1 | **Status:** PROPOSED | **Profile:** Linux desktop / OpenRouter live provider

This is a product acceptance kit, **not** an accepted architectural decision. It depends on accepted ADR-001–006 and VSLICE-001–003. Do not replace or relax the existing runtime, package, WIT, bus, UI, Operator, approval, and room contracts.

## Contents
- `SPECIFICATION.md`: normative user journey, boundaries and failure semantics.
- `TEST_MATRIX.md`: VS4-001–VS4-030 mandatory gates, divided into credential-free hosted and actual live acceptance.
- `VALIDATION_TASK.md`: implementation sequence and evidence collection.
- `REVIEW_CHECKLISTS.md`: independent privacy/security and desktop/UX reviews.
- `CODEX_PROMPT.md`: Codex handoff.
- `fixtures/scenarios.json`: scenarios and expected outcomes (not fabricated test output).
- `evidence/RESULTS.template.md`: evidence ledger and acceptance checklist.

## Repository placement
Copy this directory to `docs/implementation/vslice-004/`, preserving any repository-local changes. Update references to actual source paths after inspecting the checkout. Avoid overwriting accepted ADR evidence.

## Acceptance
1. Mandatory **VS4-001–026** deterministic local and hosted Linux gates pass without live secrets.
2. **VS4-027–028** two independent reviews approve with tracked findings resolved.
3. **VS4-029** hosted Linux CI and artifact verification pass after final source changes.
4. **VS4-030** separately supervised **real OpenRouter desktop** walkthrough passes with sanitized evidence and declared model/version and costs (if available).
5. Accepted regression suites stay green; changes to established contracts require separate ADR, not quiet modifications.

Passing mocked CI alone does not qualify as live-profile acceptance. A real-provider run alone does not qualify without the deterministic gates.
