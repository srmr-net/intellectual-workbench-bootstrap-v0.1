# RUNTIME-001 — ADR-001 validation spike

- **Owner:** Codex; branch `adr-001-validation`.
- **Goal:** Validate the proposed pure WIT world with independently built guests and ABI-001..010.
- **Contract versions:** supplied `workbench:component@0.1.0`; manifest discriminator unchanged.
- **Dependencies / ADRs:** explicit validation authorization in VALIDATION_TASK.md; ADR-001 conditional; SPEC-002 integration deferred.
- **Owned paths:** wb-wasm-host, examples/wasm-components, wit/component.wit, ADR-001 and evidence; necessary workspace/toolchain/CI/lockfiles.
- **Forbidden paths:** manifest schema, ontology, semantic rules, kernel and permissions; upstream references.
- **Inputs / fixtures:** authoritative WIT; independent Rust echo/integer guests; disallowed WASI guest; malformed component fixtures.
- **Required changes:** generated bindings, typed host, schema/port checks, digest checks, isolated stores and concrete budgets, Linux CI and evidence.
- **Explicit non-goals:** general runtime integration, granted imports, registry installation, financial arithmetic, Qt.
- **Acceptance checks:** build guests with pinned cargo-component; `cargo test --locked --workspace`; `cargo fmt --all -- --check`; `cargo clippy --locked --workspace --all-targets -- -D warnings`; `python3 tools/check_contracts.py`; ABI suite via documented script.
- **Threat / failure cases:** mismatched world, WASI imports, malformed payload/schema/port, trap, fuel/epoch/memory exhaustion, oversized results and changed digest.
- **Review required:** independent review of Wasm execution boundary; ADR stays proposed until all mandatory evidence including Linux CI exists.
- **Deliverables:** source, locked dependencies, guest build/validation scripts, regression tests and ADR-001-results.md.
- **Known blockers:** no cargo-component initially installed; no remote/hosted CI execution currently available.

## Verification / PR review description

**Intent:** Validate the proposed stateless WIT ABI using independently compiled echo/integer components and a Wasmtime-only pure-compute adapter; keep production kernel integration gated.

**Changed contracts:** Authoritative supplied WIT checked in, generated bindings, pinned Rust/Wasmtime/cargo-component inputs. Manifest and semantic contracts preserved. No capabilities or WASI linkers introduced.

**Results:** 23 workspace tests pass including 12 ABI/regression tests; Clippy, 30 schema fixtures/reference JSON/Turtle and four Python harness tests pass. Required whole-workspace fmt fails in unchanged baseline sources; changed source formatting passes. Clean guest rebuild yields identical hashes. Independent Wasm boundary review completed, all code findings addressed. Full evidence: ADR-001-results.md and ADR-001-linux logs.

**Compatibility:** Wasmtime49.0.1/Linux only; Rust1.99.1, cargo-component0.21.1. No cross-runtime/platform portability claim, no installer or mortgage example changes.

**Risks / remaining work:** Hosted clean-machine Linux CI not executed (no remote). Formatting baseline failure remains visible. Compilation/host validation/lifting allocations need production admission policy. ADR-001 remains Proposed; other architectural decisions unchanged. No hosted PR can be submitted without a remote; this description is ready for review.
