# Task 003: Implement local setup server and shell routing

**depends-on**: task-003-write-local-setup-server-contract-test-test

## Description

Implement the local HTTP surface for first launch, including setup shell routing and the minimal route structure that later UI slices can build on.

## Execution Context

**Task Number**: 003 of 008  
**Phase**: Core Features  
**Prerequisites**: Red test exists for local setup server behavior

## BDD Scenario

```gherkin
Scenario: First launch opens setup without prior manual configuration
  Given MatrixClaw is installed
  And no configuration file exists
  When the user runs matrixclaw
  Then MatrixClaw starts a local setup experience
  And the user can configure provider, workspace, and auth settings
  And MatrixClaw writes the resulting configuration to its home directory
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/lib.rs`
- Modify: `crates/app-host/src/setup.rs`
- Modify: `crates/app-host/src/http/mod.rs`
- Modify: `crates/app-host/src/http/routes.rs`
- Create: `crates/app-host/src/http/setup_api.rs`
- Optionally Modify: `crates/app-host/src/ui_assets.rs`

## Steps

### Step 1: Re-run the failing test
- Confirm the setup server contract test still fails before implementation.

### Step 2: Implement the local setup surface
- Start or expose a loopback-only HTTP surface when config is missing.
- Route setup requests to the setup shell.
- Keep the server thin: it should orchestrate config persistence, not reimplement runtime logic in the UI layer.

### Step 3: Verify
- Confirm the targeted setup server test passes.
- Re-run all `app-host` tests to preserve install and asset behavior.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host local_setup_server -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- First launch exposes a setup surface instead of only silent bootstrap.
- The shell and API routes are cleanly separated.
- The app remains loopback-local and compatible with later Tauri wrapping.
