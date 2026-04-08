#!/bin/sh
set -eu

if [ -z "${ZSTAR_SOURCE_BIN:-}" ]; then
  echo "ZSTAR_SOURCE_BIN is required" >&2
  exit 1
fi

install_dir="${HOME:-$PWD}/.zstar/bin"
mkdir -p "$install_dir"
cp "$ZSTAR_SOURCE_BIN" "$install_dir/zstar"
chmod +x "$install_dir/zstar"
