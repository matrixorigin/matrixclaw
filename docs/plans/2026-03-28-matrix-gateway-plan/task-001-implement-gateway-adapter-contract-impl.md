# Task 001: Implement gateway adapter contract

**depends-on**: task-001-write-gateway-adapter-contract-test-test

## Description

Add the generic gateway adapter boundary and related type contracts so external IM connectors can normalize inbound events and project outbound deliveries without changing the runtime core.

## Execution Context

**Task Number**: 001 of 006  
**Phase**: Foundation  
**Prerequisites**: failing gateway contract test exists

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

- Create or modify: `crates/app-host/src/gateway/mod.rs`
- Create or modify: `crates/app-host/src/gateway/types.rs`
- Modify: `crates/app-host/src/lib.rs`
- Modify: `crates/app-host/tests/gateway_adapter_contract.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the contract test still fails for the expected reason.

### Step 2: Implement the shared gateway types
- Introduce gateway-facing inbound event, outbound delivery, adapter trait, and runtime handoff contracts.
- Keep function signatures and types explicit and runtime-neutral.

### Step 3: Verify
- Run the targeted test.
- Re-run app-host tests if shared modules moved.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host gateway_adapter_contract -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- External gateways have a stable contract to target.
- The runtime boundary remains ingress-based and transport-neutral.
