#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
export WB_CHAT_EVIDENCE_DIR="$PWD/target/chat-001-evidence"
mkdir -p "$WB_CHAT_EVIDENCE_DIR" target/operator-chat-build target/operator-chat-tests
unset WB_LLM_SECRET_FILE OPENROUTER_API_KEY OPENAI_API_KEY QT_QPA_PLATFORMTHEME DISPLAY WAYLAND_DISPLAY
export QT_QPA_PLATFORM=offscreen
manifest=apps/desktop-qt/operator-chat/host/Cargo.toml
cargo fmt --manifest-path "$manifest" -- --check 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/fmt.txt"
cargo test --locked --manifest-path "$manifest" --all-features --target-dir target/operator-chat-host 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/native-tests.txt"
cargo clippy --locked --manifest-path "$manifest" --all-features --all-targets --target-dir target/operator-chat-host -- -D warnings 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/clippy.txt"
cargo build --locked --manifest-path "$manifest" --features live-http --target-dir target/operator-chat-native 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/production-build.txt"
cargo build --locked --manifest-path "$manifest" --features test-transport --target-dir target/operator-chat-fixture 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/fixture-build.txt"
python3 apps/desktop-qt/operator-chat/process_tests.py -v 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/process-tests.txt"
(cd target/operator-chat-build && qmake6 ../../apps/desktop-qt/operator-chat/chat.pro && make -j2) 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/app-build.txt"
(cd target/operator-chat-tests && qmake6 ../../apps/desktop-qt/operator-chat/tests.pro && make -j2) 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/test-build.txt"
export WB_CHAT_TEST_WORKER="$PWD/target/operator-chat-fixture/debug/wb-operator-chat"
target/operator-chat-tests/operator-chat-tests -o -,txt -o "$WB_CHAT_EVIDENCE_DIR/qt-tests.xml",junitxml 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/qt-tests.txt"
qmake6 -v 2>&1 | tee "$WB_CHAT_EVIDENCE_DIR/qt-version.txt"
rustc -Vv | tee "$WB_CHAT_EVIDENCE_DIR/rust-version.txt"
python3 apps/desktop-qt/operator-chat/collect.py
