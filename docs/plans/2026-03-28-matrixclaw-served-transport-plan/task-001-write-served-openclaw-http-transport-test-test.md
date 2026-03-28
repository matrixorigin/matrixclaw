# Task 001: Write served OpenClaw HTTP transport test

**depends-on**: none

## Description

Write a failing test that proves `app-host` serves an OpenClaw-compatible HTTP chat endpoint and routes it through the shared live runtime.

## Execution Context

**Task Number**: 001 of 005  
**Phase**: Served Transport Foundation  
**Prerequisites**: none

## BDD Scenario

```gherkin
Scenario: OpenClaw HTTP client reaches the shared live runtime through a served endpoint
  Given a running MatrixClaw loopback server
  And an OpenClaw-compatible HTTP request with a conversation id and user message
  When the client posts the request to the served OpenClaw endpoint
  Then MatrixClaw routes the request through the shared live runtime service
  And the compatibility-shaped response is returned from app-host
  And the conversation persists in the same session store used by the browser path
```

## Files to Modify/Create

- Create: `crates/app-host/tests/openclaw_http_over_server.rs`

## Steps

### Step 1: Write the failing server test
- Start the existing loopback test server.
- Issue a real HTTP request to the planned OpenClaw endpoint.
- Assert protocol-shaped response fields and shared session persistence.

### Step 2: Confirm Red state
- Run the targeted test and confirm it fails for the missing served endpoint.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host openclaw_http_over_server -- --exact
```

## Success Criteria

- A real failing test exists for served OpenClaw HTTP handling.
- The test exercises server routing rather than only library functions.

