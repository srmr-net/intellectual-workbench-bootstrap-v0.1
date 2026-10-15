# SPEC-001 — Machine-validatable JSON contracts

- **Owner:** Codex; branch `spec-001-schemas`.
- **Goal:** Strict composition, semantic UI and room schemas with positive/negative fixtures.
- **Contract versions:** Draft 2020-12; Workbench 0.1.
- **Dependencies / ADRs:** Architecture §§7,9,12; package specification §6. ADR-001 remains OPEN; new ADR-011 records proposed schema details.
- **Owned paths:** schemas/ (excluding component.schema.json), fixtures/, tools/, this task file, docs/adr/ADR-011.md.
- **Forbidden paths:** references/, Rust crates, runtime, canonical component manifest schema.
- **Inputs / fixtures:** Existing manifest and mortgage UI; architecture examples.
- **Required changes:** Closed object schemas, immutable pins, declarative binding grammar, fixture discovery and offline reference resolution.
- **Explicit non-goals:** Runtime, permissions policy, semantic edge/binding resolution, persistence implementation, SPEC-002 DTOs.
- **Acceptance checks (exact commands):** `cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `python3 tools/check_contracts.py`; `python3 -m unittest discover -s tools -p 'test_*.py'`.
- **Threat / failure cases:** Unknown fields/controls, executable bindings, mutable pins, stale/negative revisions, stored grants, unsafe UI references.
- **Review required:** Shared protocol and proposed ADR-011 review before adoption; no trust boundary implementation.
- **Deliverables (code, tests, docs):** Three schemas, schema README, fixture matrix, checker/tests and decision record.
- **Known blockers:** ADR-001 through ADR-010 remain unresolved; schema detail decisions in ADR-011 are proposed, not approved. Existing manifest defects must be surfaced without modifying its schema.

## Verification results

- `cargo fmt --all -- --check`: FAIL on pre-existing formatting in Rust scaffolding (wb-bus, wb-capabilities, wb-kernel, wb-reactive, wb-registry, wb-semantic, wb-storage, wb-testkit, wb-wasm-host). Rust paths were not edited.
- `cargo test --workspace`: PASS, 12 unit tests plus doc-test suites.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `python3 tools/check_contracts.py`: system Python reports missing optional dependencies (exit 2). Installed declared requirements into `/tmp/wb-spec-001-venv`; `/tmp/wb-spec-001-venv/bin/python tools/check_contracts.py`: PASS, 30 fixtures, reference JSON and two Turtle parses.
- `/tmp/wb-spec-001-venv/bin/python -m unittest discover -s tools -p 'test_*.py'`: PASS, four harness/contract tests.
- `git diff --check`: PASS. Canonical manifest schema and references have no diff.
- Existing untracked Cargo.lock preserved and excluded. No WASM runtime implementation begun.
- No Git remote is configured, so a hosted PR cannot be submitted; review description is provided below.

## PR description

**Intent:** Supply the missing SPEC-001 G0 composition, declarative semantic UI and room shape contracts with offline positive/negative fixture validation.

**Changed contracts:** Three proposed Draft 2020-12 schemas; closed object/control vocabularies, exact pins, revision fields, declarative paths and permission requests. The canonical manifest schema is unchanged. ADR-011 records details requiring shared-contract review.

**Verification:** Results above. Formatting remains a baseline failure outside task ownership.

**Compatibility:** Existing manifest and mortgage UI remain valid. No Rust DTO, runtime, renderer, registry or storage implementation changes.

**Risks / remaining work:** ADR-011 review before contract adoption; ADR-001..010 remain OPEN. Graph/port/binding resolution, unique logical identifiers, authorization, revision concurrency and artifact restore need host validation. No end-to-end CT passes claimed.
