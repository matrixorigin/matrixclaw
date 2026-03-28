# Task 002: Write served OpenClaw WebSocket transport test

**depends-on**: task-001-implement-served-openclaw-http-transport-impl

## Description

Write a failing test that proves `app-host` can host an OpenClaw-compatible WebSocket conversation boundary on top of the shared live runtime.

## Execution Context

**Task Number**: 002 of 005  
**Phase**: Served Conversation Transport  
**Prerequisites**: served OpenClaw HTTP transport is available

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

- Create: `crates/app-host/tests/openclaw_websocket_over_server.rs`

## Steps

### Step 1: Write the failing transport test
- Model the expected conversation flow at the server boundary.
- Assert protocol frames and shared session persistence.

### Step 2: Confirm Red state
- Run the targeted test and confirm it fails because the served conversation transport does not exist yet.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host openclaw_websocket_over_server -- --exact
```

## Success Criteria

- A failing test exists for served OpenClaw conversation handling.
- The test exercises the real host boundary rather than a pure helper.

