#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
cargo build --locked --manifest-path apps/desktop-qt/operator-chat/host/Cargo.toml --features live-http --target-dir target/operator-chat-native
mkdir -p target/operator-chat-build
(cd target/operator-chat-build && qmake6 ../../apps/desktop-qt/operator-chat/chat.pro && make -j2)
