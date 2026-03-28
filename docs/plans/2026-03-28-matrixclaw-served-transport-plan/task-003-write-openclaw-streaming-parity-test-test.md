# Task 003: Write OpenClaw streaming parity test

**depends-on**: task-001-implement-served-openclaw-http-transport-impl, task-002-implement-served-openclaw-websocket-transport-impl

## Description

Write a failing test that proves OpenClaw transport emits runtime events progressively instead of buffering the full result until completion.

## Execution Context

**Task Number**: 003 of 005  
**Phase**: Streaming Parity  
**Prerequisites**: served OpenClaw HTTP and conversation transports exist

## BDD Scenario

```gherkin
Scenario: OpenClaw transport streams runtime events progressively without diverging from browser semantics
  Given a provider-backed live runtime that emits ordered assistant deltas and completion events
  When an OpenClaw-compatible client consumes the served transport
  Then it receives progressive assistant output before final completion
  And the event ordering matches the shared runtime event model
  And the final message is not duplicated at the end of the stream
```

## Files to Modify/Create

- Create: `crates/app-host/tests/openclaw_streaming_parity.rs`

## Steps

### Step 1: Write the failing parity test
- Use a scripted provider or transport double.
- Assert progressive frames and non-duplicated finalization.

### Step 2: Confirm Red state
- Run the targeted test and confirm the current OpenClaw serving path does not yet satisfy parity.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host openclaw_streaming_parity -- --exact
```

## Success Criteria

- A failing parity test exists for served OpenClaw streaming behavior.
- The test is grounded in real runtime event ordering.

