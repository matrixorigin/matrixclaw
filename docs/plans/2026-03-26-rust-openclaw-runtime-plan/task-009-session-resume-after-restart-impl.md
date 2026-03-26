# Task 009: [IMPL] Session resume after restart

**depends-on**: task-009-session-resume-after-restart-test

## Description

Implement session-runtime recovery so restart reconstructs durable message history, queue metadata, and the next continuation context.

## Execution Context

**Task Number**: 009 of 019 (impl)  
**Phase**: Session Runtime  
**Prerequisites**: The paired Red test fails because restart recovery is incomplete.

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

- Modify: `crates/session-runtime/src/session.rs`
- Modify: `crates/session-runtime/src/sqlite.rs`
- Modify: `crates/session-runtime/src/recovery.rs`
- Create: `crates/session-runtime/src/context_builder.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm restart recovery still fails before implementation.

### Step 2: Implement minimal recovery behavior

- Load transcript messages and queue metadata from durable storage.
- Rebuild the continuation context from the persisted session state.
- Keep restart recovery explicit rather than trying to replay an interrupted provider stream byte-for-byte.

### Step 3: Verify Pass

- Run the targeted restart-resume test and confirm it passes.

### Step 4: Regression sweep

- Re-run session-runtime tests to protect transcript and queue behavior after restart.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime session_resume_after_restart -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- Restarted runtime instances can continue from durable session state.
- Queue metadata survives restart and remains usable.
- The targeted scenario passes.
