# RUNTIME-001 follow-up — baseline formatting and hosted acceptance

- **Owner:** Codex; branch `adr-001-hosted-validation`, based on `adr-001-validation`.
- **Goal:** Separate formatting-only baseline cleanup, rerun local checks and obtain GitHub-hosted Linux evidence for ADR-001.
- **Contract versions:** Existing WIT/manifest/schema versions unchanged.
- **Dependencies / ADRs:** ADR-001 and ADR-001-results.md; explicit user authorization to push and execute hosted CI.
- **Owned paths:** Baseline crates' Rust formatting only; GitHub workflow and validation evidence collection; ADR-001 and associated evidence/status documentation.
- **Forbidden paths:** Runtime functionality, component ABI/WIT, manifest/schema/ontology and semantic behavior, existing test expectations, references/.
- **Inputs / fixtures:** Existing checked-in ABI tests, guest sources, both lockfiles and evidence.
- **Required changes:** Dedicated formatting-only commit; complete local checks; verify/configure CI; push branch, execute Linux runner; record run URL, exact versions, checks and hashes; accept ADR only if all documented requirements pass.
- **Explicit non-goals:** New capabilities, runtime functionality, broader platform/runtime claims, merging into master.
- **Acceptance checks:** `cargo fmt --all -- --check`; `cargo test --locked --workspace`; `cargo clippy --locked --workspace --all-targets -- -D warnings`; `python3 tools/check_contracts.py`; `python3 -m unittest discover -s tools -p 'test_*.py'`; `bash examples/wasm-components/validate.sh`; hosted workflow success with artifacts tied to pushed SHA.
- **Threat / failure cases:** Formatting changes behavior; missing guest prerequisites; hosted job skips ABI checks; evidence mismatches SHA; unavailable authentication/runner; unsupported pinned tools.
- **Review required:** Independent review of baseline kernel formatting and unchanged behavior. Previous spike independent review remains recorded.
- **Deliverables:** Separate formatting commit, configured workflow as needed, local/hosted evidence and conditional ADR acceptance.
- **Known blockers:** Hosted authentication/runner access being checked; no success claimed before executed results.
