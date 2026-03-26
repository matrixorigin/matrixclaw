# Task 006: [TEST] Steering queue delivery

**depends-on**: task-004-tool-calls-extend-loop-impl

## Description

Create a failing session-runtime test that proves steering messages are queued during an active run and delivered before the next assistant turn without disturbing prior tool-result ordering.

## Execution Context

**Task Number**: 006 of 019 (test)  
**Phase**: Session Runtime  
**Prerequisites**: Core loop turn continuation and tool-result messages exist.

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

- Create: `crates/session-runtime/Cargo.toml`
- Create: `crates/session-runtime/tests/steering_queue_delivery.rs`
- Create: `crates/session-runtime/src/lib.rs`
- Create: `crates/session-runtime/src/queue.rs`
- Create: `crates/session-runtime/src/run_controller.rs`

## Steps

### Step 1: Verify Scenario

- Confirm steering messages must not splice into an in-flight assistant stream.

### Step 2: Create the failing Red test

- Use core-loop doubles to simulate a multi-turn run with tool results.
- Queue a steering message mid-run and assert on queue persistence, delivery point, and preserved result ordering.
- Make the failure semantic by checking wrong insertion timing or reordered messages.

### Step 3: Lock the queue contract

- Define queue item types, run-controller hooks, and persistence-facing interfaces needed by the test.
- Do not implement queue delivery in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime steering_queue_delivery -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- One failing session-runtime test covers steering queue delivery.
- The failure demonstrates incorrect delivery timing or ordering.
- Queue semantics are expressed through explicit contracts only.
