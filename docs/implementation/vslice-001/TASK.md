# VSLICE-001 — Headless mortgage integration

- **Owner:** Codex; independent numerical/schema and security/integration reviewers.
- **Base:** merged PR #5, master ae5a0090ad46afa967b5b925b8b8287c1e295a17;
  master hosted run 40247542078 passed; registry implementation f275c21 accepted.
- **Goal:** real compiled mortgage guest → actual local registry → exact handle →
  kernel lifecycle → native ingress/bus → WIT invocation → atomic room projection.
- **Dependencies:** accepted ADR-001/002/003, this kit's SPECIFICATION/TEST_MATRIX;
  existing package schema and ontology. ADR-008 remains open for shared numeric
  conventions; this slice defines only its own bounded domain policy.
- **Owned paths:** examples/mortgage-headless/, new guest workspace members,
  crates/wb-kernel headless adapter/CLI, wb-testkit VSLICE tests, build/evidence tools,
  CI and task/evidence/backlog/status docs. Kit imported under this directory;
  vectors retain fixtures/mortgage-vectors.json. Original root README preserved.
- **Forbidden:** accepted WIT, manifest schema, ontology, accepted ABI/IPC/REG test
  sources, lifecycle/bus/registry semantics; no Qt/Operator/WASI/grants/remote APIs.
- **Acceptance:** bash examples/mortgage-headless/validate.sh; locked workspace
  tests/Clippy; rustfmt; Python contracts/harness; real REG/IPC/ABI and VSLICE-001..020;
  clean hosted Ubuntu CI, downloaded verified artifact and two independent reviews.
- **Failure cases:** schema/domain errors, output batches, tamper/imports, no grant,
  exact-pin failures, cancelled/stale tickets, queue pressure, room isolation,
  interrupted install, native restore with fresh approvals.
- **Status:** Accepted — 2026-10-26; all mandatory gates and verified hosted evidence pass. See evidence/RESULTS.md.
