# ADR-002 validation — lifecycle, IPC and reactive integration

**Status: Accepted — 2026-10-25, after verified hosted IPC-001..015 success.**
[Successful GitHub-hosted Linux run](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40225967487) validates
commit `e19edb2165218de66f64a228fefc75c840ff9cc0`. All three jobs pass. The local
IPC-015 declaration test alone was never treated as hosted acceptance; verified
workflow results and artifact identities close that gate. The implementation
remained Proposed until these results were available.

## Implementation and compatibility

Additive versioned canonical EventEnvelope, plan/snapshot DTOs, lifecycle/audit
records and shared error registry live in wb-contracts::ipc. The legacy prototype
DTOs remain compatible. wb-bus owns finite room mailboxes and host sequence/ID
allocation; wb-reactive owns stable DAG order and exact fences; wb-kernel owns
logical lifecycle, explicit execution approval, plans, invocation tickets, state
transactions and logical restore. Actual compute calls exclusively use the existing
ADR-001 adapter. No WIT, host adapter, original ABI test/guest, existing schema,
ontology, native capability broker or Qt changes were made.

The registry/cache boundary still consumes trusted verified bundle inputs; this is
not a production installer or RDF validator. Execution approvals default deny and
name an exact room/instance/pin. They are not native capability grants. Every Wasm
world remains import-free; the existing disallowed WASI guest still fails admission.

UI acceptance stages uncommitted FIFO events. Each committed input starts one
revision/wave, whose descendants finish in lexical topological order before the
next UI commit. Explicit topology changes still fence detached results. Stop has a
separate control path and revokes active and queued descendant commit authority.
Snapshots contain authoritative whole-port inputs only and reconstruct admitted
running generations/recomputed outputs, never guest memory, queues, UI credentials
or grants. Strict schema/semantic/unit equality and one aggregate input are the
initial profile. See ADR-002 §16 and API READMEs for exact limits and boundaries.

## Executed local verification

The frozen final source is `e19edb2165218de66f64a228fefc75c840ff9cc0`. Its local metadata/logs
are archived in [ADR-002-local/run.json](ADR-002-local/run.json) and adjacent files.
The supplied unrelated task-file relocation remains outstanding locally; all
executable changes were committed, and the hosted checkout reports no changes.


```sh
PATH=/tmp/wb-spec-001-venv/bin:/tmp/wb-abi-tools/bin:$PATH bash examples/wasm-components/validate-ipc.sh
```

The script builds the three original guests and the new independent, import-free
ipc-aggregate guest from the single accepted WIT, then runs:

```sh
cargo test --locked --workspace -- --nocapture
cargo clippy --locked --workspace --all-targets -- -D warnings
python3 tools/check_contracts.py
python3 -m unittest discover -s tools -p 'test_*.py'
cargo fmt --all -- --check
```

All pass locally: **47 Rust tests** (24 IPC tests, including regressions, plus all
23 pre-existing tests), 30 schema fixtures/reference JSON, two Turtle parses, four
Python harness tests, formatting and Clippy with warnings denied. No tests skip.
Native and guest lockfile edits add only the new crate dependency links/guest;
existing Rust/Wasmtime/cargo-component/wit-bindgen pins remain unchanged.
Local metadata may show the user's outstanding VALIDATION_TASK.md relocation;
that unrelated change is preserved and excluded from this implementation.

## IPC acceptance matrix

| Case | Executed checks | Gate |
| --- | --- | --- |
| IPC-001 | Audited declared/prepared/running/stopped path; transition table; no Store execution during admission; removal releases resources | Local + hosted PASS |
| IPC-002 | Invalid transitions, duplicate logical IDs, invoking stopped instance | Local + hosted PASS |
| IPC-003 | Missing pins/digest, bad world, denied execution/native permission, WASI import denial; live approval/UI revocation | Local + hosted PASS |
| IPC-004 | Forged payload routing stays data; host envelopes select registered edges, room/source/sequence; cross-room UI token denied | Local + hosted PASS |
| IPC-005 | Concurrent producers get unique monotonic sequences; lexical DAG order; actual kernel FIFO waves preserve independent inputs and same-source ordering | Local + hosted PASS |
| IPC-006 | Bad ingress rejected before commit/run; invalid second output rejects whole batch and preserves old committed projection | Local + hosted PASS |
| IPC-007 | Aggregate input value+add flows through two independently compiled components; descendant-only stable propagation; no Operator calls | Local + hosted PASS |
| IPC-008 | Exact revision/generation fencing, cancellation/restart, duplicate ingress, active descendant quarantine, new pending UI/unrelated stop preserve active work | Local + hosted PASS |
| IPC-009 | Reject-new bounded queues, no mutation on full, direct stop under saturation, whole fanout batch rejected atomically | Local + hosted PASS |
| IPC-010 | Trap fails only its instance; unrelated instances and concurrently executing rooms continue through separate Stores | Local + hosted PASS |
| IPC-011 | Invalid edge/cycle/stale revision/quota failure leaves topology intact and releases staged reservations | Local + hosted PASS |
| IPC-012 | Exact pinned restore, host inputs/graph, fresh generations, recomputation, no queue/memory/adapter/grant replay; same-name input/output ports preserve input authority | Local + hosted PASS |
| IPC-013 | Guest code/detail, budgets, traps, schemas, cancellation and stale results remain distinct/correlated; audit excludes payload and guest detail | Local + hosted PASS |
| IPC-014 | All unchanged ABI-001..010 tests, original schema fixtures, frozen WIT hash and canonical DTO positive/negative serialization | Local + hosted PASS |
| IPC-015 | All executable checks and clean GitHub-hosted Linux workflow pass; tested SHA, versions and archive verified | Hosted PASS, verified run/artifact |

## Independent review and findings

Two independent reviewers examined the kernel/authority boundary and the
DTO/mailbox/reactive conformance coverage. They found queued-input loss from
premature revisions, direction-name snapshot collisions, guest detail removal and
active-descendant cancellation gaps. Those were corrected with staged FIFO waves,
input-only snapshots, separately redacted audit, and descendant-wide quarantine.
Nine additional regression cases cover those findings, revocation, quota rollback,
the lifecycle table and actual concurrent rooms. No acceptance criterion was relaxed.
One reviewer independently executed the descendant-cancellation regression: PASS.
A final self-audit also added typed first-failure projections and retained static
budget detail without adding audit content; negative assertions were strengthened
to require both no committed value and a typed error. Independent re-review found
no remaining implementation blockers in this spike's scope.

## Reproduction and remaining limits

Use pinned Rust 1.99.1, Wasmtime 49.0.1, cargo-component 0.21.1 and guest
wit-bindgen-rt 0.41.0, installing targets/dependencies as documented under
examples/wasm-components. validate-ipc.sh retains validate.sh and the full ABI
suite; the evidence collector asserts the accepted WIT and unchanged ABI source
hash, records all IPC/ABI case IDs, and archives binaries, WIT, lockfiles, host
source, tests, versions, checks and tested Git SHA. Artifacts are
ipc-validation-linux; raw local output is target/adr-002-local.txt and
machine-readable data lives in target/abi-evidence/{run,ipc-report}.json.

Finite pending queues and invocation/instance leases are engineering test limits,
not total process-memory budgets. Audit/dedup history retain controller-lifetime
records. Original compiler/schema/native-lifting resource limitations remain.
No durable transaction/queue guarantees, native grants, generalized compatibility,
field-level state migration, cross-process IPC, Qt or privileged guest services
are introduced. Other ADRs remain separate. No end-to-end CT completion is claimed.

## Hosted IPC-015 evidence

- Run: [https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40225967487](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40225967487), attempt 1, push event.
- Source: `e19edb2165218de66f64a228fefc75c840ff9cc0`; branch `adr-002-validation`.
- GitHub runner group: GitHub Actions, `ubuntu-24.04` x86_64; platform `Linux-6.17.0-1022-azure-x86_64-with-glibc2.39`.
- Jobs: rust, contracts, ipc-validation (includes ABI): **success**.
- Exactly 47 Rust tests pass: 24 IPC tests covering all 15 case IDs, plus the 23
  pre-existing tests including every original ABI case; no skips/failures.
- Clippy with warnings denied, formatting, 30 schema fixtures/reference JSON/Turtle
  checks, and four Python harness tests pass.
- Versions: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; `cargo 1.99.1 (5f94df478 2026-08-27)`;
  Wasmtime `49.0.1`; cargo-component 0.21.1; wit-bindgen-rt 0.41.0;
  Python 3.11.17; jsonschema 4.26.0; rdflib 7.6.0.
- Artifact: ipc-validation-linux, ID `12611218555`, 177,524 bytes.
- Archive SHA-256: `402ba530292ca82fc9fd0e7d1db1cb8fae448f0f03a0c6e9839c1820b0ce654e`;
  independently downloaded hash equals GitHub's published digest.

[Run API](ADR-002-hosted/run-api.json), [jobs/runner evidence](ADR-002-hosted/jobs-api.json),
[artifact metadata](ADR-002-hosted/artifacts-api.json), [versions/tree](ADR-002-hosted/versions.txt),
[workspace output](ADR-002-hosted/workspace-tests.txt), [Clippy](ADR-002-hosted/clippy.txt),
[contract checks](ADR-002-hosted/contracts.txt), [Python tests](ADR-002-hosted/python-tests.txt),
[guest builds](ADR-002-hosted/guest-build.txt) and [named IPC report](ADR-002-hosted/ipc-report.json)
are committed with [independent verification](ADR-002-hosted/verification.json).
Text logs remove terminal colors and trailing whitespace only. Raw zip/full workflow
log remain in target/adr-002-hosted. Formatting logs are empty on success.

### Recorded immutable artifact/source hashes

Every file below was extracted and rehashed. Both lockfiles, accepted WIT, original
ABI test source and unchanged host source match the checkout byte for byte.
Lockfile changes add dependency links/the IPC guest without external version upgrades.

| File | SHA-256 |
| --- | --- |
| `native-Cargo.lock` | `8e2a14c8ee335c18fe99725eeb4916b335f6f281a6721cb21b7d17c1e31ceb67` |
| `guest-Cargo.lock` | `323be8de5fb3d64e7ffb6470beb00dd04c193e8f5325df00630616c176f266b8` |
| `component.wit` | `cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0` |
| `abi-tests.rs` | `263f74a47a8daf26197f8b3e947fc51e73e05da4030ac008ea416ed4d9542c95` |
| `host-adapter.rs` | `a04bd4f87020d4d06e644dfc324d22f6752f57557273eae4aa543bb6344a2da2` |
| `components/ipc-aggregate.wasm` | `a7a475e540c7193511053aceb9a3e691a0c53ddf28624a3c4d012921994f9627` |
| `components/echo.wasm` | `3957fa6c0b3a7b1d621661e90252a277ffef0120fda18f0e63e9eadd747b2aec` |
| `components/integer.wasm` | `b1dda424bf8cc28936ad4e1610d511def78deb3922bfba2047ee8ec33a375494` |
| `components/denied-wasi.wasm` | `03f3652468822b9c5439ad7c7b6c47b546f282d621279b0402c6f210587617af` |

Hosted/local positive guest bytes differ, as already observed in ADR-001; the
actual digests are recorded separately. This does not assert cross-machine
bit-identical builds or permit artifact substitution. ADR-006 still governs build/
archive canonicalization. The archive contains actual tested components and source
proofs, not installable packages or guest privileges.

### Reproduce and verify

On a checkout of the tested source with pinned tools/dependencies installed:

```sh
bash examples/wasm-components/validate-ipc.sh
mkdir -p target/adr-002-hosted
gh api repos/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40225967487 > target/adr-002-hosted/run-api.json
gh api repos/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40225967487/jobs > target/adr-002-hosted/jobs-api.json
gh api repos/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/40225967487/artifacts > target/adr-002-hosted/artifacts-api.json
gh api repos/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/artifacts/12611218555/zip > target/adr-002-hosted/artifact.zip
python3 docs/adr/evidence/ADR-002-hosted/verify_hosted.py
```

Retrieve hosted artifacts while retained by GitHub. The checked-in verification
script confirms actual hosted runner/job results, source/run attempt, all ABI/IPC
case IDs, 47 test successes, nine file hashes and archive identity. Subsequent
acceptance commits change documentation/evidence only, not tested executable code.

ADR-002 is Accepted only for the documented pure process-local integration profile;
all limits/deferred work above remain. No new WASI imports, native capability grants,
privileged guest services, Qt integration, persistent engine or distributed delivery
is approved by this acceptance. ADR-003..010 remain separate. Full production
kernel/security or CT-01..CT-15 end-to-end completion is not claimed.
