# OP-001 — Bounded local Operator

- Owner: primary Codex; distinct agent-security and runtime-integration reviewers.
- Base: cd39f8fbd76648bc384eabc92c49f01c205ab872, merged accepted PR8/VS2.
- Goal: read-only semantic catalog, replaceable async provider/offline fixture,
  strict powerless suggestions, canonical preview, explicit single-use approval,
  revision-fenced kernel apply, typed initialization, minimal real Qt intent UI.
- Owned: wb-operator new crate, narrow new kernel preview module +module declaration,
  new desktop Operator entry/panel/worker, supplemental schemas/fixtures, wrappers,
  CI/API/evidence docs. Old accepted source/test/collector hashes remain intact.
- Non-goals: new ABI/imports/permissions, remote registries/providers, generated
  QML, multi-component plans, automatic apply, changes to accepted snapshots.
- Available checks: cargo test --locked --workspace; cargo fmt --all -- --check;
  cargo clippy --locked --workspace --all-targets -- -D warnings; Python contract
  checks; bash apps/desktop-qt/validate-vslice-002.sh for accepted Qt/process gates.
- Implementation frozen for independent review at
  188cbba2c69e72f6800b7239b9068fe99d02c34f: bounded controller, real worker handoff,
  explicit native Qt consent and adversarial/process tests are implemented.
- Final command: bash apps/desktop-qt/validate-operator.sh. Pending acceptance:
  source-bound independent sign-offs, complete local wrapper and successful hosted
  run/download verification. Status PROPOSED until every gate passes.
- See CONFORMANCE_MAP.md, IMPLEMENTATION_MAP.md and evidence. PR9 remains draft
  while mandatory acceptance evidence is incomplete.
