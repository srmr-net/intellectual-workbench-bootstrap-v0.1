#!/usr/bin/env bash
# Retain every accepted ABI/IPC check; add ADR-003 evidence.
set -euo pipefail
cd "$(dirname "$0")/../.."
# The workspace integration tests need an installable compiled package before tests.
bash examples/wasm-components/build.sh
python3 examples/wasm-components/registry_fixture.py
export REGISTRY_PYTHON="$(python3 -c 'import sys; print(sys.executable)')"
bash examples/wasm-components/validate-ipc.sh
python3 examples/wasm-components/registry_evidence.py
