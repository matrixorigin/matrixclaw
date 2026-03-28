# Task 001: Write gateway adapter contract test

## Description

Write a failing contract test for a generic gateway boundary that receives external channel events, normalizes them into ingress requests, and keeps channel-specific delivery concerns outside the live runtime.

## Execution Context

**Task Number**: 001 of 006  
**Phase**: Foundation  
**Prerequisites**: none

## BDD Scenario

```gherkin
Scenario: External channel event is normalized before entering the runtime
  Given an inbound message arrives from an external IM gateway
  And the gateway carries sender, channel, thread, and reply-routing metadata
  When MatrixClaw accepts that event
  Then the gateway adapter converts it into a transport-neutral ingress envelope
  And the live runtime consumes only the normalized envelope
  And gateway-specific delivery metadata remains outside the runtime core
```

**Spec Source**: `../2026-03-28-matrix-gateway-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/gateway_adapter_contract.rs`

## Steps

### Step 1: Write the failing contract test
- Define the expected gateway-facing contract for inbound messages, outbound delivery routing, and normalized ingress handoff.
- Assert that the runtime-facing shape is transport-neutral.

### Step 2: Confirm Red state
- Run the targeted contract test and confirm the gateway abstraction does not yet exist.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host gateway_adapter_contract -- --exact
```

## Success Criteria

- A failing gateway contract test exists.
- The test makes the runtime/gateway boundary explicit.
