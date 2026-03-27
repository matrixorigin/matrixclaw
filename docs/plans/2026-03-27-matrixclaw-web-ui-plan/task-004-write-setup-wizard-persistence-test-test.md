# Task 004: Write setup wizard persistence test

**depends-on**: task-003-implement-local-setup-server-and-shell-routing-impl

## Description

Add a failing test proving setup submissions validate and persist provider, workspace, auth, and execution defaults through the browser-first setup flow.

## Execution Context

**Task Number**: 004 of 008  
**Phase**: Core Features  
**Prerequisites**: local setup server and route structure exist

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

- Create: `crates/app-host/tests/setup_wizard_persists_config.rs`
- Create: `ui/src/routes/setup/+page.svelte`
- Create: `ui/src/lib/setup/`
- Modify: `crates/app-host/src/http/setup_api.rs`
- Modify: `crates/manifests/src/config.rs`

## Steps

### Step 1: Verify Scenario
- Confirm the wizard must cover provider, workspace, auth, and execution defaults.

### Step 2: Create the failing Red test
- Add an integration test that submits setup data through the local HTTP contract.
- Assert config and execution defaults are persisted only after valid input.
- Assert invalid input produces a structured error, not partial writes.

### Step 3: Lock request/response contracts
- Define setup payload and validation response shapes.
- Do not implement persistence or UI logic in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host setup_wizard_persists_config -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- A failing Red test captures end-to-end wizard persistence expectations.
- Validation contracts are explicit.
- Partial-write behavior is guarded by the test.
