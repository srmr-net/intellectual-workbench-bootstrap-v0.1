#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
# Full accepted chain, unchanged: VS3/OP/VS2/UI/VS1/REG/IPC/ABI.
bash apps/desktop-qt/validate-vslice-003.sh
export WB_LLM_EVIDENCE_DIR="$PWD/target/abi-evidence/llm"
export REGISTRY_PYTHON="$(python3 -c 'import sys; print(sys.executable)')"
export QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1
unset QT_QPA_PLATFORMTHEME DISPLAY WAYLAND_DISPLAY OPENAI_API_KEY WB_LLM_SECRET_FILE WB_LLM_TEST_TRANSPORT WB_LLM_MODEL WB_LLM_BUDGET WB_LLM_PROVIDER OPENROUTER_API_KEY
mkdir -p "$WB_LLM_EVIDENCE_DIR"
manifest=apps/desktop-qt/live-provider/host/Cargo.toml
cargo fmt --manifest-path "$manifest" -- --check 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/fmt.txt"
cargo test --locked --manifest-path "$manifest" --target-dir target 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/default-tests.txt"
cargo test --locked --manifest-path "$manifest" --features test-transport --target-dir target 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/mock-tests.txt"
cargo clippy --locked --manifest-path "$manifest" --all-features --target-dir target --all-targets -- -D warnings 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/clippy.txt"
# This worker contains no enabled live HTTP path. Never use --all-features here.
cargo build --locked --manifest-path "$manifest" --features test-transport --target-dir target 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/mock-build.txt"
mkdir -p target/llm-qt-build
(cd target/llm-qt-build && qmake6 ../../apps/desktop-qt/live-provider/operator.pro && make -j2) 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/qt-build.txt"
python3 apps/desktop-qt/live-provider-tests/transport_tests.py 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/transport-tests.txt"
python3 apps/desktop-qt/live-provider-tests/process_cases.py 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/process-tests.txt"
python3 apps/desktop-qt/live-provider-tests/evidence_tests.py 2>&1 | tee "$WB_LLM_EVIDENCE_DIR/metadata-tests.txt"
python3 apps/desktop-qt/live-provider-tests/collect_evidence.py
