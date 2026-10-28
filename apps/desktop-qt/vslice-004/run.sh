#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
profile="${XDG_STATE_HOME:-$HOME/.local/state}/intellectual-workbench"
python="${REGISTRY_PYTHON:-$root/target/vs2-venv/bin/python}"
package="$root/target/ui-fixtures/mortgage-ui"
credential=''
actions=''
report=''
while (($#)); do
  case "$1" in
    --profile) profile="$2"; shift 2;;
    --credential-file) credential="$2"; shift 2;;
    --package) package="$2"; shift 2;;
    --validation-actions) actions="$2"; shift 2;;
    --validation-report) report="$2"; shift 2;;
    *) echo 'Usage: run.sh [--profile /absolute/private/directory] [--credential-file /absolute/private/key-file] [--package /absolute/admitted-package]' >&2; exit 2;;
  esac
done
[[ "$profile" = /* && "$profile" != / && ! -L "$profile" ]] || { echo 'Profile must be an absolute non-symlink private directory' >&2; exit 2; }
umask 077
mkdir -p "$profile"
[[ -O "$profile" && "$(stat -c %a "$profile")" = 700 ]] || { echo 'Profile must be owned by you with mode 700' >&2; exit 2; }
for executable in "$root/target/vs4-qt-build/workbench-vslice004" "$root/target/debug/wb-vslice004-worker" "$root/target/debug/wb-desktop-install" "$python"; do
  [[ -x "$executable" ]] || { echo 'Build first: see apps/desktop-qt/vslice-004/README.md' >&2; exit 2; }
done
if [[ -n "$credential" ]]; then
  [[ "$credential" = /* ]] || { echo 'Credential source must be an absolute private file path, never the key text' >&2; exit 2; }
  export WB_LLM_SECRET_FILE="$credential"
fi
# No key contents enter this launcher, registry, room snapshot or Qt widgets.
# Admission reads the actual manifest; no component ID/version/digest is built in.
"$root/target/debug/wb-desktop-install" "$profile/registry" "$python" "$root/tools/registry_validate.py" "$root/target/debug/wb-registry-inspect" "$package"
validation_args=()
if [[ -n "$actions" || -n "$report" ]]; then
  [[ -n "$actions" && -n "$report" ]] || { echo 'Validation actions and report must be supplied together' >&2; exit 2; }
  validation_args=(--validation-actions "$actions" --validation-report "$report")
fi
exec "$root/target/vs4-qt-build/workbench-vslice004" "$root/target/debug/wb-vslice004-worker" "$profile/registry" "$profile/workspace" "$python" "$root/tools/registry_validate.py" "$root/target/debug/wb-registry-inspect" "${validation_args[@]}"
