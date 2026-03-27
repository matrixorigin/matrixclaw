# Task 003: Write local setup server contract test

**depends-on**: task-002-implement-embedded-asset-pipeline-impl

## Description

Add a failing test proving `app-host` starts a local HTTP surface that exposes setup routes and serves the shell instead of silently writing config with no operator-visible flow.

## Execution Context

**Task Number**: 003 of 008  
**Phase**: Core Features  
**Prerequisites**: embedded asset serving exists

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

- Create: `crates/app-host/tests/local_setup_server.rs`
- Create: `crates/app-host/src/http/mod.rs`
- Create: `crates/app-host/src/http/routes.rs`
- Modify: `crates/app-host/src/lib.rs`
- Modify: `crates/app-host/src/setup.rs`

## Steps

### Step 1: Verify Scenario
- Confirm the core design expects a local setup experience, not only silent file writes.

### Step 2: Create the failing Red test
- Start `app-host` in a no-config temp home.
- Assert a setup route or setup-mode HTTP contract becomes available.
- Assert config is not treated as complete until the setup flow submits valid data.

### Step 3: Lock the server contract
- Define the minimum HTTP surface needed by the test:
  - setup shell route
  - setup config submission endpoint
  - health/status route if needed for verification

## Verification Commands

```bash
cargo test -p matrixclaw-app-host local_setup_server -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- A failing Red test proves the absence of the browser-first setup flow.
- The HTTP contract for setup mode is explicit.
- The failure is semantic and isolated.
