#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

if [[ "$(uname -s)" == "Linux" && -z "${PKG_CONFIG:-}" && -x /usr/bin/pkg-config ]]; then
  export PKG_CONFIG=/usr/bin/pkg-config
fi

can_run_native_tauri_tests() {
  local pkg_config_bin="${PKG_CONFIG:-pkg-config}"

  case "$(uname -s)" in
    Darwin)
      return 0
      ;;
    Linux)
      if ! command -v "${pkg_config_bin}" >/dev/null 2>&1; then
        return 1
      fi

      "${pkg_config_bin}" --exists gtk+-3.0 gdk-3.0 pango atk webkit2gtk-4.1
      ;;
    *)
      return 1
      ;;
  esac
}

echo "Building bundled UI assets"
bun run --cwd ui build

echo "Running bundled asset packaging test"
cargo test -p matrixclaw-app-host bundled_asset_packaging -- --exact

echo "Running desktop shell bootstrap tests"
bun run --cwd apps/desktop-shell test

if can_run_native_tauri_tests; then
  cargo test --manifest-path apps/desktop-shell/src-tauri/Cargo.toml
else
  echo "Skipping native Tauri Rust tests on this host: missing desktop system prerequisites"
fi

echo "Running desktop UI contract tests"
(
  cd ui
  bunx playwright test \
    tests/desktop_app_shell.spec.ts \
    tests/setup_onboarding_flow.spec.ts \
    tests/workspace_pane_layout.spec.ts
)
