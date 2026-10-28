# ADR-006 — Codex implementation and validation task

## Preparation
1. Start from latest merged main/master and read `AGENTS.md`, accepted ADR-001–005, VSLICE-001–003 and their evidence. Identify the actual `OperatorProvider` signature, suggestion contract and Qt request path; document their source paths.
2. Import `ADR-006.md`, `TEST_MATRIX.md`, accompanying fixtures/schemas and evidence template into docs. Keep the decision Proposed.
3. List all proposed source changes and demonstrate that accepted WIT, canonical plan, registry, permissions, UI and room schema contracts remain untouched.

## Implement in bounded PRs
1. Add provider profile, remote consent and strict policy validation behind an opt-in feature flag; retain offline provider as default.
2. Add a transport abstraction with deterministic HTTP mocks, response-size limits, abort/deadline and error taxonomy.
3. Implement one actual API adapter, preferably OpenAI Responses API strict structured output, using supported model settings validated against current docs. No tools and no dynamic base URLs.
4. Reuse existing ADR-005 untrusted suggestion parser and trusted preview/approval/apply path. Do not create an alternative model execution pipeline.
5. Add minimal Qt status/selection/consent/cancel and safe error presentation without hard-coded mortgage handling.
6. Implement named `LLM-001`–`LLM-024` gates, including attack fixtures and regression suites; require no network credentials for CI.

## Independent review
- **Review A — security/privacy:** network egress, secret storage/redaction, SSRF/redirects, consent, data minimisation, malicious catalog/model text, logging, budget and cancellation. Reviewer must examine actual source and execute representative tests.
- **Review B — provider/runtime:** API shape and failure classification, deterministic mock fidelity, accepted provider boundary, exact pin rechecks, approval, Qt threading, room fences, offline restoration and ABI/IPC/REG/UI regression preservation.
- Archive reviewer identity/role, scope, findings, fix commits and retest outcomes. Reviewer approval is not implied by a CI pass.

## Evidence
- Run `cargo test --locked --workspace`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, formatter and all Python/Qt/process tests configured by current repository. Use the existing CI workflow, do not substitute a weaker one.
- Save runner OS, toolchain/dependency versions, commit ID, full command list, gate counts, redacted test logs and artifact SHA-256 hashes.
- Confirm hosted GitHub Linux run successful and download/verify uploaded evidence independently.
- Run the separate opt-in live test using a private credential source. Never upload secret-bearing logs, prompts or credentials. Distinguish mocked PASS from LIVE PASS in the ledger.
- Accept ADR only after all 24 gates, reviews, hosted CI and live tests pass. If live account unavailable leave Proposed, explicitly noting deterministic implementation readiness.

## Change prohibitions
No changes to WIT v0.1, `planVersion: "0.1"`, accepted manifest/ontology/UI schema, core approval and permission semantics, no WASI import expansion, no remote registry, model-driven tools, arbitrary QML or multi-component planning.
