# Task 001: Implement served OpenClaw HTTP transport

**depends-on**: task-001-write-served-openclaw-http-transport-test-test

## Description

Expose a real OpenClaw-compatible HTTP endpoint from `app-host` that reuses the shared live runtime service.

## Execution Context

**Task Number**: 001 of 005  
**Phase**: Served Transport Foundation  
**Prerequisites**: failing served HTTP transport test exists

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

- Modify: `crates/app-host/src/server.rs`
- Modify: `crates/app-host/src/http/mod.rs`
- Create or modify: `crates/app-host/src/http/openclaw_api.rs`
- Modify: `crates/app-host/src/openclaw_transport.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the served HTTP transport test still fails before implementation.

### Step 2: Implement the served endpoint
- Parse OpenClaw HTTP payloads in `app-host`.
- Delegate into the existing shared OpenClaw transport adapter.
- Return protocol-shaped HTTP responses without creating a second runtime path.

### Step 3: Verify
- Run the targeted server test.
- Re-run the app-host test suite.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host openclaw_http_over_server -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- `app-host` serves a real OpenClaw HTTP endpoint.
- The served endpoint uses the shared live runtime service.
- Browser and OpenClaw persistence stay unified.

