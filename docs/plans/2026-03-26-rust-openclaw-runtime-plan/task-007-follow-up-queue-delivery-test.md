# Task 007: [TEST] Follow-up queue delivery

**depends-on**: task-006-steering-queue-delivery-impl

## Description

Create a failing session-runtime test proving follow-up messages are deferred until the current run reaches a natural stop and are not injected into the active turn.

## Execution Context

**Task Number**: 007 of 019 (test)  
**Phase**: Session Runtime  
**Prerequisites**: Steering queue delivery and run-controller boundaries are already defined.

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

- Create: `crates/session-runtime/tests/follow_up_queue_delivery.rs`
- Modify: `crates/session-runtime/src/queue.rs`
- Modify: `crates/session-runtime/src/run_controller.rs`

## Steps

### Step 1: Verify Scenario

- Confirm follow-up messages are separate from steering messages and must start a new run after a stable stop point.

### Step 2: Create the failing Red test

- Simulate an active run, enqueue a follow-up message, and assert it does not appear in the current turn context.
- Assert that the follow-up triggers a new run only when the current run would otherwise end.
- Keep the failure semantic by checking for premature injection or failure to start the next run.

### Step 3: Lock the queue-kind contract

- Extend queue item typing and run-controller signatures only as needed for the test.
- Do not implement follow-up behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime follow_up_queue_delivery -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- One failing test covers deferred follow-up delivery.
- The failure demonstrates wrong timing rather than missing harness code.
- Queue types clearly distinguish steering from follow-up behavior.
