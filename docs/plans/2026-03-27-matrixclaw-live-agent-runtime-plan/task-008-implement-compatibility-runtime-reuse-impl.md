# Task 008: Implement compatibility runtime reuse

**depends-on**: task-008-write-compatibility-runtime-reuse-test-test

## Description

Implement the OpenClaw transport adapter on top of the same live runtime service used by the browser path.

## Execution Context

**Task Number**: 008 of 009  
**Phase**: Boundary Integration  
**Prerequisites**: failing compatibility reuse test exists

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
- Modify: `crates/compat-openclaw/src/http.rs`
- Modify: `crates/compat-openclaw/src/websocket.rs`
- Modify: `crates/app-host/src/live_runtime.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the compatibility reuse test still fails before implementation.

### Step 2: Implement the Green path
- Inject the shared runtime service into the OpenClaw transport adapter.
- Keep protocol translation isolated from the runtime core.
- Ensure compatibility frames project the same underlying runtime events used by the browser.

### Step 3: Verify
- Run the targeted compatibility test.
- Re-run the existing compat-openclaw test suite.

## Verification Commands

```bash
cargo test -p matrixclaw-compat-openclaw compat_runtime_reuse -- --exact
cargo test -p matrixclaw-compat-openclaw
```

## Success Criteria

- Browser and OpenClaw transport entrypoints share one runtime service.
- Translation logic stays boundary-only.
- Existing compatibility tests remain green.
