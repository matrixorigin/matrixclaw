# Task 002: Write session-backed live run service test

**depends-on**: task-001-implement-provider-streaming-adapter-impl

## Description

Add a failing test for the shared live run service that will back browser and compatibility chat requests with one persisted session model.

## Execution Context

**Task Number**: 002 of 009  
**Phase**: Core Runtime  
**Prerequisites**: production provider adapter exists

## BDD Scenario

```gherkin
Scenario: Stored transcript matches visible behavior
  Given a user has completed a conversation with tool calls, retries, and warnings
  When MatrixClaw persists the session
  Then every user-visible assistant message is present in session storage
  And every tool result that influenced the assistant is present in session storage
  And terminal warning or failure messages are also persisted when shown to the user
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/session_backed_live_run_service.rs`
- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `crates/session-runtime/src/`
- Optionally Create: `crates/app-host/src/live_runtime.rs`

## Steps

### Step 1: Define the service contract
- Lock the request and response shape for creating or continuing a session-backed live run.
- Require a provider double so the test does not call a real network model.

### Step 2: Write the Red test
- Assert that the first prompt creates a session id.
- Assert that user-visible assistant output is both returned and persisted.
- Assert that runtime events needed for the browser transcript are available at the boundary.

### Step 3: Keep implementation out of the test task
- Do not implement the actual runtime service in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host session_backed_live_run_service -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The shared live run service is test-defined before implementation.
- Transcript durability is explicit at the service boundary.
- The test fails for the intended missing session-backed behavior.
