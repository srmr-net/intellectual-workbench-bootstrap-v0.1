#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
export WB_REPOSITORY_ROOT="$PWD"
export REGISTRY_PYTHON="$(python3 -c 'import sys; print(sys.executable)')"
export WB_VS2_EVIDENCE_DIR="$PWD/target/abi-evidence/vs2"
export QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1
unset QT_QPA_PLATFORMTHEME DISPLAY WAYLAND_DISPLAY
mkdir -p "$WB_VS2_EVIDENCE_DIR" target/desktop-build target/desktop-tests
# Prepare every real fixture needed by new workspace tests before old wrappers.
bash examples/wasm-components/build.sh
python3 examples/wasm-components/registry_fixture.py
bash examples/mortgage-headless/build.sh
python3 apps/desktop-qt/package_fixture.py
python3 apps/desktop-qt/desktop-tests/prepare_fixtures.py
cargo build --locked -p wb-desktop-host -p wb-registry --bins 2>&1 | tee "$WB_VS2_EVIDENCE_DIR/native-build.txt"
(cd target/desktop-build && qmake6 ../../apps/desktop-qt/desktop/desktop.pro && make -j2) 2>&1 | tee "$WB_VS2_EVIDENCE_DIR/desktop-build.txt"
(cd target/desktop-tests && qmake6 ../../apps/desktop-qt/desktop-tests/desktop_tests.pro && make -j2) 2>&1 | tee "$WB_VS2_EVIDENCE_DIR/qt-build.txt"
# This unchanged wrapper retains all UI/ABI/IPC/REG/VSLICE001 tests, original
# collectors, Rustfmt, locked workspace tests/Clippy and Python contract checks.
bash apps/desktop-qt/validate.sh
# Individual VS2 process cases launch the very same product executable, report
# its hash, PID/childPID and actual Qt accessible/rendered values, then cold reopen.
python3 apps/desktop-qt/desktop-tests/process_cases.py 2>&1 | tee "$WB_VS2_EVIDENCE_DIR/process-tests.txt"
target/desktop-tests/vs2-qt-conformance -o "$WB_VS2_EVIDENCE_DIR/qt-tests.txt",txt -o "$WB_VS2_EVIDENCE_DIR/qt-tests.xml",junitxml
python3 apps/desktop-qt/desktop-tests/check_snapshot_fixtures.py 2>&1 | tee "$WB_VS2_EVIDENCE_DIR/snapshot-fixtures.txt"
sha256sum target/desktop-build/workbench-desktop target/debug/wb-desktop-worker target/debug/wb-desktop-install target/debug/wb-desktop-offline > "$WB_VS2_EVIDENCE_DIR/binary-sha256.txt"
qmake6 -v > "$WB_VS2_EVIDENCE_DIR/qt-version.txt"
pkg-config --modversion Qt6Core Qt6Quick Qt6Test Qt6Widgets >> "$WB_VS2_EVIDENCE_DIR/qt-version.txt"
c++ --version > "$WB_VS2_EVIDENCE_DIR/cxx-version.txt"
python3 apps/desktop-qt/desktop-tests/collect_evidence.py
