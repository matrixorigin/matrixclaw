# Task 001: Write provider streaming adapter test

## Description

Add a failing Red test that defines the real provider adapter boundary for a streamed assistant turn without relying on a live network.

## Execution Context

**Task Number**: 001 of 009  
**Phase**: Foundation  
**Prerequisites**: none

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

- Create: `crates/app-host/tests/openrouter_provider_streaming.rs`
- Modify: `crates/app-host/src/openai_compatible.rs`
- Optionally Modify: `crates/agent-core/src/provider.rs`

## Steps

### Step 1: Lock the adapter contract
- Define how a production provider adapter emits `RunStarted`, delta events, and final completion from one upstream model invocation.
- Require test doubles or fixture HTTP payloads instead of real network access.

### Step 2: Write the Red test
- Use a local fixture server or fixture body reader to simulate an OpenAI-compatible streamed response.
- Assert that one logical provider turn yields ordered runtime events and one final assistant message.
- Assert that the adapter does not require a second completion call to obtain streaming semantics.

### Step 3: Keep production behavior out of the test task
- Do not implement the real adapter logic in this task.
- Do not depend on a live OpenRouter key.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host openrouter_provider_streaming -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The provider boundary is defined by a failing isolated test.
- Network behavior is fixture-backed and deterministic.
- The test fails for the intended missing streaming behavior.
