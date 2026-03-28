# Task 005: Implement cross-transport session reuse smoke harness

**depends-on**: task-005-write-cross-transport-session-reuse-smoke-test-test

## Description

Implement the final smoke harness and supporting served transport behavior so browser and OpenClaw can resume the same session through one persisted runtime.

## Execution Context

**Task Number**: 005 of 005  
**Phase**: Cross-Transport Validation  
**Prerequisites**: failing cross-transport smoke test exists

## BDD Scenario

```gherkin
Scenario: Browser and OpenClaw transports can resume the same persisted session
  Given a persisted live-runtime session created through one served transport
  When a second transport resumes the same session id
  Then the shared live runtime continues the conversation without losing prior context
  And both transports observe the same persisted history and queue semantics
  And the smoke harness proves the product boundary rather than only unit helpers
```

## Files to Modify/Create

- Modify: `crates/app-host/src/server.rs`
- Modify: `crates/app-host/src/openclaw_transport.rs`
- Modify: `crates/app-host/tests/cross_transport_session_reuse.rs`
- Create or modify: `scripts/verify-served-transports.sh`

## Steps

### Step 1: Re-run the Red smoke
- Confirm the cross-transport smoke still fails.

### Step 2: Implement the Green path
- Ensure browser and OpenClaw served transports can reopen the same persisted session id.
- Reuse one session store, one queue model, and one runtime service.
- Add a maintainer-facing harness that exercises the served transport boundary.

### Step 3: Verify
- Run the targeted smoke test.
- Re-run app-host tests and the harness.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host cross_transport_session_reuse -- --exact
cargo test -p matrixclaw-app-host
./scripts/verify-served-transports.sh
```

## Success Criteria

- Browser and OpenClaw served transports can resume the same session.
- The harness proves the real product boundary end-to-end.
