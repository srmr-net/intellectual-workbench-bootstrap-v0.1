#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
# Preserve every accepted wrapper and test unchanged.
bash apps/desktop-qt/validate-operator.sh
export WB_VS3_EVIDENCE_DIR="$PWD/target/abi-evidence/vs3"
export REGISTRY_PYTHON="$(python3 -c 'import sys; print(sys.executable)')"
export QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1
unset QT_QPA_PLATFORMTHEME DISPLAY WAYLAND_DISPLAY
mkdir -p "$WB_VS3_EVIDENCE_DIR"
manifest=apps/desktop-qt/vslice-003/host/Cargo.toml
cargo fmt --manifest-path "$manifest" -- --check 2>&1 | tee "$WB_VS3_EVIDENCE_DIR/fmt.txt"
cargo build --locked --manifest-path "$manifest" --target-dir target 2>&1 | tee "$WB_VS3_EVIDENCE_DIR/build.txt"
cargo test --locked --manifest-path "$manifest" --target-dir target 2>&1 | tee "$WB_VS3_EVIDENCE_DIR/rust-entry-tests.txt"
cargo clippy --locked --manifest-path "$manifest" --target-dir target --all-targets -- -D warnings 2>&1 | tee "$WB_VS3_EVIDENCE_DIR/clippy.txt"
mkdir -p target/vs3-build
(cd target/vs3-build && qmake6 ../../apps/desktop-qt/vslice-003/operator.pro && make -j2) 2>&1 | tee "$WB_VS3_EVIDENCE_DIR/qt-build.txt"
qmake6 -v > "$WB_VS3_EVIDENCE_DIR/qt-version.txt"
python3 apps/desktop-qt/vslice-003-tests/worker_cases.py 2>&1 | tee "$WB_VS3_EVIDENCE_DIR/worker-tests.txt"
python3 apps/desktop-qt/vslice-003-tests/process_cases.py 2>&1 | tee "$WB_VS3_EVIDENCE_DIR/process-tests.txt"
python3 apps/desktop-qt/vslice-003-tests/evidence_tests.py 2>&1 | tee "$WB_VS3_EVIDENCE_DIR/metadata-tests.txt"
python3 apps/desktop-qt/vslice-003-tests/collect_evidence.py
