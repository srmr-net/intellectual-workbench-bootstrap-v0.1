#!/usr/bin/env bash
# ADR-002 adds no alternate execution path: retain every ADR-001 check.
set -euo pipefail
cd "$(dirname "$0")/../.."
bash examples/wasm-components/validate.sh
python3 - <<'PY'
from pathlib import Path
import json
p=Path('target/abi-evidence/ipc-report.json')
report=json.loads(p.read_text())
assert len(report['cases']) == 15
assert report['witUnchanged']
print('PASS: IPC-001..015 executable checks; final hosted acceptance requires a successful workflow run.')
PY
