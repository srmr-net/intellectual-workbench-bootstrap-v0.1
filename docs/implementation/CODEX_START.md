# Starting with Codex

1. Extract ZIP and initialise a Git repository (`git init && git add . && git commit -m "bootstrap v0.1"`).
2. Install Rust stable and Python dependencies, run checks in `README.md`.
3. Open the directory in Codex or use the Codex CLI. Its repository guidance is `AGENTS.md`.
4. First task prompt:

> Read `AGENTS.md`, `docs/CONTRACT_STATUS.md`, `references/architecture_interface_v0.1.md` and `docs/implementation/BACKLOG.md`. Implement **SPEC-001 only** in an isolated branch. Produce JSON Schema 2020-12 for composition plans, abstract UI, and room snapshots, with positive/negative fixtures and Python validation tests. Treat `schemas/component.schema.json` as read-only. Do not invent or silently approve pending ADR decisions. Run all available tests; report missing tools or blockers. Do not weaken test expectations. Summarise changed files and contract assumptions in your PR.

5. Review the PR yourself, especially any modification to shared contracts. Then ask Codex for SPEC-002.
6. Begin the WASM host only after completing and approving ADR-001 with pinned versions and an executable spike.
