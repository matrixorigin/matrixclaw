# Task 004: [IMPL] Tool calls extend loop

**depends-on**: task-004-tool-calls-extend-loop-test

## Description

Implement tool-call finalization, tool execution lifecycle events, tool-result message creation, and loop continuation behavior in `agent-core`.

## Execution Context

**Task Number**: 004 of 019 (impl)  
**Phase**: Core Runtime  
**Prerequisites**: The paired Red test fails because tool lifecycle behavior is missing.

## BDD Scenario

```gherkin
Scenario: Tool calls extend the turn loop
  Given the model responds with one or more tool calls
  When MatrixClaw validates and executes those tool calls
  Then tool execution lifecycle events are emitted in order
  And tool result messages are appended to the session
  And the loop continues with the assistant using those tool results
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/agent-core/src/loop.rs`
- Modify: `crates/agent-core/src/message.rs`
- Modify: `crates/agent-core/src/tool.rs`
- Modify: `crates/agent-core/src/event.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the tool-loop test still fails before implementation.

### Step 2: Implement minimal tool-turn behavior

- Finalize the assistant message before tool preflight begins.
- Execute tool calls through the tool contract and emit ordered lifecycle events.
- Create structured tool-result messages and continue into the next assistant turn with those results in context.

### Step 3: Verify Pass

- Run the targeted tool-loop test and confirm it passes.

### Step 4: Regression sweep

- Re-run the core crate tests to protect single-generation and event-order guarantees.

## Verification Commands

```bash
cargo test -p matrixclaw-agent-core tool_calls_extend_turn_loop -- --exact
cargo test -p matrixclaw-agent-core
```

## Success Criteria

- Tool calls extend the loop through explicit lifecycle events and structured result messages.
- Assistant continuation uses tool results from the prior turn.
- The targeted scenario passes without duplicating provider generations.
