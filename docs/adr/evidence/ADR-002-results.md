# ADR-002 validation — lifecycle, IPC and reactive integration

**Status: Proposed — local gates passed; hosted IPC-015 pending.** No hosted
success or Accepted decision is inferred from the IPC-015 declaration test.

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
| IPC-001 | Audited declared/prepared/running/stopped path; transition table; no Store execution during admission; removal releases resources | Local PASS |
| IPC-002 | Invalid transitions, duplicate logical IDs, invoking stopped instance | Local PASS |
| IPC-003 | Missing pins/digest, bad world, denied execution/native permission, WASI import denial; live approval/UI revocation | Local PASS |
| IPC-004 | Forged payload routing stays data; host envelopes select registered edges, room/source/sequence; cross-room UI token denied | Local PASS |
| IPC-005 | Concurrent producers get unique monotonic sequences; lexical DAG order; actual kernel FIFO waves preserve independent inputs and same-source ordering | Local PASS |
| IPC-006 | Bad ingress rejected before commit/run; invalid second output rejects whole batch and preserves old committed projection | Local PASS |
| IPC-007 | Aggregate input value+add flows through two independently compiled components; descendant-only stable propagation; no Operator calls | Local PASS |
| IPC-008 | Exact revision/generation fencing, cancellation/restart, duplicate ingress, active descendant quarantine, new pending UI/unrelated stop preserve active work | Local PASS |
| IPC-009 | Reject-new bounded queues, no mutation on full, direct stop under saturation, whole fanout batch rejected atomically | Local PASS |
| IPC-010 | Trap fails only its instance; unrelated instances and concurrently executing rooms continue through separate Stores | Local PASS |
| IPC-011 | Invalid edge/cycle/stale revision/quota failure leaves topology intact and releases staged reservations | Local PASS |
| IPC-012 | Exact pinned restore, host inputs/graph, fresh generations, recomputation, no queue/memory/adapter/grant replay; same-name input/output ports preserve input authority | Local PASS |
| IPC-013 | Guest code/detail, budgets, traps, schemas, cancellation and stale results remain distinct/correlated; audit excludes payload and guest detail | Local PASS |
| IPC-014 | All unchanged ABI-001..010 tests, original schema fixtures, frozen WIT hash and canonical DTO positive/negative serialization | Local PASS |
| IPC-015 | Local declaration/collector checks pass; successful hosted clean Linux workflow with tested SHA/versions/hashes still required | Hosted pending |

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
