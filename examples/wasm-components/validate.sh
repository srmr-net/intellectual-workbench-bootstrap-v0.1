#!/usr/bin/env bash
# Run from a clean checkout after installing the pinned toolchain and cargo-component.
set -euo pipefail
cd "$(dirname "$0")/../.."
report="$PWD/target/abi-evidence"
mkdir -p "$report"
export CARGO_COMPONENT_CACHE_DIR="$PWD/target/abi-component-cache"
{
  uname -a
  rustc -Vv
  cargo -V
  cargo component -V
  cargo component --help
  cargo component build --help
  cargo tree --locked -p wb-wasm-host
} > "$report/versions.txt" 2>&1
bash examples/wasm-components/build.sh 2>&1 | tee "$report/guest-build.txt"
sha256sum wit/component.wit Cargo.lock examples/wasm-components/Cargo.lock target/abi-guests/wasm32-unknown-unknown/release/abi_echo.wasm target/abi-guests/wasm32-unknown-unknown/release/abi_integer.wasm target/abi-guests/wasm32-wasip1/release/abi_denied_wasi.wasm > "$report/sha256.txt"
cargo test --locked --workspace -- --nocapture 2>&1 | tee "$report/workspace-tests.txt"
cargo clippy --locked --workspace --all-targets -- -D warnings 2>&1 | tee "$report/clippy.txt"
python3 tools/check_contracts.py 2>&1 | tee "$report/contracts.txt"
python3 -m unittest discover -s tools -p 'test_*.py' 2>&1 | tee "$report/python-tests.txt"
# Retain the full formatting check; publish success metadata only after it passes.
cargo fmt --all -- --check > "$report/fmt.txt" 2>&1
python3 examples/wasm-components/collect_evidence.py
