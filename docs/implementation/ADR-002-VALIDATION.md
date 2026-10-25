# Codex implementation task — ADR-002 validation spike

Read `AGENTS.md`, the accepted ADR-001 and hosted evidence, Architecture/Interface Specification v0.1, Component Package Specification v0.1, and `ADR-002.md`.

## Objective
Implement the **smallest testable host-side component lifecycle and process-local IPC path** without changing `wit/component.wit` or relaxing the existing ABI suite. Keep new DTOs/versioning in `wb-contracts`; orchestration in `wb-kernel`; queues/envelopes in `wb-bus`; graph execution in `wb-reactive`; use the proven `wb-wasm-host` rather than copying its execution logic.

## Suggested execution order
1. Inventory existing crate interfaces and test fixtures; report discrepancies against ADR-002 before changing them.
2. Define versioned envelope DTO and shared errors (including additions in ADR-002); add serialization fixtures and rejection tests.
3. Implement room-owned instance lifecycle machine with audited transitions and generation tokens.
4. Implement per-room, finite-capacity process-local message queue, monotonic sequencing and privileged stop/cancel path.
5. Add typed ingress/egress validation hooks; wire output batches through all-or-nothing validation.
6. Implement deterministic DAG order, invalidation and stale result checks; explicitly support aggregate input-port pattern.
7. Integrate the existing ADR-001 host adapter with the kernel; no ambient WASI imports or new WIT definitions.
8. Demonstrate two instances connected by a validated edge; process a UI-originating input and obtain output under the expected revision.
9. Implement IPC-001–014 and retain ABI-001–010; add Linux CI evidence for IPC-015.
10. Update ADR-002 status only when all gates pass; otherwise record what failed and keep status PROPOSED.

## Scope boundaries
Do not implement Qt FFI, cloud registries, new database engine, WASI filesystem access, or an agent runtime. Do not assume `component.json` validation implies authorization. Isolate formatting-only baseline repairs in separate changes.

## Deliverables
- Executable kernel/bus/reactive integration spike
- DTO JSON fixtures and negative cases
- Lifecycle transition table implemented as tests
- Tests and named IPC conformance report
- Hosted CI link and toolchain/build evidence
- Review notes identifying version-sensitive host behavior

## Suggested verification commands
```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --locked --workspace
# additionally run existing ABI guest build and Python contract suites as documented in repository
```

## Stop conditions
If source contracts disagree, a shared schema requires a breaking edit, or new host imports appear necessary, stop that work package and propose a separate ADR rather than silently modifying the public ABI.
