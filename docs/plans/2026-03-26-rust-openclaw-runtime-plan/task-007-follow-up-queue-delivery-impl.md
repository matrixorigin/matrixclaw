# Task 007: [IMPL] Follow-up queue delivery

**depends-on**: task-007-follow-up-queue-delivery-test

## Description

Implement follow-up queue semantics so deferred messages start a new run only after the current run completes naturally.

## Execution Context

**Task Number**: 007 of 019 (impl)  
**Phase**: Session Runtime  
**Prerequisites**: The paired Red test fails because follow-up messages are injected too early or never resumed.

## BDD Scenario

```gherkin
Scenario: Follow-up message is delivered only after the current run completes
  Given the assistant is currently processing a task
  When the user queues a follow-up message
  Then MatrixClaw does not inject it into the current turn
  And delivers it only after the agent would otherwise stop
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/session-runtime/src/queue.rs`
- Modify: `crates/session-runtime/src/run_controller.rs`
- Modify: `crates/session-runtime/src/session.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the follow-up queue test still fails before implementation.

### Step 2: Implement minimal follow-up behavior

- Persist follow-up queue items independently from steering messages.
- Hold follow-up delivery until the current run reaches a natural stop condition.
- Start a new run with the deferred message after that stop point, without mutating the finished run transcript.

### Step 3: Verify Pass

- Run the targeted follow-up queue test and confirm it passes.

### Step 4: Regression sweep

- Re-run session-runtime tests to protect steering and follow-up queue semantics together.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime follow_up_queue_delivery -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- Follow-up messages never appear in the active turn context.
- A new run begins only after the prior run would otherwise finish.
- The targeted scenario passes.
