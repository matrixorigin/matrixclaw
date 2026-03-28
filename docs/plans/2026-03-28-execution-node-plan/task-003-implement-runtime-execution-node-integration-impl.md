# Task 003: Implement runtime execution node integration

**depends-on**: task-003-write-runtime-execution-node-integration-test-test

## Goal
Make the live runtime or tool execution path call the Execution Node through a stable boundary.

## Files
- Modify: `crates/app-host/src/live_runtime.rs`
- Modify as needed: `crates/app-host/src/node/execution.rs`
- Modify tests impacted by the runtime integration

## Verification
- `cargo test -p matrixclaw-app-host runtime_execution_node_integration -- --exact`
- `cargo test -p matrixclaw-app-host live_tool_execution -- --exact`
