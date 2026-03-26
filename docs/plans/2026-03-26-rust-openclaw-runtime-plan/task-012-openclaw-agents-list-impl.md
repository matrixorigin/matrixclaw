# Task 012: [IMPL] OpenClaw agents list

**depends-on**: task-012-openclaw-agents-list-test

## Description

Implement the initial OpenClaw WebSocket handshake, capability descriptor, auth validation, and `agents.list` response path.

## Execution Context

**Task Number**: 012 of 019 (impl)  
**Phase**: Protocol Compatibility  
**Prerequisites**: The paired Red test fails because handshake or agent listing behavior is missing.

## BDD Scenario

```gherkin
Scenario: OpenClaw-compatible client lists agents
  Given MatrixClaw is running with compatibility mode enabled
  When a compatible client authenticates over the OpenClaw WebSocket boundary
  Then the client receives the expected connection challenge and response flow
  And the client can request the list of available agents
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/compat-openclaw/src/websocket.rs`
- Modify: `crates/compat-openclaw/src/capabilities.rs`
- Create: `crates/compat-openclaw/src/auth.rs`
- Create: `crates/compat-openclaw/src/translation.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the agents-list compatibility test still fails before implementation.

### Step 2: Implement minimal handshake and listing behavior

- Add loopback-safe WebSocket connection handling.
- Validate compatibility auth tokens and emit a capability descriptor.
- Translate `agents.list` into the internal agent-registry interface and return the compatibility-shaped response.

### Step 3: Verify Pass

- Run the targeted agents-list test and confirm it passes.

### Step 4: Regression sweep

- Re-run compatibility tests to protect the boundary contract.

## Verification Commands

```bash
cargo test -p matrixclaw-compat-openclaw list_agents_over_websocket -- --exact
cargo test -p matrixclaw-compat-openclaw
```

## Success Criteria

- Clients can authenticate and request the agent list over the compatibility boundary.
- Capability reporting and auth handling remain explicit and testable.
- The targeted scenario passes.
