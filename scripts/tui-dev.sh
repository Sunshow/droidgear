#!/bin/bash
# Start TUI in development mode with cargo watch for auto-rebuild.
# Usage: npm run tui:dev
#        npm run tui:dev -- --home /tmp/test-home
#
# If cargo-watch is installed, rebuilds on file changes.
# Otherwise falls back to a single cargo run.

set -eo pipefail

EXTRA_ARGS="${*:-}"
TUI_PACKAGE="droidgear-tui"

if command -v cargo-watch &>/dev/null; then
  echo "Starting TUI dev with cargo-watch (auto-rebuild on changes)..."
  cd src-tauri && cargo watch -x "run -p ${TUI_PACKAGE} -- ${EXTRA_ARGS}"
else
  echo "cargo-watch not found, running once. Install with: cargo install cargo-watch"
  cd src-tauri && cargo run -p "${TUI_PACKAGE}" -- ${EXTRA_ARGS}
fi
