#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
mkdir -p target/abi-evidence/ui target/qt-build target/qt-tests
export WB_REPOSITORY_ROOT="$PWD"
export REGISTRY_PYTHON="$(python3 -c 'import sys; print(sys.executable)')"
export WB_UI_EVIDENCE_DIR="$PWD/target/abi-evidence/ui"
export QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1
unset QT_QPA_PLATFORMTHEME DISPLAY WAYLAND_DISPLAY
# New workspace tests need the real package before the unchanged regression wrapper.
bash examples/mortgage-headless/build.sh
python3 apps/desktop-qt/package_fixture.py
cargo build --locked -p wb-ui-host --bins 2>&1 | tee target/abi-evidence/ui/native-build.txt
# Preserve every accepted ABI/IPC/REG/VSLICE gate and locked Rust/Python check.
bash examples/mortgage-headless/validate.sh
python3 -m unittest discover -s tools -p 'test_*.py' -v 2>&1 | tee target/abi-evidence/ui/python-tests.txt
qmake6 -v > target/abi-evidence/ui/qt-version.txt
pkg-config --modversion Qt6Core Qt6Quick Qt6Test Qt6QuickControls2 >> target/abi-evidence/ui/qt-version.txt
c++ --version > target/abi-evidence/ui/cxx-version.txt
(cd target/qt-build && qmake6 ../../apps/desktop-qt/app.pro && make -j2) 2>&1 | tee target/abi-evidence/ui/qt-build.txt
(cd target/qt-tests && qmake6 ../../apps/desktop-qt/tests/ui_tests.pro && make -j2) 2>&1 | tee target/abi-evidence/ui/qt-test-build.txt
target/qt-tests/ui-conformance -o "$WB_UI_EVIDENCE_DIR/qt-tests.txt",txt -o "$WB_UI_EVIDENCE_DIR/qt-tests.xml",junitxml
python3 apps/desktop-qt/collect_evidence.py
