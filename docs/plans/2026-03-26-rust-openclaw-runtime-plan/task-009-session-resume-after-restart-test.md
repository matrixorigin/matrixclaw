# Task 009: [TEST] Session resume after restart

**depends-on**: task-008-transcript-parity-impl

## Description

Create a failing persistence test proving session-runtime can reload stored message history and queue metadata after a restart and continue the conversation from that durable state.

## Execution Context

**Task Number**: 009 of 019 (test)  
**Phase**: Session Runtime  
**Prerequisites**: Durable transcript storage and runtime-home layout are already defined.

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

- Create: `crates/session-runtime/tests/session_resume_after_restart.rs`
- Modify: `crates/session-runtime/src/session.rs`
- Modify: `crates/session-runtime/src/sqlite.rs`
- Create: `crates/session-runtime/src/recovery.rs`

## Steps

### Step 1: Verify Scenario

- Confirm restart behavior requires message history plus queue metadata reconstruction, not just transcript reload.

### Step 2: Create the failing Red test

- Persist a session with messages and queued runtime metadata into a temporary database.
- Reconstruct a fresh runtime instance from disk and assert that the next prompt sees the persisted context.
- Keep the failure semantic by checking for missing queue state or lost transcript context.

### Step 3: Lock the recovery contract

- Add only session loader and recovery signatures needed by the test.
- Do not implement recovery behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime session_resume_after_restart -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- One failing restart-resume test exists.
- The failure demonstrates incomplete reconstruction of durable state.
- Recovery logic stays behind explicit session-runtime interfaces.
