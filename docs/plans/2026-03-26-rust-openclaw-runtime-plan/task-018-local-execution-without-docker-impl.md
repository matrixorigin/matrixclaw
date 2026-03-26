# Task 018: [IMPL] Local execution without Docker

**depends-on**: task-018-local-execution-without-docker-test

## Description

Implement default local execution backend selection so command tools work without Docker and startup remains unaffected.

## Execution Context

**Task Number**: 018 of 019 (impl)  
**Phase**: Operations  
**Prerequisites**: The paired Red test fails because execution backend selection assumes Docker.

## BDD Scenario

```gherkin
Scenario: Safe local execution works without Docker
  Given the user has not installed Docker
  When the assistant uses local command execution
  Then MatrixClaw uses the default local execution mode
  And the runtime remains functional without failing startup
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/execution.rs`
- Modify: `crates/manifests/src/config.rs`
- Modify: `crates/app-host/src/main.rs`
- Create: `crates/app-host/src/local_command.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the local-execution test still fails before implementation.

### Step 2: Implement minimal backend selection

- Make `local` the default execution mode.
- Ensure startup succeeds even when Docker is absent.
- Route local command execution through the default backend and return structured results.

### Step 3: Verify Pass

- Run the targeted local-execution test and confirm it passes.

### Step 4: Regression sweep

- Re-run app-host tests to protect setup, asset, and execution behavior together.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host local_execution_without_docker -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- MatrixClaw starts and executes local commands without Docker installed.
- Execution mode defaults are explicit in config and code.
- The targeted scenario passes.
