# Task 002: Write execution node routing test

**depends-on**: task-001-implement-execution-node-contract-impl

## Goal
Create failing tests proving the Execution Node routes local, sandboxed, and denied execution through the Node boundary.

## Scenario
Scenario: Execution Node routes local, sandboxed, and denied execution
  Given execution policy may allow local execution, require sandboxing, or deny execution
  When the Execution Node handles a capability request
  Then it routes the request to the correct backend
  And it reports the backend used in the structured result
  And denied execution fails at the Node boundary rather than inside Gateway logic

## Files
- Create or modify: `crates/app-host/tests/execution_node_routing.rs`
- Expected future production files: `crates/app-host/src/node/execution.rs`

## Verification
- `cargo test -p matrixclaw-app-host execution_node_routing -- --exact`
