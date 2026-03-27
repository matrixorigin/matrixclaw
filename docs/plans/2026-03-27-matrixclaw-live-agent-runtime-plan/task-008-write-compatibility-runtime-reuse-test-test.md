# Task 008: Write compatibility runtime reuse test

**depends-on**: task-002-implement-session-backed-live-run-service-impl

## Description

Add a failing test proving the OpenClaw transport adapter reuses the same live runtime service rather than keeping a separate execution path.

## Execution Context

**Task Number**: 008 of 009  
**Phase**: Boundary Integration  
**Prerequisites**: session-backed run service exists

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

- Create: `crates/compat-openclaw/tests/compat_runtime_reuse.rs`
- Modify: `crates/compat-openclaw/src/translation.rs`
- Modify: `crates/session-runtime/src/lib.rs`
- Optionally Create: `crates/app-host/src/live_runtime.rs`

## Steps

### Step 1: Define the shared adapter boundary
- Lock how compatibility code receives a runtime handle or service interface.
- Avoid a live provider or live network in this test.

### Step 2: Write the Red test
- Assert the compatibility request is routed into the same internal runtime abstraction as the browser path.
- Assert the translated output remains protocol-shaped without leaking browser-specific types.

### Step 3: Keep the task Red-only
- Do not implement boundary reuse yet.

## Verification Commands

```bash
cargo test -p matrixclaw-compat-openclaw compat_runtime_reuse -- --exact
cargo test -p matrixclaw-compat-openclaw
```

## Success Criteria

- Shared runtime reuse is test-defined at the OpenClaw transport adapter.
- The test fails for the intended duplicated-path problem.
- Protocol translation remains decoupled from product-specific transport details.
