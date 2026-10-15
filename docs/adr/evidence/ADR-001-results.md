# ADR-001 validation results — 2026-10-15

**Historical local snapshot — superseded by [hosted acceptance](ADR-001-hosted-results.md).**
At this initial stage the decision remained PROPOSED. Local ABI execution passed. Required hosted Linux CI
has not run: this checkout has no Git remote, so no hosted PR or CI URL can be
provided. Whole-workspace formatting also fails in unchanged baseline crates.
A configured workflow or passing local `abi_010` test is not a successful CI run.

Archived text logs remove terminal color codes and trailing whitespace only;
original captured output remains under `target/abi-evidence/`.

## Executed environment and pins

- Linux x86_64, Omarchy 4.0.4; kernel `7.2.5-3-omarchy`.
- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; `cargo 1.99.1 (5f94df478 2026-08-27)`.
- Wasmtime **49.0.1**, default features, exact dependency pin; root Cargo.lock committed.
- cargo-component **0.21.1**, installed with `--locked`; wit-bindgen-rt **0.41.0**.
- Guest targets: `wasm32-unknown-unknown` for echo/integer,
  `wasm32-wasip1` for the intentionally denied filesystem guest.
- Separate guest Cargo.lock committed. Authoritative WIT is only `wit/component.wit`.
- Python 3.14.7 virtual environment `/tmp/wb-spec-001-venv` with jsonschema 4.26.0
  and rdflib 7.6.0 (declared tools/requirements.txt ranges).

Full versions, CLI help and dependency tree: [versions.txt](ADR-001-linux/versions.txt).
Official tool instructions consulted: [cargo-component](https://github.com/bytecodealliance/cargo-component),
[Wasmtime 49.0.1](https://docs.rs/wasmtime/49.0.1/wasmtime/).
Only this Linux/Wasmtime combination was executed; Windows/macOS and a second
Component Model runtime are deferred. No binary portability beyond Wasmtime is claimed.

## Commands actually executed

Initial `rustc --version`, `cargo --version` and `cargo test --workspace` passed
(12 baseline scaffold unit tests). `cargo component --version` and help initially
failed because the subcommand was absent. Installation/build prerequisites:

```sh
cargo install cargo-component --version 0.21.1 --locked --root /tmp/wb-abi-tools
rustup target add wasm32-unknown-unknown wasm32-wasip1
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown --target wasm32-wasip1
cargo fetch
cargo fetch --manifest-path examples/wasm-components/Cargo.toml
```

Downloads/target installation required sandbox approval. An attempted toolchain
alias was rejected by rustup; the exact official 1.99.1 distribution was installed
and used instead. The guest build script uses the tool's own workspace cache
setting, `CARGO_COMPONENT_CACHE_DIR`, and never changes HOME.

After inspecting both `cargo component --help` and `cargo component build --help`:

```sh
PATH=/tmp/wb-spec-001-venv/bin:/tmp/wb-abi-tools/bin:$PATH bash examples/wasm-components/validate.sh
cargo test --locked -p wb-wasm-host --test abi -- --nocapture
rustfmt --edition 2021 --check crates/wb-wasm-host/src/lib.rs crates/wb-wasm-host/tests/abi.rs
rustfmt --edition 2021 --check --config skip_children=true examples/wasm-components/echo/src/lib.rs examples/wasm-components/integer/src/lib.rs examples/wasm-components/denied-wasi/src/lib.rs
```

`validate.sh` runs three separate locked cargo-component builds, records hashes,
then executes the entire workspace, Clippy, Python contract and harness checks,
and whole-workspace rustfmt. It exits **1** at the final formatting check; preceding
checks pass. Changed host/guest source formatting passes independently.

A clean guest rebuild was also executed (only generated guest build output removed):

```sh
cargo clean --manifest-path examples/wasm-components/Cargo.toml --target-dir target/abi-guests
PATH=/tmp/wb-abi-tools/bin:$PATH bash examples/wasm-components/build.sh
```

All three rebuilt guest SHA-256 values match the preceding build. See
[clean rebuild log](ADR-001-linux/clean-guest-rebuild.txt) and [recorded hashes](ADR-001-linux/sha256.txt).
Same-path local rebuild evidence does not assert identical artifacts across machines.

## Guest SHA-256

| Artifact | SHA-256 of raw component bytes |
| --- | --- |
| echo | `d9499a7a60a7dd7c814de67ae7d3da8aaf5e12d054b0f0555e4b67530be2233d` |
| integer | `871d862cd23aef8f746861c42ca01976eca453f09c3600b7efb6292d0f7d54bd` |
| denied WASI | `03f3652468822b9c5439ad7c7b6c47b546f282d621279b0402c6f210587617af` |

## Acceptance matrix

All automated ABI tests are ordinary non-ignored workspace tests. Missing guest
artifacts fail with a build instruction; there is no skip path.

| ID | Executed result | Evidence |
| --- | --- | --- |
| ABI-001 | PASS locally | cargo-component generates bindings from root WIT; Wasmtime bindgen compiles; both artifacts have Component encoding and register |
| ABI-002 | PASS locally | echo input `hello` produces typed result `HELLO` |
| ABI-003 | PASS locally | separately compiled integer guest maps 21 to 42 unchanged under the same Host; checked overflow is a guest error |
| ABI-004 | PASS locally | invalid bytes, core module, empty component and incompatible `compute` signature rejected as World |
| ABI-005 | PASS locally | compiled std::fs guest contains real `wasi:filesystem` imports; empty linker cannot prepare it; pure Host rejects imports |
| ABI-006 | PASS locally | guest EXPECTED error and engine trap remain distinct; successful calls continue in isolated fresh stores |
| ABI-007 | PASS locally | malformed/invalid input rejected before execution, malformed/invalid output rejected afterward, undeclared ports rejected; bad/remote schemas rejected |
| ABI-008 | PASS locally | loops exhaust fuel; separate effectively unlimited-fuel loop hits epoch deadline; memory growth, output bytes/count and input bytes fail with Budget; successful calls recover. Additional memory/table/instance count fixtures exercise Budget classification |
| ABI-009 | PASS locally | generated manifests validate unchanged canonical schema; raw digest verified; modified bytes, wrong kind/interface and duplicate ports rejected |
| ABI-010 | Local checks PASS; hosted CI NOT RUN | exact toolchain/native+guest lockfiles, version-enforcing builder, Linux workflow and archived logs/hashes exist; clean local guest rebuild matches. Hosted clean-machine run remains mandatory |

The final workspace run passes **23 tests** (11 scaffold unit tests plus 12 ABI/
regression tests), with zero failures/skips, plus doc-test suites. Clippy passes
with `-D warnings`. Python passes 30 schema fixtures, reference JSON, two Turtle
parses and four harness tests. See [workspace](ADR-001-linux/workspace-tests.txt),
[Clippy](ADR-001-linux/clippy.txt), [contracts](ADR-001-linux/contracts.txt),
[Python tests](ADR-001-linux/python-tests.txt) and [guest build](ADR-001-linux/guest-build.txt).

Whole-workspace rustfmt fails on existing wb-bus, wb-capabilities, wb-contracts,
wb-kernel, wb-reactive, wb-registry, wb-semantic, wb-storage and wb-testkit source.
Those files were not edited or their checks weakened. [Full diff](ADR-001-linux/fmt.txt).
The Linux workflow retains the formatting failure and uploads ABI logs even when red.

## Findings and independent review

An initial ABI-004 run failed: Wasmtime 49's generated interface `Pre::new` checks
export indices without checking the nested function signature. Registration now
explicitly typechecks `compute` against generated Message/ComponentError types
before any guest code starts. All ABI tests were rerun successfully afterward.

The independent reviewer ran the then-current 11-test ABI suite successfully,
reviewed fresh-store isolation, empty linkers, generated signature checks and CI
prerequisites, and found no remaining code blockers. Review identified cross-engine
error misclassification (fixed with Engine::same and regression), missing guest
build prerequisites in the old CI job (tests moved to ABI job), and resource-count
error classification (fixed and covered with the final twelfth test).
Count-cap errors use the pinned Wasmtime 49 error-chain prefix; executable memory,
table and instance regression wrappers guard that version-sensitive mapping.
Final full workspace run includes that additional test and passes.

## Limits and remaining work

These are execution-spike budgets, not approval for production hostile components:
8 MiB linear memory, one memory, two tables (1,024 elements each), eight core
instances, 2,000,000 fuel, 100 engine ticks at 10 ms, 16 KiB input/batch bytes and
eight output messages. One pure event gets one Store; no WASI or native imports
are linked. Failure of any returned message rejects the entire batch.

Fuel and epoch budgets cover initialization, compute and canonical call cleanup.
They do not bound Component::new compilation or host schema validation. Canonical
lifting allocates output before result-byte/count checks; execution memory/fuel
bounds are coarse protection, not a complete native allocation budget. Full
production admission needs compiler isolation/resource policy and schema/lifting
allocation analysis. Epoch deadlines depend on OS scheduling, not hard real time.

The native API accepts trusted parsed manifest/schema values and raw artifact
bytes, not unverified filesystem paths. Duplicate-key-safe parsing, safe bundle
path/symlink resolution, RDF cross-validation, semantic/unit compatibility,
installation and permission policy remain separate gates. Generated manifests are
ABI-only fixtures, not installable packages. The mortgage example stays metadata-only.

Manifest schema, semantic contracts, ontology, kernel and permission defaults are
unchanged. No shared wb-contracts DTOs or production runtime integration was added.
ADR-002..010 remain unresolved; this task does not approve them. Required next
acceptance evidence is a successful hosted Linux CI run after the existing baseline
formatting issue is handled within appropriate ownership. ADR-001 stays Proposed.
