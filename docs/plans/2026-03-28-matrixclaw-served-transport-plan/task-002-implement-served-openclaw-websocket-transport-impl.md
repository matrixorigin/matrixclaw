# Task 002: Implement served OpenClaw WebSocket transport

**depends-on**: task-002-write-served-openclaw-websocket-transport-test-test

## Description

Implement served OpenClaw-compatible conversation handling in `app-host` without creating a second runtime path.

## Execution Context

**Task Number**: 002 of 005  
**Phase**: Served Conversation Transport  
**Prerequisites**: failing served WebSocket transport test exists

## BDD Scenario

```gherkin
Scenario: OpenClaw WebSocket client reaches the shared live runtime through a served conversation transport
  Given a running MatrixClaw loopback server
  And an OpenClaw-compatible client that performs the expected capability and authentication flow
  When it opens a served conversation transport and sends a chat request
  Then MatrixClaw serves protocol frames from app-host
  And the chat request executes through the shared live runtime service
  And the resulting conversation persists in the same session store used by other transports
```

## Files to Modify/Create

- Modify: `crates/app-host/src/server.rs`
- Create or modify: `crates/app-host/src/http/openclaw_api.rs`
- Modify: `crates/app-host/src/openclaw_transport.rs`
- Modify: `crates/compat-openclaw/src/websocket.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the served conversation transport test still fails.

### Step 2: Implement the served conversation path
- Host the OpenClaw handshake/conversation flow in `app-host`.
- Keep `compat-openclaw` protocol-shaped and runtime-agnostic.
- Reuse the same runtime service and persistence model used by browser and HTTP transport.

### Step 3: Verify
- Run the targeted transport test.
- Re-run app-host and compat-openclaw tests.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host openclaw_websocket_over_server -- --exact
cargo test -p matrixclaw-app-host
cargo test -p matrixclaw-compat-openclaw
```

## Success Criteria

- `app-host` hosts a real OpenClaw conversation transport.
- WebSocket handling does not fork runtime semantics from HTTP/browser paths.

