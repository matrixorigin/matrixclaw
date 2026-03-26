# Task 019: [TEST] Optional sandbox mode

**depends-on**: task-018-local-execution-without-docker-impl

## Description

Create a failing execution-policy test proving sandbox mode is opt-in and, when enabled, routes command execution through the configured sandbox backend with structured results.

## Execution Context

**Task Number**: 019 of 019 (test)  
**Phase**: Operations  
**Prerequisites**: Default local execution mode already exists.

## BDD Scenario

```gherkin
Scenario: Optional sandbox mode is enabled explicitly
  Given the user enables sandboxed execution in configuration
  When a tool requires isolated command execution
  Then MatrixClaw routes that command through the configured sandbox backend
  And returns structured execution results to the agent loop
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/optional_sandbox_mode.rs`
- Modify: `crates/app-host/src/execution.rs`
- Modify: `crates/manifests/src/config.rs`
- Create: `crates/app-host/src/sandbox_backend.rs`

## Steps

### Step 1: Verify Scenario

- Confirm sandboxing is optional operator policy, not a mandatory install dependency.

### Step 2: Create the failing Red test

- Use a sandbox backend double and a config fixture that enables sandboxed execution.
- Assert that isolated commands route through the sandbox backend and return structured results.
- Keep the failure semantic by checking for local-backend fallback or missing structured results.

### Step 3: Lock the sandbox contract

- Define sandbox backend traits, execution-result types, and config wiring signatures needed by the test.
- Do not implement sandbox routing in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host optional_sandbox_mode -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- One failing test covers opt-in sandbox routing.
- The failure demonstrates missing policy routing rather than an absent sandbox implementation detail.
- Sandbox control remains explicit and backend-agnostic.
