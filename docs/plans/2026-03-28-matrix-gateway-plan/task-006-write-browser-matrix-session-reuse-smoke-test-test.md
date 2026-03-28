# Task 006: Write browser Matrix session reuse smoke test

**depends-on**: task-003-implement-matrix-streamed-delivery-impl, task-004-implement-gateway-dedupe-and-retry-boundary-impl, task-005-implement-optional-matrix-gateway-startup-impl

## Description

Write a failing end-to-end smoke test proving that a browser-created session can be resumed through the Matrix gateway while preserving persisted history and queue semantics.

## Execution Context

**Task Number**: 006 of 006  
**Phase**: Testing  
**Prerequisites**: Matrix delivery, gateway state, and startup wiring are implemented

## BDD Scenario

```gherkin
Scenario: Browser and Matrix gateway share one persisted session model
  Given a conversation was started through the browser transport
  And that conversation has a persisted session id and queued runtime metadata
  When the mapped Matrix room resumes the same session
  Then the shared live runtime continues from the persisted browser state
  And steering and follow-up queue semantics remain correct
  And the Matrix reply reflects the same shared session history visible to browser users
```

**Spec Source**: `../2026-03-28-matrix-gateway-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/browser_matrix_session_reuse.rs`
- Create or modify: `scripts/verify-matrix-gateway.sh`

## Steps

### Step 1: Write the failing smoke test
- Exercise one browser path and one Matrix path against a shared persisted session id or room mapping.
- Assert persisted history and queue semantics across both transports.

### Step 2: Confirm Red state
- Run the targeted smoke test and confirm the product boundary is not yet proven end-to-end.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host browser_matrix_session_reuse -- --exact
```

## Success Criteria

- A failing end-to-end browser/Matrix smoke exists.
- The test exercises the real product boundary rather than only helper functions.
