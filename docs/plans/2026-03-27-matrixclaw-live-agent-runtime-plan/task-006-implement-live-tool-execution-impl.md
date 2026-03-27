# Task 006: Implement live tool execution

**depends-on**: task-006-write-live-tool-execution-test-test

## Description

Implement tool execution in the live runtime service so browser/API/compat runs can execute tools and continue the assistant turn with persisted results.

## Execution Context

**Task Number**: 006 of 009  
**Phase**: Core Runtime  
**Prerequisites**: failing live tool execution test exists

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

- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `crates/app-host/src/live_runtime.rs`
- Modify: `crates/app-host/src/local_command.rs`
- Modify: `crates/session-runtime/src/run_controller.rs`
- Optionally Modify: `crates/agent-core/src/tool.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the live tool execution test still fails before implementation.

### Step 2: Implement the Green path
- Connect the live runtime service to the existing tool execution loop.
- Persist tool lifecycle and result messages in the session.
- Keep tool execution explicit and structured for later UI rendering.

### Step 3: Verify
- Run the targeted tool execution test.
- Re-run relevant agent-core and app-host test suites.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host live_tool_execution -- --exact
cargo test -p matrixclaw-agent-core tool_calls_extend_turn_loop -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The live runtime executes tools through the same core loop semantics as the isolated agent-core tests.
- Tool results are durable and ordered.
- Later safety work can build on one live tool path instead of multiple implementations.
