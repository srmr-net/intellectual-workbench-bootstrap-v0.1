#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
# Retain the complete accepted ABI/IPC/REG/UI/OP/LLM/VS1–3 and VS4 deterministic chain.
bash apps/desktop-qt/validate-vslice-004.sh
export WB_UX_EVIDENCE_DIR="$PWD/target/abi-evidence/ux001"
export WB_REPOSITORY_ROOT="$PWD"
export REGISTRY_PYTHON="$(python3 -c 'import sys; print(sys.executable)')"
export QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1
unset QT_QPA_PLATFORMTHEME DISPLAY WAYLAND_DISPLAY OPENROUTER_API_KEY OPENAI_API_KEY WB_LLM_SECRET_FILE WB_VS4_TEST_MODE WB_VS4_TEST_CAPTURE
mkdir -p "$WB_UX_EVIDENCE_DIR" target/operator-direct-build target/operator-direct-tests target/ux-001-tests
(cd target/operator-direct-build && qmake6 ../../apps/desktop-qt/operator-direct/operator.pro && make -j2) 2>&1 | tee "$WB_UX_EVIDENCE_DIR/build.txt"
(cd target/operator-direct-tests && qmake6 ../../apps/desktop-qt/operator-direct/tests.pro && make -j2) 2>&1 | tee "$WB_UX_EVIDENCE_DIR/reference-build.txt"
(cd target/ux-001-tests && qmake6 ../../apps/desktop-qt/operator-direct/ux_tests.pro && make -j2) 2>&1 | tee "$WB_UX_EVIDENCE_DIR/test-build.txt"
target/operator-direct-tests/operator-direct-tests 2>&1 | tee "$WB_UX_EVIDENCE_DIR/reference-tests.txt"
target/ux-001-tests/ux-001-tests -o -,txt -o "$WB_UX_EVIDENCE_DIR/tests.xml",junitxml 2>&1 | tee "$WB_UX_EVIDENCE_DIR/tests.txt"
qmake6 -v 2>&1 | tee "$WB_UX_EVIDENCE_DIR/qt-version.txt"
python3 -m unittest discover -s docs/implementation/ux-001/evidence -p test_verify.py -v 2>&1 | tee "$WB_UX_EVIDENCE_DIR/provenance-tests.txt"
python3 docs/implementation/ux-001/evidence/collect.py
