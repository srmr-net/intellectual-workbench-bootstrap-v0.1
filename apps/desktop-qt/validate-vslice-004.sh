#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
# Preserve the complete accepted chain, including ADR006 mock-only validation.
bash apps/desktop-qt/validate-live-provider.sh
export WB_VS4_EVIDENCE_DIR="$PWD/target/abi-evidence/vs4"
export REGISTRY_PYTHON="$(python3 -c 'import sys; print(sys.executable)')"
export QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1
unset QT_QPA_PLATFORMTHEME DISPLAY WAYLAND_DISPLAY OPENAI_API_KEY OPENROUTER_API_KEY WB_LLM_SECRET_FILE WB_LLM_TEST_TRANSPORT WB_LLM_MODEL WB_LLM_BUDGET WB_LLM_PROVIDER WB_VS4_TEST_MODE WB_VS4_TEST_CAPTURE
mkdir -p "$WB_VS4_EVIDENCE_DIR"
manifest=apps/desktop-qt/vslice-004/host/Cargo.toml
cargo fmt --manifest-path "$manifest" -- --check 2>&1 | tee "$WB_VS4_EVIDENCE_DIR/fmt.txt"
cargo test --locked --manifest-path "$manifest" --target-dir target 2>&1 | tee "$WB_VS4_EVIDENCE_DIR/default-tests.txt"
cargo test --locked --manifest-path "$manifest" --features test-transport --target-dir target 2>&1 | tee "$WB_VS4_EVIDENCE_DIR/mock-tests.txt"
cargo clippy --locked --manifest-path "$manifest" --all-features --target-dir target --all-targets -- -D warnings 2>&1 | tee "$WB_VS4_EVIDENCE_DIR/clippy.txt"
# Never combine live-http with test-transport for desktop tests.
cargo build --locked --manifest-path "$manifest" --features test-transport --target-dir target 2>&1 | tee "$WB_VS4_EVIDENCE_DIR/mock-build.txt"
mkdir -p target/vs4-qt-build
(cd target/vs4-qt-build && qmake6 ../../apps/desktop-qt/vslice-004/operator.pro && make -j2) 2>&1 | tee "$WB_VS4_EVIDENCE_DIR/qt-build.txt"
python3 apps/desktop-qt/vslice-004-tests/process_cases.py 2>&1 | tee "$WB_VS4_EVIDENCE_DIR/process-tests.txt"
python3 apps/desktop-qt/vslice-004-tests/evidence_tests.py 2>&1 | tee "$WB_VS4_EVIDENCE_DIR/metadata-tests.txt"
python3 apps/desktop-qt/vslice-004-tests/collect_evidence.py
