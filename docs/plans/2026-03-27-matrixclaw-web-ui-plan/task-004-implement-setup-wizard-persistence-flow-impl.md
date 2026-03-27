# Task 004: Implement setup wizard persistence flow

**depends-on**: task-004-write-setup-wizard-persistence-test-test

## Description

Implement the setup wizard UI and backend submission flow so valid setup data persists the runtime config and transitions the operator out of first-launch mode.

## Execution Context

**Task Number**: 004 of 008  
**Phase**: Core Features  
**Prerequisites**: Red wizard persistence test exists

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

- Modify: `ui/src/routes/setup/+page.svelte`
- Modify: `ui/src/lib/setup/`
- Modify: `crates/app-host/src/http/setup_api.rs`
- Modify: `crates/app-host/src/setup.rs`
- Modify: `crates/manifests/src/config.rs`

## Steps

### Step 1: Re-run the failing test
- Confirm the setup wizard persistence test still fails before implementation.

### Step 2: Implement the Green path
- Build the setup form and submission flow.
- Validate inputs on the backend boundary.
- Persist config plus execution defaults atomically enough to avoid partial success semantics.

### Step 3: Verify and regress
- Run the focused test.
- Re-run `app-host` tests.
- Rebuild frontend assets if the project build requires it.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host setup_wizard_persists_config -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Completing the wizard persists the expected config.
- Invalid input is rejected cleanly.
- The setup experience transitions naturally toward the workspace surface.
