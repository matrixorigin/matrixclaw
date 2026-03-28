# Task 001: Write bundled asset packaging test

**depends-on**: none

## Goal
Create a failing test proving the packaged product can resolve setup and workspace shell assets without relying on a source checkout or repo-relative `ui/build` path.

## Scenario
Scenario: Packaged app launches without repo-relative assets
  Given MatrixClaw is launched from a packaged desktop installation
  When the runtime resolves shell and static UI assets
  Then it serves bundled application assets without requiring the repository layout
  And a missing source checkout does not break `/setup` or `/workspace`
  And packaged asset resolution fails only when the bundle itself is incomplete

## Files
- Create or modify: `crates/app-host/tests/bundled_asset_packaging.rs`
- Expected future production files: `crates/app-host/src/ui_assets.rs`, `apps/desktop-shell/src-tauri/`

## Verification
- `cargo test -p matrixclaw-app-host bundled_asset_packaging -- --exact`
