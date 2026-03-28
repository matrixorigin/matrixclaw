# Task 001: Implement bundled asset packaging

**depends-on**: `task-001-write-bundled-asset-packaging-test-test`

## Goal
Make UI asset resolution product-safe by bundling the built frontend into the packaged app and removing repo-relative assumptions from the launch path.

## Scenario
Scenario: Packaged app launches without repo-relative assets
  Given MatrixClaw is launched from a packaged desktop installation
  When the runtime resolves shell and static UI assets
  Then it serves bundled application assets without requiring the repository layout
  And a missing source checkout does not break `/setup` or `/workspace`
  And packaged asset resolution fails only when the bundle itself is incomplete

## Files
- Modify: `crates/app-host/src/ui_assets.rs`
- Modify: `crates/app-host/src/server.rs`
- Modify: `apps/desktop-shell/src-tauri/tauri.conf.json`
- Modify or add: packaging/build support files needed for bundling

## Verification
- `cargo test -p matrixclaw-app-host bundled_asset_packaging -- --exact`
