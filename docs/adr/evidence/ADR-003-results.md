# ADR-003 — Local registry validation evidence

Status: Proposed; independent review complete, final clean local/hosted execution
and artifact verification pending. This report does not claim hosted acceptance.

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
operation is mocked. Actual results and hosted source/run are recorded below
once the final validations finish.

| Gate | Executable scenario |
|---|---|
| REG-001 | Compiled pure fixture, exact bytes, kernel adapter, cross-store handle rejection |
| REG-002 | Unknown fields, duplicate keys, malformed versions/ports and unsupported protocol |
| REG-003 | Traversal/absolute paths, symlink parents/files, hardlinks, FIFO/socket, source policy |
| REG-004 | Missing artifacts/schema, invalid schema, local refs/cycles, unchanged literal/ref-sibling semantics, dialect/vocabulary rejection |
| REG-005 | Admission digest mismatch, artifact/record corruption after restart; unavailable, never replaced |
| REG-006 | Turtle syntax, actual pySHACL violation and unknown semantic vocabulary |
| REG-007 | Independent identity/version/name/direction/schema/type/unit/path/digest mismatches |
| REG-008 | UI absent accepted, all UI-bearing packages explicitly blocked including invalid binding/executable UI |
| REG-009 | Manifest/RDF permission requests denied, never granted |
| REG-010 | Core/malformed/wrong export/type/import rejection; valid trapping initializer admitted without execution |
| REG-011 | Identical reinstall, including release-count capacity and restart |
| REG-012 | Conflicting artifact under same identity/version preserves original |
| REG-013 | Exact resolution, no version/digest fallback; same bytes with distinct version stay separate |
| REG-014 | Real SIGKILL at seven checkpoints, restart/read/reinstall; corrupt marker preserves possible pinned object |
| REG-015 | Independent thread/process installers/readers using actual flock/rename/fsync |
| REG-016 | Only committed metadata discoverable; no inspection/guest tools needed for discovery |
| REG-017 | Dependency-shaped declarations, remote/file/traversal schema references rejected |
| REG-018 | Forged assurance stays unreviewed; missing evidence rejected |
| REG-019 | File/count/depth/total/store/output/address/time bounds, live symlink swaps and parent-death termination |
| REG-020 | Frozen WIT/ABI and unchanged complete IPC suite; executed hosted workflow required |

## Independent review

Two independent reviewers were explicitly requested for filesystem and durability
review. Both reviewed production traversal, workers, nonexecution, atomicity and
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
