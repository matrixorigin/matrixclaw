# Task 001: Implement provider streaming adapter

**depends-on**: task-001-write-provider-streaming-adapter-test-test

## Description

Implement the real OpenAI-compatible provider adapter so the live runtime can receive ordered delta events and one normalized final assistant message from a single upstream call.

## Execution Context

**Task Number**: 001 of 009  
**Phase**: Foundation  
**Prerequisites**: failing provider streaming test exists

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

- Modify: `crates/app-host/src/openai_compatible.rs`
- Modify: `crates/app-host/Cargo.toml`
- Optionally Modify: `crates/agent-core/src/provider.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the provider streaming test still fails before implementation.

### Step 2: Implement the Green path
- Add a production-ready OpenAI-compatible adapter that can project upstream stream chunks into agent-core events.
- Normalize assistant output before it is returned to higher layers.
- Keep provider configuration outside the core loop.

### Step 3: Verify
- Run the targeted provider test.
- Re-run all `app-host` tests to ensure the new dependency path does not break the existing browser shell.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host openrouter_provider_streaming -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The adapter performs one logical generation per turn.
- Streamed and final messages match after normalization.
- The live provider boundary is reusable by later runtime services.
