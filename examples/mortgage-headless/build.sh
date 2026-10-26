#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
[[ "$(cargo component --version)" == *" 0.21.1" ]]
export CARGO_TARGET_DIR="$PWD/target/abi-guests"
export CARGO_COMPONENT_CACHE_DIR="$PWD/target/abi-component-cache"
cargo component build --manifest-path examples/wasm-components/Cargo.toml -p mortgage-headless -p mortgage-adversary -p mortgage-observer --target wasm32-unknown-unknown --release --locked
python3 examples/mortgage-headless/package.py
