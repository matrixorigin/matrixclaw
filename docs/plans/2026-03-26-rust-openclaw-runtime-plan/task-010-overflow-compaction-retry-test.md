# Task 010: [TEST] Overflow compaction retry

**depends-on**: task-008-transcript-parity-impl

## Description

Create a failing runtime-policy test proving that provider overflow triggers compaction outside the core loop and retries from the compacted context.

## Execution Context

**Task Number**: 010 of 019 (test)  
**Phase**: Session Runtime  
**Prerequisites**: Transcript persistence and run-state storage are available.

## BDD Scenario

```gherkin
Scenario: Runtime compacts context before retrying an overflowed request
  Given a model request fails with context overflow
  When the session runtime handles the failure
  Then it removes the failure-only message from active context if needed
  And runs compaction outside the core loop
  And retries the run once from the compacted context
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/session-runtime/tests/compacts_before_retry_on_overflow.rs`
- Create: `crates/session-runtime/src/retry.rs`
- Create: `crates/session-runtime/src/compaction.rs`
- Modify: `crates/session-runtime/src/run_controller.rs`

## Steps

### Step 1: Verify Scenario

- Confirm overflow handling belongs to `session-runtime`, not `agent-core`.

### Step 2: Create the failing Red test

- Use a provider double that returns a classified overflow error on the first attempt.
- Assert that the runtime records compaction, rebuilds active context, and issues one continuation run.
- Keep the failure semantic by checking for loop-owned compaction, missing retry, or wrong active-context composition.

### Step 3: Lock retry and compaction contracts

- Define error taxonomy, compaction request types, and retry-policy signatures only as needed for the test.
- Do not implement retry or compaction behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime compacts_before_retry_on_overflow -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- One failing test covers overflow-triggered compaction and retry.
- The failure demonstrates missing runtime policy behavior rather than a provider stub issue.
- Retry and compaction responsibilities stay explicit at the session-runtime layer.
