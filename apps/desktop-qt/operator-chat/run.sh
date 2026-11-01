#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../../.." && pwd)"
fixture=()
while (($#)); do
 case "$1" in
  --credential-file) [[ $# -ge 2 ]] || exit 2; export WB_LLM_SECRET_FILE="$2"; shift 2;;
  --fixture) fixture=(--fixture); shift;;
  *) echo 'Usage: workbench [--credential-file /private/key-file] [--fixture] | workbench --tools [tool-options]' >&2; exit 2;;
 esac
done
worker="$root/target/operator-chat-native/debug/wb-operator-chat"
if ((${#fixture[@]})); then
 worker="$root/target/operator-chat-fixture/debug/wb-operator-chat"
 unset WB_LLM_SECRET_FILE OPENROUTER_API_KEY OPENAI_API_KEY
elif [[ -z "${WB_LLM_SECRET_FILE:-}" && -z "${OPENROUTER_API_KEY:-}" && -f "$HOME/openrouter.key" ]]; then
 # The private source explicitly supplied by the user; contents stay host-only.
 export WB_LLM_SECRET_FILE="$HOME/openrouter.key"
fi
[[ -x "$worker" && -x "$root/target/operator-chat-build/workbench-operator-chat" ]] || { echo 'Build with ./workbench --build first. The fixture needs the test build.' >&2; exit 2; }
exec "$root/target/operator-chat-build/workbench-operator-chat" "$worker" "$root/apps/desktop-qt/operator-direct/run.sh" "${fixture[@]}"
