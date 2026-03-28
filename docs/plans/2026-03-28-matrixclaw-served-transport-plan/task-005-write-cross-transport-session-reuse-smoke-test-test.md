# Task 005: Write cross-transport session reuse smoke test

**depends-on**: task-003-implement-openclaw-streaming-parity-impl, task-004-implement-normalized-ingress-envelope-contract-impl

## Description

Write a failing smoke test that proves browser and OpenClaw transports can resume the same persisted session through one runtime.

## Execution Context

**Task Number**: 005 of 005  
**Phase**: Cross-Transport Validation  
**Prerequisites**: served transports and ingress normalization are in place

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

- Create: `crates/app-host/tests/cross_transport_session_reuse.rs`
- Create or modify: `scripts/verify-served-transports.sh`

## Steps

### Step 1: Write the failing smoke test
- Exercise two different served transports against one session id.
- Assert shared persistence and continued context.

### Step 2: Confirm Red state
- Run the targeted smoke test and confirm the current product boundary does not yet prove this path end-to-end.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host cross_transport_session_reuse -- --exact
```

## Success Criteria

- A failing end-to-end smoke exists for cross-transport session reuse.
- The test exercises real served boundaries instead of only helper modules.

