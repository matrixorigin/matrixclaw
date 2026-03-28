#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

can_run_native_tauri_tests() {
  case "$(uname -s)" in
    Darwin)
      return 0
      ;;
    Linux)
      if ! command -v pkg-config >/dev/null 2>&1; then
        return 1
      fi

      pkg-config --exists gtk+-3.0 gdk-3.0 pango atk webkit2gtk-4.1
      ;;
    *)
      return 1
      ;;
  esac
}

echo "Building bundled UI assets"
pnpm --dir ui build

echo "Running bundled asset packaging test"
cargo test -p matrixclaw-app-host bundled_asset_packaging -- --exact

echo "Running desktop shell bootstrap tests"
pnpm --dir apps/desktop-shell test

if can_run_native_tauri_tests; then
  cargo test --manifest-path apps/desktop-shell/src-tauri/Cargo.toml
else
  echo "Skipping native Tauri Rust tests on this host: missing desktop system prerequisites"
fi

echo "Running desktop UI contract tests"
pnpm --dir ui exec playwright test \
  ui/tests/desktop_app_shell.spec.ts \
  ui/tests/setup_onboarding_flow.spec.ts \
  ui/tests/workspace_pane_layout.spec.ts
