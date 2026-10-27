# Intellectual Workbench — Bootstrap Repository v0.1

**State:** G0 bootstrap scaffold, not a finished Workbench application. Origin: Architecture and Interface Specification v0.1, Minimal Semantic Ontology v0.1, Component Package & Manifest Specification v0.1.

## What exists

- Ten Rust crates with explicit responsibility boundaries and starter contract types.
- Versioned upstream reference documents, actual component JSON Schema, provisional RDF ontology and the reference package (metadata-only: **no `logic.wasm`**).
- A safe deny-by-default capability broker, structural bus validation and a minimal reactive dependency-graph validator as starter code. These are **not** production security enforcement.
- Agent instructions (`AGENTS.md`), backlog, ADR templates, gates, task templates and CI.
- Separately gated ADR-004 semantic Qt frontend and native worker; [build and validation](apps/desktop-qt/README.md). See ADR status before claiming hosted acceptance.
- Executable Python package validator and schema fixture smoke test (when `jsonschema` is installed).

## Prerequisites

Rust stable (cargo/rustfmt/clippy), Python 3.11+, optionally `pip install -r tools/requirements.txt`. Accepted pure Component Model runtime uses pinned Wasmtime49.0.1. The separately gated semantic desktop profile requires system Qt6 development/QML modules.

## Quick start

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 tools/check_contracts.py
```

`check_contracts.py` validates schema fixtures, reference JSON and Turtle syntax if optional dependencies are installed; it explicitly reports SKIP otherwise. `tools/validate_package.py` will **fail** on `examples/mortgage-calc.wbc` because the intentionally missing WASM binary makes it un-installable.

## First Codex task

Read `AGENTS.md`, `docs/implementation/BACKLOG.md` and `docs/adr/ADR-001.md`. Start with **SPEC-001**; produce missing composition/UI/room JSON Schemas and positive/negative conformance fixtures without changing upstream `component.schema.json`. Do not implement a runtime based on the illustrative WIT sketch until ADR-001 is accepted.

## Contract boundaries

`component.json` is authoritative for identity, ports, permissions, artifacts. RDF assists semantic discovery. Live plans, reactive state and UI data are JSON/typed runtime objects, not executable RDF. The namespace `https://workbench.example/ns/0.1#` is provisional. No arbitrary component-supplied QML is executable.

See `docs/CONTRACT_STATUS.md` for binding versus unresolved decisions.

The accepted [headless mortgage vertical slice](examples/mortgage-headless/README.md)
builds a real local package and exercises accepted registry/lifecycle/WIT/bus APIs.
Its [task and acceptance matrix](docs/implementation/vslice-001/README.md) track
product evidence separately from the original metadata-only mortgage reference.

The accepted [interactive Linux desktop and persistent canvas](apps/desktop-qt/desktop/README.md)
adds installed-component selection, pinning and zero-compute cold display of exact
committed snapshots. Build/test with `bash apps/desktop-qt/validate-vslice-002.sh`;
see [VSLICE-002 evidence](docs/implementation/vslice-002/evidence/RESULTS.md).

The opt-in bounded local Operator adds explicit request, exact-pin preview and fresh
consent to the existing shell. Build/check with
`bash apps/desktop-qt/validate-operator.sh`; see the [Operator entry](apps/desktop-qt/operator/README.md)
and [ADR005 gates](docs/implementation/adr-005/CONFORMANCE_MAP.md). Its ADR status
requires independent review and successful archived hosted evidence.
