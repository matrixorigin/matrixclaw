#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_SH="$ROOT_DIR/scripts/install.sh"

if [[ ! -f "$INSTALL_SH" ]]; then
  echo "ERROR: expected installer script at $INSTALL_SH but it is missing." >&2
  exit 1
fi

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'EOF'
Usage: scripts/install-macos.sh [args...]

Runs the OpenClaw macOS installer.
This is a thin wrapper around scripts/install.sh for Darwin hosts.

Examples:
  scripts/install-macos.sh --help
  scripts/install-macos.sh --no-onboard
  scripts/install-macos.sh --install-method git
EOF
  exit 0
fi

if [[ "$(uname -s 2>/dev/null || true)" != "Darwin" ]]; then
  echo "WARN: install-macos.sh is intended for macOS. Continuing anyway via scripts/install.sh." >&2
fi

OPENCLAW_AUTO_PATH_ADD="${OPENCLAW_AUTO_PATH_ADD:-1}"
if [[ "${OPENCLAW_AUTO_PATH_ADD}" == "1" ]]; then
  OPENCLAW_AUTO_PATH_ADD=1 exec "$INSTALL_SH" --auto-path "$@"
fi

exec "$INSTALL_SH" "$@"
