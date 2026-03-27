# Task 004: Implement live queue integration

**depends-on**: task-004-write-live-queue-integration-test-test

## Description

Implement steering and follow-up delivery in the real live runtime path so queued messages affect the next live assistant turn correctly.

## Execution Context

**Task Number**: 004 of 009  
**Phase**: Core Runtime  
**Prerequisites**: failing live queue integration test exists

## BDD Scenario

```gherkin
Scenario: Live queue controls honor next-turn and next-run boundaries
  Given the assistant is currently processing a task
  When the user queues a steering message and a follow-up message
  Then MatrixClaw delivers the steering message before the next assistant turn begins
  And delivers the follow-up message only after the current run would otherwise stop
  And preserves the ordering of prior tool results
```

**Spec Source**: scope-specific scenario derived from `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/http/queue_api.rs`
- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `crates/session-runtime/src/queue.rs`
- Modify: `crates/session-runtime/src/run_controller.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the queue integration test still fails before implementation.

### Step 2: Implement the Green path
- Feed queued steering/follow-up items into the live runtime service using the same ordering semantics as the core session runtime.
- Ensure the browser/API path consumes the shared queue model instead of shadow state.

### Step 3: Verify
- Run the targeted queue integration test.
- Re-run session-runtime queue tests and all `app-host` tests.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host live_queue_integration -- --exact
cargo test -p matrixclaw-session-runtime steering_queue_delivery -- --exact
cargo test -p matrixclaw-session-runtime follow_up_queue_delivery -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The live runtime respects queue semantics already promised by the design.
- Browser queue controls now influence real runs.
- Existing queue tests remain green.
