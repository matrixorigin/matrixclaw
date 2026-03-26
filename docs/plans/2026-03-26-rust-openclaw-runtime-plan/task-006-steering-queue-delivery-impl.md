# Task 006: [IMPL] Steering queue delivery

**depends-on**: task-006-steering-queue-delivery-test

## Description

Implement steering-message persistence and delivery so queued operator guidance enters active context immediately before the next assistant turn.

## Execution Context

**Task Number**: 006 of 019 (impl)  
**Phase**: Session Runtime  
**Prerequisites**: The paired Red test exists and fails for steering insertion timing or ordering.

## BDD Scenario

```gherkin
Scenario: Steering message is delivered before the next assistant turn
  Given the assistant is currently processing a task with tool calls
  When the user queues a steering message
  Then MatrixClaw stores the steering message in the session runtime queue
  And delivers it before the next LLM turn begins
  And preserves the original ordering of prior tool results
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/session-runtime/src/queue.rs`
- Modify: `crates/session-runtime/src/run_controller.rs`
- Create: `crates/session-runtime/src/session.rs`
- Create: `crates/session-runtime/src/message_projection.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the steering delivery test still fails before implementation.

### Step 2: Implement minimal steering delivery

- Persist steering queue items at enqueue time.
- Deliver steering messages only at the boundary before the next assistant turn.
- Preserve prior tool-result ordering in the active context projection.

### Step 3: Verify Pass

- Run the targeted steering-queue test and confirm it passes.

### Step 4: Regression sweep

- Re-run session-runtime tests to ensure queue changes do not destabilize other runtime behavior.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime steering_queue_delivery -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- Steering messages persist and deliver at the correct turn boundary.
- Prior tool results keep their original relative ordering.
- The targeted scenario passes.
