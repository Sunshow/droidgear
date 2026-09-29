#!/usr/bin/env bash
# Launch a DroidGear Copilot profile with the shared configuration loader.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "${NODE_EXE:-node}" "$SCRIPT_DIR/scripts/copilot-local.js" "$@"
