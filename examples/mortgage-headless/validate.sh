#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
mkdir -p target/abi-evidence/vslice
bash examples/mortgage-headless/build.sh 2>&1 | tee target/abi-evidence/vslice/build.txt
python3 examples/mortgage-headless/reference.py 2>&1 | tee target/abi-evidence/vslice/reference.txt
python3 examples/mortgage-headless/rebuild.py
cargo build --locked -p wb-kernel -p wb-registry --bins 2>&1 | tee target/abi-evidence/vslice/native-build.txt
# Retain every original registry/IPC/ABI suite and every local check.
bash examples/wasm-components/validate-registry.sh
# Check every authored guest entry point; generated ignored bindings.rs stays
# generator-owned and is not rewritten to satisfy a newer rustfmt layout.
rustfmt --edition 2021 --config skip_children=true --check examples/wasm-components/*/src/lib.rs > target/abi-evidence/vslice/guest-fmt.txt 2>&1
python3 examples/mortgage-headless/collect_evidence.py
