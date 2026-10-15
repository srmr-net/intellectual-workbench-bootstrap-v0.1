# ADR-001 — GitHub-hosted Linux acceptance, 2026-10-15

**ACCEPTED for the existing pure, stateless computation ABI.** The documented
ABI-001 through ABI-010 requirements passed locally and on a GitHub-hosted Linux
runner. Runtime functionality, WIT, manifest and semantic contracts are unchanged.
Production kernel integration and other ADRs are not approved by this decision.

## Separate formatting change

[Formatting-only PR #1](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/pull/1)
contains only rustfmt layout changes and optional trailing commas in ten baseline
crate files. Branch `baseline-formatting` consists of commits `b5fb74e` and
`1adc9c8` over the initial master. Independent review confirmed unchanged behavior,
test inputs and assertions, kernel rejection and capability denial.

The validation branch needs nine of those formatting changes (`bdbf076`); its
wb-wasm-host placeholder was already replaced by the previously reviewed spike.
The standalone branch also formats that original placeholder, preserving its
version test. An initial standalone CI failure identified the omitted placeholder;
it was corrected without weakening any check.

The final standalone branch passes 12 Rust tests, fmt, Clippy and its existing
Python checks locally; both hosted PR jobs pass in
[run 38807620560](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/38807620560).

## Pushed source and hosted run

- [Successful workflow](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/38807099576), attempt 1, triggered by push.
- Tested commit: [`ed73ec7238f1e21192fcbf0bb5698ed1b6197986`](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/commit/ed73ec7238f1e21192fcbf0bb5698ed1b6197986).
- Branch: `adr-001-hosted-validation`; no tracked source modifications in the job.
- GitHub-hosted runner group: `GitHub Actions`; ABI label `ubuntu-24.04`, x86_64.
- ABI runner: `GitHub Actions 1000000076`; platform `Linux-6.17.0-1022-azure-x86_64-with-glibc2.39`.
- Jobs `rust`, `contracts`, and `abi-validation`: **success**.
- Run created 2026-10-15 21:04:55 UTC; completed by 21:12:57 UTC.
- [ABI job](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/38807099576/job/116670649852).

[Run API snapshot](ADR-001-github-linux/run-api.json) and
[jobs/runner snapshot](ADR-001-github-linux/jobs-api.json) bind results to the
pushed commit and actual hosted runner. The subsequent acceptance commit changes
only documentation/evidence, not the executable sources validated at this SHA.

## Exact versions and immutable inputs

| Tool | Hosted value |
| --- | --- |
| Rust | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| Cargo | `cargo 1.99.1 (5f94df478 2026-08-27)` |
| Wasmtime | `49.0.1` (exact Cargo dependency and lockfile) |
| cargo-component | `0.21.1`, installed with `--locked` |
| wit-bindgen-rt | `0.41.0`, guest lockfile |
| Python | `3.11.17` |
| jsonschema / rdflib | `4.26.0` / `7.6.0` |

The CLI reports `cargo-component-component 0.21.1`; its repeated command-name suffix is the
already documented Cargo CLI quirk. Native/guest lockfiles and WIT were downloaded
with the artifact and byte-compared with this checkout. No dependency pins changed.
See [full versions and dependency tree](ADR-001-github-linux/versions.txt) and
[machine-readable run metadata](ADR-001-github-linux/run.json).

## Executed acceptance and local checks

The workflow installs the pinned Rust toolchain and targets, Python requirements,
and cargo-component, then executes:

```sh
bash examples/wasm-components/validate.sh
```

That script independently builds echo/integer and denied-WASI components, then runs:

```sh
cargo test --locked --workspace -- --nocapture
cargo clippy --locked --workspace --all-targets -- -D warnings
python3 tools/check_contracts.py
python3 -m unittest discover -s tools -p 'test_*.py'
cargo fmt --all -- --check
```

All commands pass both locally at the tested commit and on the hosted runner.
The local run used the existing temporary Python/build-tool environments:
`PATH=/tmp/wb-spec-001-venv/bin:/tmp/wb-abi-tools/bin:$PATH bash examples/wasm-components/validate.sh`.
[Local metadata](ADR-001-followup-local/run.json) records the same clean source SHA,
versions, passing checks and local hashes. Its fmt log is empty because fmt passed.

| Acceptance case | Hosted result |
| --- | --- |
| ABI-001 | PASS: guest/host generated from the same WIT; real Component encoding |
| ABI-002 | PASS: typed echo transform |
| ABI-003 | PASS: independent integer guest under the same host |
| ABI-004 | PASS: malformed bytes, core module, missing/wrong export signature rejected |
| ABI-005 | PASS: compiled filesystem-importing WASI guest denied by empty linker and host |
| ABI-006 | PASS: guest error, trap, overflow and recovery remain distinct |
| ABI-007 | PASS: input/output JSON schemas, invalid schemas and registered ports checked |
| ABI-008 | PASS: memory, fuel, epoch, byte/count budgets and successful recovery |
| ABI-009 | PASS: canonical manifest, raw digest and discriminator checks; tampering denied |
| ABI-010 | PASS: hosted clean runner records exact versions, lockfiles, hashes and all tests |

Hosted totals: **23 Rust tests**, zero failures/ignored tests, including all ten
ABI cases and two additional regression tests; Clippy with `-D warnings`; 30 schema
fixtures and reference JSON/Turtle checks; four Python harness tests; formatting.
See [workspace tests](ADR-001-github-linux/workspace-tests.txt),
[Clippy](ADR-001-github-linux/clippy.txt), [contracts](ADR-001-github-linux/contracts.txt),
[Python](ADR-001-github-linux/python-tests.txt) and [guest builds](ADR-001-github-linux/guest-build.txt).

## Artifact identity and independent verification

- Artifact: `abi-validation-linux`, ID `12006654411`.
- [Artifact in the successful run](https://github.com/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/38807099576/artifacts/12006654411).
- Archive size: 135,172 bytes.
- Archive SHA-256: `c260f140121426610fe6df7b639c6b2c32766b5c95206d158414649d3b77e0da`.
- GitHub's reported archive digest equals the SHA-256 calculated after download.
- Every one of the six recorded WIT/lockfile/binary hashes was recomputed from the
  extracted artifact and matched its metadata.

| Artifact file | SHA-256 |
| --- | --- |
| `native-Cargo.lock` | `e3ef3eafaddf6aeffa9e0bf9da0a39bde37b3a005bef55198b68ec88bec93e28` |
| `guest-Cargo.lock` | `c2b227060d5352809f056a1e11102211868264c606813992699df684869f4c30` |
| `component.wit` | `cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0` |
| `components/echo.wasm` | `3957fa6c0b3a7b1d621661e90252a277ffef0120fda18f0e63e9eadd747b2aec` |
| `components/integer.wasm` | `b1dda424bf8cc28936ad4e1610d511def78deb3922bfba2047ee8ec33a375494` |
| `components/denied-wasi.wasm` | `03f3652468822b9c5439ad7c7b6c47b546f282d621279b0402c6f210587617af` |

The two positive guest binary hashes differ from the local build; WIT and both
lockfiles match, and the denied-WASI binary matches. Each environment's manifests
verify its own raw bytes and all ABI tests pass. This is **not** a claim of
bit-identical binaries across machines; reproducibility here satisfies the
unchanged ABI-010 requirement to record exact inputs, versions and executed CI.
Build/archive canonicalization remains ADR-006 work. Never substitute one digest
for another when restoring a pinned room.

The archive contains the exact tested WASM binaries, WIT and both lockfiles; it is
CI evidence, not an installable package. Raw archive and full workflow log remain
under `target/abi-hosted/`. Committed text logs remove terminal color and trailing
whitespace only. Original local evidence is retained as a historical snapshot.

Reproduce artifact retrieval/verification from the repository root while the
GitHub artifact is retained:

```sh
mkdir -p target/abi-hosted
gh api repos/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/38807099576 > target/abi-hosted/run-api.json
gh api repos/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/38807099576/jobs > target/abi-hosted/jobs-api.json
gh api repos/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/runs/38807099576/artifacts > target/abi-hosted/artifacts-api.json
gh api repos/srmr-net/intellectual-workbench-bootstrap-v0.1/actions/artifacts/12006654411/zip > target/abi-hosted/artifact.zip
python3 docs/adr/evidence/ADR-001-github-linux/verify_hosted.py
```

[Verification results](ADR-001-github-linux/verification.json) include the checked
commit, hosted jobs, archive hash, rehashed files and test counts.

## Acceptance scope and deferred work

The prior independent Wasm execution review and this task's independent formatting
review found no remaining code blockers. The earlier formatting/hosted-CI gates
are now satisfied, so ADR-001 is Accepted for its existing stateless pure-compute
world and tested Wasmtime/Linux versions. WIT hash is frozen without changing bytes.

No kernel permission defaults, schema/semantic rules, guest ABI, runtime functionality or
existing test inputs/assertions were changed. No end-to-end CT-01..CT-15 or full G1 production
completion is claimed. Compilation/schema-processing/native-lifting resource
policy, production workload calibration, kernel integration, registry/package
verification, stateful/capability worlds, other operating systems and second-runtime
conformance remain separate work. The initial execution-budget limitations in
ADR-001-results.md remain valid; acceptance does not expand those guarantees.
