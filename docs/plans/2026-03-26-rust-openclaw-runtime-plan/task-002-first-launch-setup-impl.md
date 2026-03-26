# Task 002: [IMPL] First launch setup

**depends-on**: task-002-first-launch-setup-test

## Description

Implement first-run setup detection, local setup flow startup, and config persistence for provider, workspace, and auth settings.

## Execution Context

**Task Number**: 002 of 019 (impl)  
**Phase**: Setup  
**Prerequisites**: The paired Red test exists and fails meaningfully.

## BDD Scenario

```gherkin
Scenario: First launch opens setup without prior manual configuration
  Given MatrixClaw is installed
  And no configuration file exists
  When the user runs `matrixclaw`
  Then MatrixClaw starts a local setup experience
  And the user can configure provider, workspace, and auth settings
  And MatrixClaw writes the resulting configuration to its home directory
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/main.rs`
- Modify: `crates/app-host/src/setup.rs`
- Modify: `crates/manifests/src/config.rs`
- Create: `crates/app-host/src/paths.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the first-launch setup test still fails before implementation.

### Step 2: Implement the setup path

- Detect missing config on startup.
- Implement a local setup mode that captures provider, workspace, and auth data through an operator-facing interface stub.
- Persist configuration using the runtime-home layout from the design docs.
- Keep the config schema aligned with the design doc and avoid hidden defaults outside the schema.

### Step 3: Verify Pass

- Run the targeted first-launch setup test and confirm it passes.

### Step 4: Regression sweep

- Re-run the app-host package tests to confirm startup behavior remains stable.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host first_launch_setup -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- First launch enters setup mode when config is absent.
- Configuration persists in the documented runtime directory.
- The targeted scenario passes without introducing external setup dependencies.
