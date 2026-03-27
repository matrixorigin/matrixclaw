# Task 004: Write live queue integration test

**depends-on**: task-002-implement-session-backed-live-run-service-impl

## Description

Add a failing test proving the live runtime consumes steering and follow-up queue items with the same semantics already required by the session runtime design.

## Execution Context

**Task Number**: 004 of 009  
**Phase**: Core Runtime  
**Prerequisites**: session-backed run service exists

## BDD Scenario

```gherkin
Scenario: Live queue controls honor next-turn and next-run boundaries
  Given the assistant is currently processing a task
  When the user queues a steering message and a follow-up message
  Then MatrixClaw delivers the steering message before the next assistant turn begins
  And delivers the follow-up message only after the current run would otherwise stop
  And preserves the ordering of prior tool results
```

**Spec Source**: scope-specific scenario derived from `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/live_queue_integration.rs`
- Modify: `crates/app-host/src/http/queue_api.rs`
- Modify: `crates/app-host/src/http/agent_api.rs`

## Steps

### Step 1: Define the live queue boundary
- Lock how queue state participates in a real live run request.
- Use provider doubles to avoid network variability.

### Step 2: Write the Red test
- Assert steering appears before the next assistant turn in a live run.
- Assert follow-up waits until the current run completes.
- Assert tool-result ordering is not corrupted by queue delivery.

### Step 3: Keep the task Red-only
- Do not implement the queue-aware live path yet.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host live_queue_integration -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Live queue semantics are defined before implementation.
- The test isolates provider behavior with doubles.
- The failure points to missing queue-aware runtime integration.
