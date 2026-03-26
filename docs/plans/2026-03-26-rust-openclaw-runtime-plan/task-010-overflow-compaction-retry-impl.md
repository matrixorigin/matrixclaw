# Task 010: [IMPL] Overflow compaction retry

**depends-on**: task-010-overflow-compaction-retry-test

## Description

Implement overflow classification, compaction orchestration, and one-step retry handling in `session-runtime`.

## Execution Context

**Task Number**: 010 of 019 (impl)  
**Phase**: Session Runtime  
**Prerequisites**: The paired Red test fails because overflow handling is absent or owned by the wrong layer.

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

- Modify: `crates/session-runtime/src/retry.rs`
- Modify: `crates/session-runtime/src/compaction.rs`
- Modify: `crates/session-runtime/src/run_controller.rs`
- Create: `crates/session-runtime/src/error.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the overflow retry test still fails before implementation.

### Step 2: Implement minimal overflow policy

- Classify provider overflow errors.
- Remove failure-only context artifacts where the design requires it.
- Invoke compaction as a session-runtime policy step, then issue one continuation run from the compacted context.

### Step 3: Verify Pass

- Run the targeted overflow-compaction-retry test and confirm it passes.

### Step 4: Regression sweep

- Re-run session-runtime tests to protect persistence, queueing, and retry behavior together.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime compacts_before_retry_on_overflow -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- Overflow is handled through runtime policy rather than loop internals.
- Exactly one retry occurs from the compacted context.
- The targeted scenario passes.
