# ADR-003 — Local registry validation evidence

Status: Accepted — 2026-10-26. REG-001..020, independent filesystem/durability
review, clean hosted Linux execution and downloaded artifact verification passed.
Acceptance is limited to ADR-003 §11 and the limitations recorded below.

## Reproduction

Install Rust 1.99.1 with rustfmt, Clippy, wasm32-unknown-unknown and wasm32-wasip1;
cargo-component 0.21.1; and Python 3.11+ with `tools/requirements.txt` (pySHACL
0.30.1). Run from the repository root:

```sh
bash examples/wasm-components/validate-registry.sh
```

The wrapper builds real independent guests and a complete installable pure echo
package, retains the entire accepted ABI/IPC wrapper, runs all workspace tests,
Clippy, formatting, 30 schema fixtures/reference JSON/Turtle checks and Python
harness tests, then validates all twenty REG identifiers and archives evidence in
`target/abi-evidence/`. `run.json` records source SHA, cleanliness, versions and
file hashes; `registry/report.json` records policy and named gates. `registry/`
contains fixture bytes, receipt and content-addressed record from actual admission,
trusted schema/ontology/shapes and exact reviewed implementation/test sources.

Canonical unchanged manifest SHA-256:
`15f2b7a2b84655e62f14fc521fe06a291fd00bf5fbcc120e509aaa84e9e1dd55`.
Unchanged ontology SHA-256:
`1741a428b7ca8bd9d4a761659c30bc5f914fcac60d201ad5361cc54ca9cc3bda`.
New admission SHACL SHA-256:
`1b897c2282e2aec9d4cabdc24d0e6879c94297fe48dedc95174f8fd36e93f051`.
Accepted WIT remains
`cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0`;
original ABI tests remain
`263f74a47a8daf26197f8b3e947fc51e73e05da4030ac008ea416ed4d9542c95`;
original IPC tests remain
`52a67d69f36f85303816ba12255f3e9c9e06a62f20e1523ee29844e778de8238`.

## Gate-to-test map

Each row maps to nonignored production-path Rust tests named `reg_NNN_*` in
`crates/wb-registry/tests/registry.rs`. No security-critical validator or storage
operation is mocked. Every listed gate passed both locally and on hosted Linux at the source commit
recorded below. REG-008 validates the explicitly blocked UI surface; it does not
claim support for package UI rendering.

| Gate | Local | Hosted | Executable scenario |
|---|---|---|---|
| REG-001 | PASS | PASS | Compiled pure fixture, exact bytes, kernel adapter, cross-store handle rejection |
| REG-002 | PASS | PASS | Unknown fields, duplicate keys, malformed versions/ports and unsupported protocol |
| REG-003 | PASS | PASS | Traversal/absolute paths, symlink parents/files, hardlinks, FIFO/socket, source policy |
| REG-004 | PASS | PASS | Missing artifacts/schema, invalid schema, local refs/cycles, unchanged literal/ref-sibling semantics, dialect/vocabulary rejection |
| REG-005 | PASS | PASS | Admission digest mismatch, artifact/record corruption after restart; unavailable, never replaced |
| REG-006 | PASS | PASS | Turtle syntax, actual pySHACL violation and unknown semantic vocabulary |
| REG-007 | PASS | PASS | Independent identity/version/name/direction/schema/type/unit/path/digest mismatches |
| REG-008 | PASS | PASS | UI absent accepted, all UI-bearing packages explicitly blocked including invalid binding/executable UI |
| REG-009 | PASS | PASS | Manifest/RDF permission requests denied, never granted |
| REG-010 | PASS | PASS | Core/malformed/wrong export/type/import rejection; valid trapping initializer admitted without execution |
| REG-011 | PASS | PASS | Identical reinstall, including release-count capacity and restart |
| REG-012 | PASS | PASS | Conflicting artifact under same identity/version preserves original |
| REG-013 | PASS | PASS | Exact resolution, no version/digest fallback; same bytes with distinct version stay separate |
| REG-014 | PASS | PASS | Real SIGKILL at seven checkpoints, restart/read/reinstall; corrupt marker preserves possible pinned object |
| REG-015 | PASS | PASS | Independent thread/process installers/readers using actual flock/rename/fsync |
| REG-016 | PASS | PASS | Only committed metadata discoverable; no inspection/guest tools needed for discovery |
| REG-017 | PASS | PASS | Dependency-shaped declarations, remote/file/traversal schema references rejected |
| REG-018 | PASS | PASS | Forged assurance stays unreviewed; missing evidence rejected |
| REG-019 | PASS | PASS | File/count/depth/total/store/output/address/time bounds, live symlink swaps and parent-death termination |
| REG-020 | PASS | PASS | Frozen WIT/ABI and unchanged complete IPC suite; executed hosted workflow required |

## Independent review

Independent agents `/root/review_registry_filesystem` and
`/root/review_registry_durability` performed the requested filesystem and durability
reviews. Both reviewed production traversal, workers, nonexecution, atomicity and
concurrency. They found and verified corrections for schema literal/ref-sibling
semantics, unsupported dialects, root descriptor ownership, worker parent-death,
metadata reader/writer limits, capacity idempotence, conservative corruption
recovery and syncing recovered/idempotent markers before acknowledgment.

Filesystem reviewer independently ran all seven SIGKILL/reopen checkpoints,
REG-004 schema regressions, worker-parent-death and corrupt-marker preservation.
Durability reviewer independently ran SIGKILL/reopen, both idempotence tests,
thread/process installers and corrupt-record restart. Both approve the documented
Linux/private-store scope, with no remaining implementation blockers.

Final reviewed source hashes:

```text
d6206a80b1c1923f6246838b1fd9573460d2bcd6111efd32d36a7d8332360275  crates/wb-registry/src/lib.rs
5807631011fdf1c41c19a0b5ccb73542740a689ffd08b6c4efd238e723d0893d  crates/wb-semantic/src/lib.rs
6c1eb2987f66584755220f4e5e099abf7223cb012eb52031ac0ad5e45ef9f8d9  tools/registry_validate.py
1adcc72c4b320f2c824902f936244c7082085dd862ceac3882c75a7ce0490298  crates/wb-registry/tests/registry.rs
f397de9e61bd989ad851edfe7cca290910cc108866848a32c9e25c5456586785  crates/wb-registry/src/bin/wb-registry-inspect.rs
```

## Scope and remaining decisions

UI package ownership/rebinding, nonempty permission vocabulary and grants, remote
registries, signatures, archives, external dependencies and removal/GC remain
unsupported. Schema IDs/anchors/dynamic/recursive refs, array-index pointers,
explicit vocabularies and non-2020-12 dialects reject explicitly. No WIT or accepted
lifecycle/message/revision contracts changed.

Tests use real process kills and real OS filesystem operations; they do not
simulate physical power loss. Store/tool files are trusted and host-private.
Same-UID store sabotage, hostile filesystems and complete parser/compiler OS
security isolation remain outside this slice. Logical byte quotas reserve staging
space but do not measure physical filesystem blocks. Worker resource and wall
limits are distinct from cooperative staging I/O deadlines; a hung kernel I/O
operation is not hard-time-bounded. Compilation does not execute guest code.


## Clean local execution

Source: `f275c2147e5981b124da889bb516d8860a7dcfc9`, trackedChanges=false.
PR: [#5](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/5).
Command: `PATH=/tmp/wb-spec-001-venv/bin:/tmp/wb-abi-tools/bin:$PATH bash examples/wasm-components/validate-registry.sh`;
exit 0. All 76 Rust tests pass, including 31 registry tests (three child entrypoints
are exercised by the process tests), all twelve original ABI tests and all 24
unchanged IPC tests. The previous registry/semantic placeholder version tests are
replaced by substantive coverage. No ignored or failed tests. All twenty REG IDs,
Clippy with warnings denied, formatting, 30 schema fixtures/reference JSON/Turtle
and four Python tests pass.

Local platform: Linux 7.2.5-3-omarchy, x86_64, glibc 2.44. Rust 1.99.1
(58cf8f9c3), Cargo 1.99.1 (5f94df478), cargo-component 0.21.1, Wasmtime 49.0.1;
Python 3.14.7, jsonschema 4.26.0, rdflib 7.6.0, pySHACL 0.30.1.
The first sandboxed filesystem run could not create a Unix socket; the mandatory
case was rerun outside that sandbox and passed. No prerequisite is skipped.

[Full local logs and inputs](ADR-003-local.zip), SHA-256
`87a40b74770643af21d7e8db4a56d90ee76dabf17bec2ba3d11d7b45086a97b3`;
[local versions/hash manifest](ADR-003-local-run.json). All 28 recorded file hashes,
including actual installed record/fixture bytes, were recomputed from the archived inputs.
Local pure artifact SHA-256:
`d9499a7a60a7dd7c814de67ae7d3da8aaf5e12d054b0f0555e4b67530be2233d`.


## Hosted Linux execution and acceptance

The complete [push workflow 40246492531](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40246492531)
concluded **success**, at exact source
`f275c2147e5981b124da889bb516d8860a7dcfc9`, with trackedChanges=false. The
[PR workflow 40246628788](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40246628788)
also passed. All jobs (formatting, contracts, registry including full IPC/ABI)
passed. All 76 Rust tests, all twenty REG IDs, ten named ABI IDs plus both
additional original ABI tests, fifteen named IPC IDs across all 24 unchanged IPC
tests, Clippy, format, 30 fixtures/reference JSON/Turtle and four Python tests pass.
No skipped prerequisites or ignored tests are accepted as passes.

Runner: GitHub-hosted Ubuntu 24.04 x86_64, kernel 6.17.0-1022-azure, glibc 2.39.
Rust 1.99.1 (58cf8f9c3), Cargo 1.99.1 (5f94df478), cargo-component 0.21.1,
Wasmtime 49.0.1, Python 3.11.17, jsonschema 4.26.0, rdflib 7.6.0,
pySHACL 0.30.1. Exact metadata:
[hosted versions and hashes](ADR-003-hosted-run.json).

GitHub artifact `registry-validation-linux`, ID `12619156024`, was downloaded
through the GitHub API and archived as [ADR-003-hosted.zip](ADR-003-hosted.zip).
Its SHA-256 equals the GitHub-reported upload digest:
`3b0821c4ad530976d62531a7ae1299e5aca4eaf6c90bb853179795ff4442cdca`.
All 28 recorded file hashes were recomputed from the ZIP, alongside every payload
hash in the actual installed record. Reviewed source files, canonical manifest,
ontology, accepted WIT and original ABI/IPC sources match archived bytes exactly.
The archive includes complete validation logs, actual fixture, receipt, record,
policy, implementation/test sources and pinned inputs.

Hosted compiled pure fixture pin:
`org.workbench.registry.echo@0.1.0` /
`3957fa6c0b3a7b1d621661e90252a277ffef0120fda18f0e63e9eadd747b2aec`.
Installed record SHA-256:
`0478012e1c42fba98d2dfcd93f3876c32c37c63e18cc0c2eb1a0bfb09316463f`.
Native lockfile SHA-256:
`d494a5049fb91a6a458ee0e294e4aa1fb8d9da0c5df588e54877330f495ebae6`.
Guest lockfile SHA-256:
`323be8de5fb3d64e7ffb6470beb00dd04c193e8f5325df00630616c176f266b8`.
Hosted/local WASM hashes differ, as already allowed by ADR-001: no cross-machine
bit-identical build claim is made; each installation resolves exact raw bytes.

All ADR-003 acceptance requirements are satisfied within the explicit supported
surface. Status adoption and evidence are a documentation-only follow-up to the
hosted tested commit; reviewed implementation hashes are unchanged. This does
not mark the complete architecture/Qt/persistence demonstration finished.
