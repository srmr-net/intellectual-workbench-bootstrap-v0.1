# Codex implementation task: ADR-001 / RUNTIME-001

Read `AGENTS.md`, `docs/CONTRACT_STATUS.md`, `references/architecture_interface_v0.1.md` and the component package spec. Implement the **smallest executable Component Model validation spike**. This is an ADR validation task, NOT permission to add general guest capabilities.

## Allowed edits
- `docs/adr/ADR-001.md`, a new root `wit/component.wit`, and ADR evidence docs.
- `crates/wb-wasm-host/**`, `crates/wb-testkit/**`, and new `examples/wasm-components/**`.
- Necessary Cargo workspace/CI declarations and narrowly scoped fixtures.
- Do NOT modify ontology, manifest schema, semantic port rules or kernel permission defaults without separate approval.

## Requirements
1. Use exactly the WIT package and world in `wit/component.wit`, and check in that single authoritative copy. Generate guest and host bindings; do not duplicate WIT by hand.
2. Pin a tested release of Wasmtime (candidate `=49.0.1`) and toolchain via lockfile/toolchain file; identify exact cargo-component version. If 49.0.1 cannot compile, document why and select an actually tested pinned alternative.
3. Build **two independently compiled WASM components**: an echo/transform guest and a numeric calculation guest (use integer arithmetic to avoid opening ADR-008). Both export the same world and require zero host imports.
4. Implement a host loading each `.wasm` as a `wasmtime::component::Component`, typechecking and instantiating using generated bindings, and invoking `compute`.
5. Validate incoming/outgoing `payload-json` with JSON Schema fixtures. Ensure typed port names match registered manifest declarations. Error on unrecognised output port.
6. Demonstrate each of ABI-001 through ABI-010 from ADR-001 with automated positive and negative tests. For ABI-005 compile a deliberately disallowed WASI-import guest; do not merely inspect a manifest.
7. Implement concrete Wasmtime memory and instruction/time limits, with regression tests for guest loops, oversized results, traps and denied imports. Avoid brittle wall-clock-only CI assertions; combine fuel and epoch as appropriate.
8. Keep error classes separate: guest-reported error, Wasmtime trap, invalid WIT world, schema violation, exceeded resource budget.
9. Produce `docs/adr/evidence/ADR-001-results.md` with executed commands, OS/architecture, versions, test results, limitations and SHA-256 of guest binaries. Never claim tests passed without running them.
10. Build/test under Linux CI; note additional platform tests deferred rather than claiming cross-platform coverage.

## Exit criteria
- All 10 ABI acceptance tests demonstrably pass on a clean machine/CI.
- `cargo test --workspace` passes, relevant fixture validators pass.
- ADR's status can be changed to ACCEPTED *only with linked reproducible evidence*.

## First command sequence to verify or adapt
```sh
rustc --version
cargo --version
cargo component --version
cargo test --workspace
```

Run `cargo component --help` and inspect the installed CLI before producing build scripts. Follow current official instructions for producing actual Component Model artifacts.
