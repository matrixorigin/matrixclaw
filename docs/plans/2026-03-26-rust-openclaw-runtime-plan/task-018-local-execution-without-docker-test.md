# Task 018: [TEST] Local execution without Docker

**depends-on**: task-005-tool-preflight-block-impl

## Description

Create a failing execution test proving MatrixClaw can run local command tools without Docker installed and without failing startup.

## Execution Context

**Task Number**: 018 of 019 (test)  
**Phase**: Operations  
**Prerequisites**: Tool execution contract and policy hooks already exist.

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

- Create: `crates/app-host/tests/local_execution_without_docker.rs`
- Create: `crates/app-host/src/execution.rs`
- Modify: `crates/agent-core/src/tool.rs`
- Modify: `crates/manifests/src/config.rs`

## Steps

### Step 1: Verify Scenario

- Confirm local execution is the default operator mode and Docker is not a startup prerequisite.

### Step 2: Create the failing Red test

- Use an execution backend double that simulates no Docker on the host.
- Assert that startup succeeds and local command execution selects the default local backend.
- Keep the failure semantic by checking for startup failure, backend mis-selection, or hard Docker dependency.

### Step 3: Lock the execution-backend contract

- Define execution mode enums, backend selection interfaces, and command-execution result types needed by the test.
- Do not implement backend selection in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host local_execution_without_docker -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- One failing test covers Docker-free local execution.
- The failure demonstrates wrong backend assumptions rather than a broken harness.
- Execution backend choice remains explicit and configurable.
