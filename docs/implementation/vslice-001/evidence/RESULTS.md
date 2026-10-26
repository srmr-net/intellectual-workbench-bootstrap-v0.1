# VSLICE-001 — validation evidence

**Status:** PROPOSED. Implementation and two independent reviews are complete;
clean full-workspace locked execution, actual hosted Linux run and downloaded
artifact verification remain acceptance gates. No acceptance is claimed here.

Base: merged ADR-003 PR #5 at `ae5a0090ad46afa967b5b925b8b8287c1e295a17`;
master hosted run40247542078 passed before this branch. Accepted ADR-001/002/003
statuses and source semantics are unchanged. See TASK.md/DESIGN.md and the
original supplied SPECIFICATION.md/TEST_MATRIX.md for requirements and mappings.

Reproduce using Rust1.99.1, cargo-component0.21.1, Wasmtime49.0.1,
wasm32-unknown-unknown/wasm32-wasip1 targets, and tools/requirements.txt:

```sh
bash examples/mortgage-headless/validate.sh
```

The wrapper builds real mortgage/adversary/observer components, stamps complete
strict local directory packages, verifies33 independent numerical oraclecases,
compares a clean same-machine rebuild, and retains all original ABI/IPC/REG
wrappers and locked workspace checks. It archives actual receipts/records,
canonical messages, dispatch contexts, projections/audit, numerical outputs,
calibration, all referenced package bytes, sources, tool versions and hashes in
`target/abi-evidence`. Collector requires both recorded independent approvals and
matching source hashes. It does not infer hosted acceptance from local success.

Development checks:23 VSLICE tests passed; final strengthened005/009 cases,
33-case financial/budget tests and key security tests independently retested.
Final clean source SHA, full totals, PR, hosted URL and archives will be added
from actual executions; current results are not substituted for those gates.

Two independent reviewers: `/root/review_vslice_finance` (numerical/schema) and
`/root/review_vslice_integration` (security/integration). Findings corrected:
strict terminal-newline schema rejection; explicit bounded20M product caller
fuel rather than changing accepted2M Host defaults; typed negative assertions
respecting early unreferenced-file rejection. Both approved final source hashes,
with concrete independently executed tests. Review records bind these hashes to
the implementation commit before hosted validation.

Unsupported/deferred: Qt/Operator/UI packages, native capabilities, remote
registries, external package resolution, signatures, shared ADR-008 conventions,
room database durability and physical power-cut proof. Required room UI reference
is inert; snapshots recompute derived values. Failure preserves prior output VALUE
while accepted diagnostics/lifecycle may change. No WIT, manifest/ontology fields,
registry scope, lifecycle/bus semantics or original acceptance tests are changed.

## Final local and hosted evidence

PENDING: exact clean source, executed full checks, artifact hash verification.
