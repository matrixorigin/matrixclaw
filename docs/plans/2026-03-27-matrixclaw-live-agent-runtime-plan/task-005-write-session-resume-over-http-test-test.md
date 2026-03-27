# Task 005: Write session resume over HTTP test

**depends-on**: task-002-implement-session-backed-live-run-service-impl

## Description

Add a failing test proving the browser/API path can continue a prior session after restart instead of acting like each prompt is stateless.

## Execution Context

**Task Number**: 005 of 009  
**Phase**: Persistence  
**Prerequisites**: session-backed run service exists

## BDD Scenario

```gherkin
Scenario: Session resumes after restart
  Given an existing persisted session
  When MatrixClaw restarts
  Then the session runtime reloads the prior message history
  And the next prompt continues from the persisted state
  And the runtime can reconstruct queued metadata needed for further processing
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/session_resume_over_http.rs`
- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `crates/session-runtime/src/storage.rs`
- Modify: `crates/session-runtime/src/session.rs`

## Steps

### Step 1: Define continuation semantics
- Lock how a request identifies an existing session.
- Ensure the test simulates restart/reload without a live network provider.

### Step 2: Write the Red test
- Create an initial session through the live runtime service.
- Simulate restart or service reconstruction.
- Assert the next prompt continues from persisted history and queue metadata.

### Step 3: Keep the task Red-only
- Do not implement session continuation yet.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host session_resume_over_http -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Session continuation is defined at the product boundary before implementation.
- Restart/reload behavior is explicit and fixture-backed.
- The test fails for the intended missing resume behavior.
