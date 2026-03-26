# Task 012: [TEST] OpenClaw agents list

**depends-on**: task-002-first-launch-setup-impl

## Description

Create a failing compatibility test proving an OpenClaw-oriented client can authenticate over the WebSocket boundary and request the list of available agents.

## Execution Context

**Task Number**: 012 of 019 (test)  
**Phase**: Protocol Compatibility  
**Prerequisites**: App-host startup, config, and loopback server settings exist.

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

- Create: `crates/compat-openclaw/Cargo.toml`
- Create: `crates/compat-openclaw/tests/list_agents_over_websocket.rs`
- Create: `crates/compat-openclaw/src/lib.rs`
- Create: `crates/compat-openclaw/src/websocket.rs`
- Create: `crates/compat-openclaw/src/capabilities.rs`

## Steps

### Step 1: Verify Scenario

- Confirm protocol compatibility claims are capability-based and fixture-driven.

### Step 2: Create the failing Red test

- Build a loopback WebSocket fixture that performs auth handshake and `agents.list`.
- Use a stub agent registry so the test is isolated from provider or runtime implementation details.
- Keep the failure semantic by checking for missing handshake frames, bad auth flow, or no agent list response.

### Step 3: Lock the compatibility contract

- Define only the compatibility capability descriptor, auth token interface, and agent-registry adapter needed by the test.
- Do not implement WebSocket behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-compat-openclaw list_agents_over_websocket -- --exact
cargo test -p matrixclaw-compat-openclaw
```

## Success Criteria

- One failing compatibility test covers handshake plus `agents.list`.
- The failure demonstrates unsupported compatibility behavior rather than missing network harness code.
- Protocol-facing types stay isolated from internal runtime types.
