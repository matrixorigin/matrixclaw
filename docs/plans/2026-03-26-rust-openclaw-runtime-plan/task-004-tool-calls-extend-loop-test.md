# Task 004: [TEST] Tool calls extend loop

**depends-on**: task-003-single-generation-streaming-impl

## Description

Create a failing `agent-core` test showing that assistant tool calls are finalized, executed, and fed back into the next turn through ordered lifecycle events and tool-result messages.

## Execution Context

**Task Number**: 004 of 019 (test)  
**Phase**: Core Runtime  
**Prerequisites**: The base streaming loop exists and exposes ordered event hooks.

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

- Create: `crates/agent-core/tests/tool_calls_extend_turn_loop.rs`
- Create: `crates/agent-core/src/tool.rs`
- Modify: `crates/agent-core/src/message.rs`
- Modify: `crates/agent-core/src/loop.rs`

## Steps

### Step 1: Verify Scenario

- Confirm the loop must finalize assistant tool calls before execution and continue with tool results in context.

### Step 2: Create the failing Red test

- Use a provider double that emits a tool call on the first turn and an assistant completion on the second.
- Use a tool double that records preflight and execution order.
- Assert on event sequence, tool result insertion, and second-turn continuation.

### Step 3: Lock the tool contract

- Add only explicit tool-call, tool-result, and executor signatures needed for the failing test.
- Do not implement execution behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-agent-core tool_calls_extend_turn_loop -- --exact
cargo test -p matrixclaw-agent-core
```

## Success Criteria

- The scenario is covered by one failing core-loop test.
- Failure is about missing tool lifecycle behavior, not missing compile-time wiring.
- Tool interfaces stay isolated behind test doubles.
