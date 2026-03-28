# Task 004: Write execution node smoke harness test

**depends-on**: task-002-implement-execution-node-routing-impl, task-003-implement-runtime-execution-node-integration-impl

## Goal
Create a failing smoke verification target that proves Execution Node behavior end-to-end and codifies the pattern for future sibling Nodes.

## Scenario
Scenario: Execution Node establishes the pattern for future Nodes
  Given the Execution Node is the first concrete Node slice
  When maintainers verify the milestone
  Then focused tests and a smoke harness prove the Node boundary works end-to-end
  And future Screenshot, Browser, Camera, Mouse, and Filesystem Nodes can follow the same layering

## Files
- Create or modify: `scripts/verify-execution-node.sh`
- Create or modify any targeted smoke test needed under `crates/app-host/tests/`

## Verification
- `./scripts/verify-execution-node.sh`
