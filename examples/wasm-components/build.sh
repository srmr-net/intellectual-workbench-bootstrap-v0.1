#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
# Pinned cargo-component 0.21.1 (Cargo 1.99 may repeat the command name).
[[ "$(cargo component --version)" == *" 0.21.1" ]] || { echo 'Require cargo-component 0.21.1' >&2; exit 1; }
export CARGO_TARGET_DIR="$PWD/target/abi-guests"
export CARGO_COMPONENT_CACHE_DIR="$PWD/target/abi-component-cache"
cargo component build --manifest-path examples/wasm-components/Cargo.toml -p abi-echo --target wasm32-unknown-unknown --release --locked
cargo component build --manifest-path examples/wasm-components/Cargo.toml -p abi-integer --target wasm32-unknown-unknown --release --locked
cargo component build --manifest-path examples/wasm-components/Cargo.toml -p abi-denied-wasi --target wasm32-wasip1 --release --locked
cargo component build --manifest-path examples/wasm-components/Cargo.toml -p ipc-aggregate --target wasm32-unknown-unknown --release --locked
python3 examples/wasm-components/bundle_fixtures.py
