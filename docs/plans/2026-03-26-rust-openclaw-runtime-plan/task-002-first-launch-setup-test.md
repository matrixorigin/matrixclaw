# Task 002: [TEST] First launch setup

**depends-on**: task-001-install-without-privileged-writes-impl

## Description

Create a failing first-run test proving that MatrixClaw starts a local setup flow when config is absent and persists the resulting configuration into the runtime home directory.

## Execution Context

**Task Number**: 002 of 019 (test)  
**Phase**: Setup  
**Prerequisites**: App-host binary, runtime home path, and installer conventions exist from Task 001.

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

- Create: `crates/app-host/tests/first_launch_setup.rs`
- Create: `crates/app-host/src/setup.rs`
- Create: `crates/manifests/src/config.rs`
- Modify: `crates/app-host/src/main.rs`

## Steps

### Step 1: Verify Scenario

- Confirm the first-run setup behavior and persistence expectations from the design spec.

### Step 2: Create the failing Red test

- Build a temporary runtime-home test harness with no config file present.
- Use test doubles for provider selection and auth token capture so no live services are contacted.
- Assert that launching `matrixclaw` enters setup mode and writes config data under `~/.matrixclaw/config/`.
- Keep the failure semantic by proving setup is skipped or config is not persisted.

### Step 3: Lock the config contract

- Add only the configuration struct and parsing signatures required for the failing test.
- Do not implement first-run setup behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host first_launch_setup -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- A single failing first-launch setup test exists.
- Test doubles isolate provider/auth concerns from external dependencies.
- The failure is about setup behavior, not missing filesystem scaffolding.
