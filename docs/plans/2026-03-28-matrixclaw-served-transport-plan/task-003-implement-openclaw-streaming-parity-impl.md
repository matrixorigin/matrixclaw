# Task 003: Implement OpenClaw streaming parity

**depends-on**: task-003-write-openclaw-streaming-parity-test-test

## Description

Project shared runtime events progressively through the served OpenClaw transport so it behaves like the browser stream instead of a buffered compatibility shim.

## Execution Context

**Task Number**: 003 of 005  
**Phase**: Streaming Parity  
**Prerequisites**: failing streaming parity test exists

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

- Modify: `crates/app-host/src/openclaw_transport.rs`
- Modify: `crates/app-host/src/server.rs`
- Modify: `crates/compat-openclaw/src/stream_adapter.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the parity test still fails.

### Step 2: Implement progressive event projection
- Stream OpenClaw frames from live runtime events.
- Preserve ordering and avoid final-message duplication.
- Keep browser and OpenClaw transport semantics aligned.

### Step 3: Verify
- Run the targeted parity test.
- Re-run the app-host suite.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host openclaw_streaming_parity -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- OpenClaw transport emits progressive runtime output.
- Finalization semantics match the browser path.

