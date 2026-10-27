#!/usr/bin/env bash
# ADR005 opt-in entry; every accepted wrapper remains unchanged beneath it.
set -euo pipefail
cd "$(dirname "$0")/../.."
export WB_REPOSITORY_ROOT="$PWD"
export REGISTRY_PYTHON="$(python3 -c 'import sys; print(sys.executable)')"
export WB_OPERATOR_EVIDENCE_DIR="$PWD/target/abi-evidence/operator"
export QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1
unset QT_QPA_PLATFORMTHEME DISPLAY WAYLAND_DISPLAY
python3 - <<'PY'
from pathlib import Path
import tempfile,shutil
out=Path('target/abi-evidence/operator')
if out.exists():
 destination=Path(tempfile.mkdtemp(prefix='operator-evidence-prior-',dir='target'))/'operator'
 shutil.move(str(out),str(destination))
out.mkdir(parents=True,exist_ok=True)
PY
# Includes real guest builds, all locked Cargo checks, Python validators,
# accepted ABI/IPC/REG/UI/VS1/VS2 gates and genuine Qt/process regressions.
bash apps/desktop-qt/validate-vslice-002.sh
cargo build --locked -p wb-operator --bins 2>&1 | tee "$WB_OPERATOR_EVIDENCE_DIR/native-build.txt"
mkdir -p target/operator-build
(cd target/operator-build && qmake6 ../../apps/desktop-qt/operator/operator.pro && make -j2) 2>&1 | tee "$WB_OPERATOR_EVIDENCE_DIR/qt-build.txt"
python3 apps/desktop-qt/operator-tests/worker_cases.py 2>&1 | tee "$WB_OPERATOR_EVIDENCE_DIR/worker-tests.txt"
python3 apps/desktop-qt/operator-tests/process_cases.py 2>&1 | tee "$WB_OPERATOR_EVIDENCE_DIR/process-tests.txt"
sha256sum target/operator-build/workbench-operator target/debug/wb-operator-worker > "$WB_OPERATOR_EVIDENCE_DIR/binary-sha256.txt"
qmake6 -v > "$WB_OPERATOR_EVIDENCE_DIR/qt-version.txt"
python3 apps/desktop-qt/operator-tests/collect_evidence.py
