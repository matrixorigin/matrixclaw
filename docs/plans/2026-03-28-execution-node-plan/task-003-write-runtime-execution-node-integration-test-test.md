# Task 003: Write runtime execution node integration test

**depends-on**: task-001-implement-execution-node-contract-impl, task-002-implement-execution-node-routing-impl

## Goal
Create a failing test proving runtime tool execution reaches the host through the Execution Node instead of bypassing the Node layer.

## Scenario
Scenario: Tool-backed runtime execution reuses the Execution Node
  Given a runtime tool requires host command execution
  When the tool is executed during a live runtime turn
  Then the runtime reaches the host through the Execution Node boundary
  And the resulting structured output is preserved in runtime-visible results
  And existing Gateway behavior remains unchanged

## Files
- Create or modify: `crates/app-host/tests/runtime_execution_node_integration.rs`
- Expected future production files: `crates/app-host/src/live_runtime.rs`, `crates/app-host/src/node/execution.rs`

## Verification
- `cargo test -p matrixclaw-app-host runtime_execution_node_integration -- --exact`
