#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
profile="${XDG_STATE_HOME:-$HOME/.local/state}/intellectual-workbench-operator"
python="${REGISTRY_PYTHON:-$root/target/vs2-venv/bin/python}"
extra=()
trace=()
while (($#)); do
 case "$1" in
  --live) extra=(--live); shift;;
  --trace) trace=(--trace "$2"); shift 2;;
  --credential-file) export WB_LLM_SECRET_FILE="$2"; shift 2;;
  --profile) profile="$2"; shift 2;;
  *) echo 'Usage: run.sh [--live] [--credential-file /private/key-file] [--profile /private/directory] [--trace /private/new-file.jsonl]' >&2; exit 2;;
 esac
done
[[ "$profile" = /* && "$profile" != / && ! -L "$profile" ]] || exit 2
umask 077
mkdir -p "$profile"
[[ -O "$profile" && "$(stat -c %a "$profile")" = 700 ]] || { echo 'Profile must be private, owned by you, with mode700.' >&2; exit 2; }
[[ -x "$root/target/operator-direct-native/wb-vslice004-worker" && -x "$root/target/operator-direct-build/workbench-operator-direct" ]] || { echo 'Run apps/desktop-qt/operator-direct/build.sh first.' >&2; exit 2; }
"$root/target/debug/wb-desktop-install" "$profile/registry" "$python" "$root/tools/registry_validate.py" "$root/target/debug/wb-registry-inspect" "$root/target/ui-fixtures/mortgage-ui"
exec "$root/target/operator-direct-build/workbench-operator-direct" "$root/target/operator-direct-native/wb-vslice004-worker" "$profile/registry" "$profile/workspace" "$python" "$root/tools/registry_validate.py" "$root/target/debug/wb-registry-inspect" "${extra[@]}" "${trace[@]}"
