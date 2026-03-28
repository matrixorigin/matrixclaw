# Task 002: Implement tauri embedded runtime startup

**depends-on**: `task-002-write-tauri-embedded-runtime-startup-test-test`

## Goal
Replace the current redirect wrapper with a Tauri-owned startup path that feels like a desktop app and reaches the embedded runtime reliably.

## Scenario
Scenario: Tauri owns single-window startup and runtime attachment
  Given the packaged MatrixClaw app launches
  When the shell initializes the runtime and window
  Then the user lands in one app window
  And the shell can reach setup or workspace without an external browser redirect flow
  And native startup failure states are surfaced as app states rather than raw network errors

## Files
- Modify: `apps/desktop-shell/src/launcher.js`
- Modify: `apps/desktop-shell/src/index.html`
- Modify: `apps/desktop-shell/src-tauri/src/main.rs`
- Modify: `apps/desktop-shell/README.md`

## Verification
- `cargo test --manifest-path apps/desktop-shell/src-tauri/Cargo.toml tauri_embedded_runtime_startup -- --exact`
