# Task 002: Implement session-backed live run service

**depends-on**: task-002-write-session-backed-live-run-service-test-test

## Description

Implement the shared live run service so browser and compatibility entrypoints execute through one session-backed runtime rather than direct provider calls.

## Execution Context

**Task Number**: 002 of 009  
**Phase**: Core Runtime  
**Prerequisites**: failing live run service test exists

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

- Modify: `crates/app-host/src/http/agent_api.rs`
- Create or Modify: `crates/app-host/src/live_runtime.rs`
- Modify: `crates/app-host/src/server.rs`
- Modify: `crates/session-runtime/src/session.rs`
- Modify: `crates/session-runtime/src/storage.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the new live run service test still fails before implementation.

### Step 2: Implement the shared runtime service
- Introduce one service boundary that accepts a prompt, selects provider/tool policy, and persists transcript state.
- Return stable session identifiers and ordered runtime events.
- Allow the loopback HTTP server to support the long-lived behavior this service needs.

### Step 3: Verify
- Run the targeted service test.
- Re-run all `app-host` tests and any affected `session-runtime` tests.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host session_backed_live_run_service -- --exact
cargo test -p matrixclaw-session-runtime
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Browser and compatibility paths can share one runtime service.
- Persisted transcript state matches visible assistant output.
- The direct provider-call shortcut is no longer the only live path.
