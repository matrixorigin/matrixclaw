# Task 006: Implement browser Matrix session reuse smoke harness

**depends-on**: task-006-write-browser-matrix-session-reuse-smoke-test-test

## Description

Implement the final browser/Matrix shared-session smoke harness and any supporting gateway wiring needed to prove the product boundary end-to-end.

## Execution Context

**Task Number**: 006 of 006  
**Phase**: Testing  
**Prerequisites**: failing browser/Matrix smoke test exists

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

- Modify: `crates/app-host/src/server.rs`
- Modify: `crates/app-host/src/gateway/matrix.rs`
- Modify: `crates/app-host/tests/browser_matrix_session_reuse.rs`
- Create or modify: `scripts/verify-matrix-gateway.sh`

## Steps

### Step 1: Re-run the Red smoke
- Confirm the browser/Matrix smoke still fails.

### Step 2: Implement the Green path
- Ensure a browser-created session can be resumed through the Matrix gateway mapping.
- Add a maintainer-facing harness that exercises the real browser and Matrix product boundaries.

### Step 3: Verify
- Run the targeted smoke test.
- Re-run app-host tests and the Matrix gateway harness.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host browser_matrix_session_reuse -- --exact
cargo test -p matrixclaw-app-host
./scripts/verify-matrix-gateway.sh
```

## Success Criteria

- Browser and Matrix gateway reuse the same persisted session model.
- The harness proves the real product boundary end-to-end.
