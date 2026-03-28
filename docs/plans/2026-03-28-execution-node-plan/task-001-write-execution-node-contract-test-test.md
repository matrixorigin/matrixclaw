# Task 001: Write execution node contract test

**depends-on**: none

## Goal
Create a failing test proving the runtime can target execution through a dedicated Node contract instead of directly calling backend helpers.

## Scenario
Scenario: Runtime reaches execution through a Node boundary
  Given the runtime needs to execute a host command
  When it issues a request through the Execution Node contract
  Then the request is represented as a Node-specific capability request
  And the Node returns a structured capability result
  And the runtime does not need to know local or sandbox backend implementation details

## Files
- Create or modify: `crates/app-host/tests/execution_node_contract.rs`
- Expected future production files: `crates/app-host/src/node/`

## Verification
- `cargo test -p matrixclaw-app-host execution_node_contract -- --exact`
