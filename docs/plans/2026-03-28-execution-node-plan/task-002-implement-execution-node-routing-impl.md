# Task 002: Implement execution node routing

**depends-on**: task-002-write-execution-node-routing-test-test

## Goal
Route local, sandboxed, and denied execution through the Execution Node using the current execution helpers.

## Files
- Modify: `crates/app-host/src/node/execution.rs`
- Modify as needed: `crates/app-host/src/execution.rs`
- Modify as needed: `crates/app-host/src/local_command.rs`
- Modify as needed: `crates/app-host/src/sandbox_backend.rs`

## Verification
- `cargo test -p matrixclaw-app-host execution_node_routing -- --exact`
