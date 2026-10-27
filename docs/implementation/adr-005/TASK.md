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
- Pending acceptance: dedicated Operator validation wrapper, OP001–024 native and
  genuine Qt/process tests, final independent reviews and hosted evidence.
  No validate-operator.sh exists yet. Status PROPOSED until all gates pass.
- Current work is a draft foundation; see BLOCKERS.md, evidence/local/RESULTS.md
  and draft PR9. No product Operator UI is shipped.
