#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
export PATH="$PWD/target/vs2-venv/bin:/tmp/wb-abi-tools/bin:$PATH"
bash examples/mortgage-headless/build.sh
python3 apps/desktop-qt/package_fixture.py
cargo build --locked -p wb-desktop-host --bins -p wb-registry --bin wb-registry-inspect
# Preserve a production-only worker separately; never run a mock worker here.
cargo build --locked --manifest-path apps/desktop-qt/vslice-004/host/Cargo.toml --features live-http --target-dir target
mkdir -p target/operator-direct-native
cp target/debug/wb-vslice004-worker target/operator-direct-native/wb-vslice004-worker
cargo build --locked --manifest-path apps/desktop-qt/vslice-004/host/Cargo.toml --features test-transport --target-dir target
mkdir -p target/operator-direct-build
(cd target/operator-direct-build && qmake6 ../../apps/desktop-qt/operator-direct/operator.pro && make -j2)
