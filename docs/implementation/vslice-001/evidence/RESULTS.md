# VSLICE-001 — validation evidence

**Status:** Accepted — 2026-10-26. All twenty mandatory gates, two independent
reviews, clean locked local/hosted checks and downloaded artifact verification
passed. Acceptance is the headless product scope; no accepted ADR is changed.

Base: merged ADR-003 PR #5 at `ae5a0090ad46afa967b5b925b8b8287c1e295a17`;
master hosted run40247542078 passed before this branch. Accepted ADR-001/002/003
statuses and source semantics are unchanged. See TASK.md/DESIGN.md and the
original supplied SPECIFICATION.md/TEST_MATRIX.md for requirements and mappings.

Reproduce using Rust 1.99.1, cargo-component 0.21.1, Wasmtime 49.0.1,
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
strict terminal-newline schema rejection; explicit bounded 20M product caller
fuel rather than changing accepted2M Host defaults; typed negative assertions
respecting early unreferenced-file rejection. Both approved final source hashes,
with concrete independently executed tests. Review records bind these hashes to
implementation `570c692106940c69ee4a49ea0a3af3fa5c6349eb`; see [review records](../review/RESULTS.json).

Unsupported/deferred: Qt/Operator/UI packages, native capabilities, remote
registries, external package resolution, signatures, shared ADR-008 conventions,
room database durability and physical power-cut proof. Required room UI reference
is inert; snapshots recompute derived values. Failure preserves prior output VALUE
while accepted diagnostics/lifecycle may change. No WIT, manifest/ontology fields,
registry scope, lifecycle/bus semantics or original acceptance tests are changed.

Implementation PR: [#6](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/6).

## Clean local execution

Validated source: `f53dd65a3a5b4fca2f1d0fcac6de058ed91be124`, trackedChanges=false.
Implementation/review binding: `570c692106940c69ee4a49ea0a3af3fa5c6349eb`;
only review documentation differs between these commits.

Executed from the repository root:

```sh
PATH=/tmp/wb-spec-001-venv/bin:/tmp/wb-abi-tools/bin:$PATH bash examples/mortgage-headless/validate.sh
```

Exit 0. All 99 Rust tests pass:23 VSLICE tests (including one process-test child
entrypoint) plus all 76 retained ABI/IPC/REG/workspace tests. Clippy with warnings
denied, native rustfmt, all seven authored guest source entrypoints, 30 schema
fixtures/reference JSON/Turtle and four Python tests pass. No ignored tests or
missing prerequisites are accepted as passes. Generator-owned ignored bindings
are compiled, not reformatted; the native required formatting check is unchanged.

Local Linux x86_64, Rust 1.99.1(58cf8f9c3), Cargo 1.99.1(5f94df478),
cargo-component 0.21.1, Wasmtime 49.0.1, guest wit-bindgen-rt 0.41.0,
num-bigint 0.4.8, Python 3.14.7, jsonschema 4.26.0, rdflib 7.6.0, pySHACL 0.30.1.
The locked guest build and clean-target rebuild match raw WASM SHA-256:
`64987f2704fa0d313ec2529e1e0fd9a0a16b8551cc9e42a352c57f950587cb95`.
This is same-machine evidence; hosted bytes may differ and must be pinned exactly.

All 33 actual guest outputs equal the independent Decimal90 reference, including
original vectors, nine boundaries and 16 seeded cases. Both reviewers independently
rechecked critical tests; the numerical reviewer also recomputed all 33 via exact
Fraction forward-balance recurrence. The 2M default probe returns RESOURCE_LIMIT,
budget=fuel; bounded product 20M passes all 33, memory 8 MiB/epoch 100 unchanged.
Slowest recorded local reference invocation 16998 microseconds under concurrent
tests; this is a measurement, not a timing guarantee.

[Complete local logs/inputs](local.zip), SHA-256
`cc9bcc159aba4c36025293e3fb184a17b725d13b4a47ca4e9a02ff6813a8b90b`;
[version/hash manifest](local-run.json). All 75 recorded hashes and actual
installation record/payload hashes are verified. Archive includes exact strict
packages for mortgage/adversary/observer, receipts, record, input envelope,
invocation context, committed result, audit, numerical outputs, calibration,
clean-build log, locks, WIT, source/tests and all validation logs.

## Gate matrix

Every row passed locally and on GitHub-hosted Linux. All rows execute the same
validation wrapper shown above; exact test outcomes are in workspace-tests.txt
inside both archives. VSLICE-019 also requires the external reviews/source binding,
and VSLICE-020 requires actual workflow success/artifact verification, both met.
Rust declaration checks alone are never accepted as those external gates.

| Gate | Test symbols | Local | Hosted |
|---|---|---|---|
| VSLICE-001 | `vslice_001_real_pure_component_and_unchanged_wit` | PASS | PASS |
| VSLICE-002 | `vslice_002_complete_package_actual_admission` | PASS | PASS |
| VSLICE-003 | `vslice_003_exact_pin_and_boundary_integrity` | PASS | PASS |
| VSLICE-004 | `vslice_004_real_lifecycle_and_explicit_policy` | PASS | PASS |
| VSLICE-005 | `vslice_005_actual_full_path_and_correlated_projection`, `vslice_005_real_cli_checks_committed_room_projection` | PASS | PASS |
| VSLICE-006 | `vslice_006_all_vectors_independent_oracle_and_boundaries`, `vslice_006_budget_calibration_retains_accepted_defaults` | PASS | PASS |
| VSLICE-007 | `vslice_007_schema_and_guest_domain_rejection` | PASS | PASS |
| VSLICE-008 | `vslice_008_adversarial_output_atomic_rollback` | PASS | PASS |
| VSLICE-009 | `vslice_009_tamper_never_registers_candidate` | PASS | PASS |
| VSLICE-010 | `vslice_010_admission_discovery_resolution_never_execute_guest` | PASS | PASS |
| VSLICE-011 | `vslice_011_empty_grants_and_forbidden_imports_permissions` | PASS | PASS |
| VSLICE-012 | `vslice_012_real_bus_output_correlation_and_fifo_waves` | PASS | PASS |
| VSLICE-013 | `vslice_013_generation_and_revision_fence_real_completions` | PASS | PASS |
| VSLICE-014 | `vslice_014_bounded_ingress_no_silent_drop` | PASS | PASS |
| VSLICE-015 | `vslice_015_interleaved_and_concurrent_room_isolation` | PASS | PASS |
| VSLICE-016 | `vslice_016_restore_exact_registry_pins_inputs_and_topology` | PASS | PASS |
| VSLICE-017 | `vslice_017_rebuild_evidence_and_repeated_invocation_stability` | PASS | PASS |
| VSLICE-018 | `vslice_018_trap_budget_and_interrupted_install_recover`, `vslice_018_crash_child` | PASS | PASS |
| VSLICE-019 | `vslice_019_independent_review_protocol_is_enforced` | PASS | PASS |
| VSLICE-020 | `vslice_020_all_accepted_regression_sources_unchanged` | PASS | PASS |

## Hosted execution and acceptance

[GitHub-hosted Linux run 40319945570](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40319945570)
concluded **success** on clean exact source
`f53dd65a3a5b4fca2f1d0fcac6de058ed91be124` (trackedChanges=false).
All jobs (rust, contracts, headless including REG/IPC/ABI) passed. All 99 Rust tests,
all twenty VSLICE gates, all ten ABI IDs/twelve original ABI tests, fifteen IPC
IDs/all 24 original IPC tests and twenty REG IDs/all 31 REG tests pass. Clippy,
native formatting, all seven authored guest entrypoints, 30 original schema
fixtures/reference checks and four Python tests pass. No ignored/skipped substitute
is used for a mandatory gate.

Runner Ubuntu 24.04 x86_64, Linux 6.17.0-1022-azure, glibc 2.39; Rust 1.99.1
(58cf8f9c3), Cargo 1.99.1(5f94df478), Wasmtime 49.0.1, cargo-component 0.21.1,
guest wit-bindgen-rt 0.41.0, num-bigint 0.4.8; Python 3.11.17,
jsonschema 4.26.0, rdflib 7.6.0, pySHACL 0.30.1.
[Exact versions and hashes](hosted-run.json).

Artifact `headless-vslice-validation-linux`, ID 12649585432, 391390 bytes,
was downloaded through the GitHub API and archived as [hosted.zip](hosted.zip).
Its SHA-256 equals GitHub's published upload digest:
`90a20e491b55df5d773101b275e650c2138bfe778d9ea254b79a62458ae6ca69`.
All 75 recorded file hashes, actual installation record and every payload hash were
recomputed. Reviewed source files match the archived sources and both review
bindings. Canonical manifest schema, ontology, accepted WIT and original ABI/IPC/REG
test sources match accepted bytes. See [verification details](hosted-verification.json).

Hosted actual pin: `org.workbench.example.mortgage-headless@0.1.0`, raw WASM
`b045130fe32aef76a0bf30cce9072b7b57ea2cb7b6bad37b23dfaf6a6e7573dd`.
Installed record SHA-256:
`3f5c0c1445804db246a0c622479462eabedbae31f5249560a3ec7257cec8b144`.
Native lock SHA-256:
`e95276b610473ba4414377a43d392710a939b3f6197ea7e2caedda27e7b534cf`.
Guest lock SHA-256:
`4da36e512ebfe55c4d5a332d5318a8652e8ac8491fdb900c4c6e21ce7bb08b4c`.
Same-machine clean rebuild matched on both hosts; cross-machine guest bytes differ
and are pinned separately, as permitted by ADR-001. Never silently substitute them.
All 33 actual hosted numerical outputs match both the independent oracle and fixture
expectations. Maximum recorded reference invocation 12405 microseconds; this is
measurement, not a real-time guarantee. Default 2M probe reports fuel exhaustion;
bounded 20M product profile passes while original memory/epoch/defaults stay intact.

## Reverify archived evidence

The following read-only standard-library script was executed successfully against
both committed archives. External hosted workflow conclusion/upload digest were
checked with the GitHub API separately; the offline verifier does not invent them.

```sh
gh run view 40319945570 --repo srmr-net/intellectual-workbench-bootstrap-v0.1 --json status,conclusion,headSha
python3 docs/implementation/vslice-001/evidence/verify.py docs/implementation/vslice-001/evidence/hosted.zip --sha256 90a20e491b55df5d773101b275e650c2138bfe778d9ea254b79a62458ae6ca69 --source f53dd65a3a5b4fca2f1d0fcac6de058ed91be124 --hosted
python3 docs/implementation/vslice-001/evidence/verify.py docs/implementation/vslice-001/evidence/local.zip --sha256 cc9bcc159aba4c36025293e3fb184a17b725d13b4a47ca4e9a02ff6813a8b90b --source f53dd65a3a5b4fca2f1d0fcac6de058ed91be124
```

All required gates are met, with no open mandatory review or architectural blocker.
Acceptance/evidence is a follow-up to the tested source; runtime/guest/test sources
and accepted contracts remain unchanged. Qt, Operator, privileged worlds, remote
registries, signatures, shared decimal conventions and durable room storage remain
separate work. No full GUI or CT-001..015 system-completion claim is made.
