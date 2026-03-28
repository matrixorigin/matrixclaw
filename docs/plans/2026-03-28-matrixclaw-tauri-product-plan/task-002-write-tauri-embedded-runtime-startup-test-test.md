# Task 002: Write tauri embedded runtime startup test

**depends-on**: `task-001-implement-bundled-asset-packaging-impl`

## Goal
Create a failing test proving the Tauri app owns single-window startup and can attach or launch the embedded runtime product path without browser-style redirect scaffolding.

## Scenario
Scenario: Tauri owns single-window startup and runtime attachment
  Given the packaged MatrixClaw app launches
  When the shell initializes the runtime and window
  Then the user lands in one app window
  And the shell can reach setup or workspace without an external browser redirect flow
  And native startup failure states are surfaced as app states rather than raw network errors

## Files
- Create or modify: `apps/desktop-shell/src-tauri/` tests or harnesses
- Expected future production files: `apps/desktop-shell/src/launcher.js`, `apps/desktop-shell/src-tauri/src/main.rs`

## Verification
- `cargo test --manifest-path apps/desktop-shell/src-tauri/Cargo.toml tauri_embedded_runtime_startup -- --exact`
