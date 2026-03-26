# Task 013: [IMPL] OpenClaw chat translation

**depends-on**: task-013-openclaw-chat-translation-test

## Description

Implement the compatibility adapter that translates OpenClaw chat requests into internal session-runtime operations and maps internal events back into compatibility responses.

## Execution Context

**Task Number**: 013 of 019 (impl)  
**Phase**: Protocol Compatibility  
**Prerequisites**: The paired Red test fails because chat translation or stream projection is missing.

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

- Modify: `crates/compat-openclaw/src/translation.rs`
- Modify: `crates/compat-openclaw/src/stream_adapter.rs`
- Modify: `crates/compat-openclaw/src/websocket.rs`
- Create: `crates/compat-openclaw/src/http.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the chat-translation test still fails before implementation.

### Step 2: Implement minimal translation behavior

- Translate compatibility chat requests into native session-runtime commands.
- Project internal message and tool events back into compatibility frames from the same underlying run.
- Keep translation logic isolated to the compatibility crate.

### Step 3: Verify Pass

- Run the targeted chat-translation test and confirm it passes.

### Step 4: Regression sweep

- Re-run compatibility tests to protect handshake and chat flows together.

## Verification Commands

```bash
cargo test -p matrixclaw-compat-openclaw chat_request_translation -- --exact
cargo test -p matrixclaw-compat-openclaw
```

## Success Criteria

- Compatibility chat requests are translated at the boundary only.
- Streamed compatibility output reflects the same underlying run persisted by the runtime.
- The targeted scenario passes.
