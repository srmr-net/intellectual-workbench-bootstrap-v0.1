#!/usr/bin/env bash
set -euo pipefail
exec "$(dirname "$0")/operator-direct/build.sh" "$@"
