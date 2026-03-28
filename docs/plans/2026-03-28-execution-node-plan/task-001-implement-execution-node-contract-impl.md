# Task 001: Implement execution node contract

**depends-on**: task-001-write-execution-node-contract-test-test

## Goal
Add the minimal Execution Node request/result contract and any supporting module structure needed for runtime-facing use.

## Files
- Create: `crates/app-host/src/node/mod.rs`
- Create or modify: `crates/app-host/src/node/execution.rs`
- Modify: `crates/app-host/src/lib.rs`

## Verification
- `cargo test -p matrixclaw-app-host execution_node_contract -- --exact`
