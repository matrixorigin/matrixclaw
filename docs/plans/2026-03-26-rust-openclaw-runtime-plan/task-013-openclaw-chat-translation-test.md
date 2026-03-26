# Task 013: [TEST] OpenClaw chat translation

**depends-on**: task-008-transcript-parity-impl, task-012-openclaw-agents-list-impl

## Description

Create a failing compatibility test proving an OpenClaw chat request is translated into internal session-runtime operations and streamed back out through compatibility frames without leaking protocol types into the core.

## Execution Context

**Task Number**: 013 of 019 (test)  
**Phase**: Protocol Compatibility  
**Prerequisites**: Transcript parity exists and the compatibility boundary can already authenticate and list agents.

## BDD Scenario

```gherkin
Scenario: OpenClaw-compatible chat request reaches the internal runtime
  Given an authenticated compatibility client
  When it sends a chat request through the compatibility boundary
  Then MatrixClaw translates that request into internal session-runtime messages
  And the core loop runs without awareness of the external protocol format
  And the resulting events are translated back into compatibility responses
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/compat-openclaw/tests/chat_request_translation.rs`
- Modify: `crates/compat-openclaw/src/translation.rs`
- Create: `crates/compat-openclaw/src/stream_adapter.rs`
- Modify: `crates/session-runtime/src/lib.rs`

## Steps

### Step 1: Verify Scenario

- Confirm protocol frames must be translated at the edge and not leak into `agent-core` or `session-runtime`.

### Step 2: Create the failing Red test

- Send a compatibility-shaped chat request into a loopback adapter using test doubles for session-runtime and event stream output.
- Assert on request translation, internal message shape, and compatibility stream framing.
- Keep the failure semantic by checking for missing translation or mismatched stream output rather than generic transport errors.

### Step 3: Lock the adapter contract

- Add only the translation and event-adapter interfaces needed by the test.
- Do not implement actual chat translation in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-compat-openclaw chat_request_translation -- --exact
cargo test -p matrixclaw-compat-openclaw
```

## Success Criteria

- One failing compatibility test covers request translation and stream projection.
- The failure demonstrates missing boundary behavior, not core runtime breakage.
- Internal runtime types remain protocol-agnostic.
