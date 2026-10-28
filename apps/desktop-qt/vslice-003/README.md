# Linux desktop Operator journey — VSLICE003

Milestone status and evidence: [VSLICE003](../../../docs/implementation/vslice-003/README.md).

Build/check: `bash apps/desktop-qt/validate-vslice-003.sh`. Prerequisites are the
accepted Qt6 profile, Rust1.99.1, cargo-component0.21.1 and tools/requirements.txt.

Launch after the accepted installer has admitted the real UI-bearing package:

```sh
target/vs3-build/workbench-vslice003 "$PWD/target/debug/wb-vslice003-worker" \
  /absolute/registry /absolute/workspace /absolute/python \
  "$PWD/tools/registry_validate.py" "$PWD/target/debug/wb-registry-inspect"
```

The center column accepts the explicit request, retains submitted text in memory,
shows bounded progress, exact canonical proposal/full pin/values/units/effects and
untrusted explanation as plain text. Check fresh consent, then Confirm, or Reject.
Only committed output reaches the generic semantic renderer. Pin and quit normally;
cold reopening displays the exact saved input/output without a provider or guest.
`WB_OPERATOR_PROVIDER=disabled` disables the provider; there is no remote adapter.

Each request proposes one fresh empty room beside existing saved rooms. It never
adds an instance to a populated room. Accepted Session/Kernel own native lifecycle,
consent and typed ingress; the product entry owns FileStore handoff. Preview captures
workspace revision/hash in addition to the accepted proposal fences. At confirmation
it reacquires the writer lock and rechecks the snapshot before mutation. Topology
checkpoint append preserves every existing room; initialization is a distinct step.
Only a schema-validated committed output replaces the new topology-only checkpoint.
Persistence acknowledgements and partial failure are reported separately. A failed
initialization offers direct form repair; it does not claim cross-stage rollback.

This is a separate opt-in executable/standalone Rust product manifest. Accepted
ADR005/desktop source, lockfile, tests, collectors, workflow and source-bound review
receipts remain unchanged. The new locked manifest references the same accepted
library crates; its formatting/build/test/Clippy are run explicitly by the wrapper.
New functionality is bounded by the existing snapshot limit of16 rooms and the
same queues/frame limits. No ABI/schema/ontology/guest permission change is involved.

Conformance uses actual Qt events, accessible widgets/Quick text, real registry,
WASM and cold OS processes. Optional validation actions use the same whitelisted
native harness as ADR005; they never execute package scripts or direct calculation.

The trusted renderer derives from the accepted generic QML with one view-lifetime
change: destroy controls while selection is pending, then recreate against the new
epoch. This prevents identical metadata in two rooms retaining the first editor's
owner epoch. Old delayed callbacks retain old ownership and fail closed. No
component-authored QML or component-specific rendering branch is introduced.
