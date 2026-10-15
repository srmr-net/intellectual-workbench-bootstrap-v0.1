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

## Completed verification

- Separate formatting-only PR: https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/1 (ten baseline files, independently reviewed; 12 standalone Rust tests and both hosted jobs pass).
- Validation branch formatting commit: `bdbf076`; unchanged existing spike host already formatted.
- Clean local source `ed73ec7238f1e21192fcbf0bb5698ed1b6197986`: complete validation script passes, 23 Rust tests, Clippy, 30 schema fixtures/reference JSON/Turtle, four Python harness tests and fmt.
- Hosted run: https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/38807099576; all jobs pass on GitHub-hosted Linux at that exact SHA.
- Downloaded archive ID `12006654411` matches GitHub SHA-256 `c260f140121426610fe6df7b639c6b2c32766b5c95206d158414649d3b77e0da`; all six file hashes verified; WIT/lockfiles equal checkout. Positive-guest raw bytes differ from local and are recorded distinctly.
- ADR-001 Accepted only after these checks; full report: ADR-001-hosted-results.md.
- No runtime/ABI/manifest/semantic functionality changes; subsequent commit contains evidence and status documentation only.
