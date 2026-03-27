# Task 006: Write live tool execution test

**depends-on**: task-002-implement-session-backed-live-run-service-impl

## Description

Add a failing test proving the live provider-backed runtime executes tool calls through the same ordered lifecycle already defined in `agent-core`.

## Execution Context

**Task Number**: 006 of 009  
**Phase**: Core Runtime  
**Prerequisites**: session-backed run service exists

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

- Create: `crates/app-host/tests/live_tool_execution.rs`
- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `crates/app-host/src/local_command.rs`
- Modify: `crates/session-runtime/src/run_controller.rs`

## Steps

### Step 1: Define tool behavior at the live boundary
- Lock how the live runtime service receives tool-capable provider output and records lifecycle events.
- Use provider/tool doubles instead of live side effects.

### Step 2: Write the Red test
- Assert ordered tool lifecycle events.
- Assert tool results are written to persisted session state.
- Assert the assistant continuation occurs after tool results are available.

### Step 3: Keep the task Red-only
- Do not implement the live tool path yet.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host live_tool_execution -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The live tool loop is defined by a failing integration test.
- Tool/provider dependencies are isolated with doubles.
- The test fails for the intended missing live tool execution path.
