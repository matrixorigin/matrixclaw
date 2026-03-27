#!/bin/sh
set -eu

if [ -z "${MATRIXCLAW_SOURCE_BIN:-}" ]; then
  echo "MATRIXCLAW_SOURCE_BIN is required" >&2
  exit 1
fi

install_dir="${HOME:-$PWD}/.matrixclaw/bin"
mkdir -p "$install_dir"
cp "$MATRIXCLAW_SOURCE_BIN" "$install_dir/matrixclaw"
chmod +x "$install_dir/matrixclaw"
