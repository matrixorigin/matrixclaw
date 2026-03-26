# Task 003: [TEST] Single generation streaming

**depends-on**: task-001-install-without-privileged-writes-impl

## Description

Create a failing `agent-core` test that proves one provider generation powers both streamed output and the final assistant message, with no second probe completion.

## Execution Context

**Task Number**: 003 of 019 (test)  
**Phase**: Core Runtime  
**Prerequisites**: Cargo workspace exists and `matrixclaw-agent-core` crate skeleton can be added.

## BDD Scenario

```gherkin
Scenario: Final assistant answer is generated once
  Given an initialized session with no pending tool calls
  When the user sends a prompt
  Then MatrixClaw streams the assistant response from a single generation pass
  And the final persisted assistant message matches the streamed content exactly
  And the runtime does not perform a second completion just to enable streaming
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/agent-core/Cargo.toml`
- Create: `crates/agent-core/tests/final_answer_generated_once.rs`
- Create: `crates/agent-core/src/lib.rs`
- Create: `crates/agent-core/src/event.rs`
- Create: `crates/agent-core/src/provider.rs`

## Steps

### Step 1: Verify Scenario

- Confirm the no-double-generation requirement from the design docs and prior FastClaw review.

### Step 2: Create the failing Red test

- Build a provider test double that counts completion and streaming invocations.
- Write a test that drives one prompt through the loop, captures `message_delta` events, and inspects the finalized assistant message projection.
- Ensure the failure is semantic, such as duplicate provider calls or mismatched final content.

### Step 3: Lock the event and provider contract

- Define only the provider trait, event enum, and minimal run request types needed by the test.
- Do not implement loop behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-agent-core final_answer_generated_once -- --exact
cargo test -p matrixclaw-agent-core
```

## Success Criteria

- The scenario is represented by one failing `agent-core` test.
- The failure proves stream/final parity is missing rather than the harness being incomplete.
- Only explicit interfaces and test doubles are introduced.
