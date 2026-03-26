# Task 003: [IMPL] Single generation streaming

**depends-on**: task-003-single-generation-streaming-test

## Description

Implement the initial streaming-first run loop so one provider stream produces one finalized assistant message and one ordered event sequence.

## Execution Context

**Task Number**: 003 of 019 (impl)  
**Phase**: Core Runtime  
**Prerequisites**: The paired Red test exists and fails for duplicate generation or parity reasons.

## BDD Scenario

```gherkin
Scenario: Final assistant answer is generated once
  Given an initialized session with no pending tool calls
  When the user sends a prompt
  Then MatrixClaw streams the assistant response from a single generation pass
  And the final persisted assistant message matches the streamed content exactly
  And the runtime does not perform a second completion just to enable streaming
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/agent-core/src/lib.rs`
- Modify: `crates/agent-core/src/event.rs`
- Modify: `crates/agent-core/src/provider.rs`
- Create: `crates/agent-core/src/message.rs`
- Create: `crates/agent-core/src/loop.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the single-generation streaming test still fails before implementation.

### Step 2: Implement the minimal loop behavior

- Add the initial run loop entrypoint and internal message model.
- Stream assistant deltas from a single provider call into one finalized assistant message object.
- Emit deterministic start, delta, completion, and run-finished events from that same generation path.

### Step 3: Verify Pass

- Run the targeted streaming parity test and confirm it passes.

### Step 4: Regression sweep

- Re-run `matrixclaw-agent-core` tests to protect the event contract.

## Verification Commands

```bash
cargo test -p matrixclaw-agent-core final_answer_generated_once -- --exact
cargo test -p matrixclaw-agent-core
```

## Success Criteria

- The loop uses one provider generation for streaming and finalization.
- The targeted scenario passes.
- The event contract stays explicit and reusable by higher layers.
