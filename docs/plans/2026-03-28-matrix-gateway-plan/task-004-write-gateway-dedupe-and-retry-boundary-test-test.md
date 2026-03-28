# Task 004: Write gateway dedupe and retry boundary test

**depends-on**: task-001-implement-gateway-adapter-contract-impl, task-002-implement-matrix-ingress-normalization-impl

## Description

Write a failing test for gateway-owned dedupe and retry behavior so duplicate Matrix events and transient delivery failures do not create duplicate runtime turns.

## Execution Context

**Task Number**: 004 of 006  
**Phase**: Integration  
**Prerequisites**: gateway contract and Matrix normalization exist

## BDD Scenario

```gherkin
Scenario: Delivery retries and dedupe stay outside the runtime
  Given a Matrix event may be delivered more than once or a reply send may fail transiently
  When the Matrix gateway processes inbound or outbound traffic
  Then dedupe keys and retry state are stored in the gateway layer
  And the live runtime does not branch on Matrix delivery mechanics
  And duplicate gateway deliveries do not create duplicate runtime turns
```

**Spec Source**: `../2026-03-28-matrix-gateway-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/gateway_dedupe_retry_boundary.rs`

## Steps

### Step 1: Write the failing boundary test
- Model duplicate inbound event ids and transient outbound send failures.
- Assert that runtime invocation count and persisted transcript stay correct.

### Step 2: Confirm Red state
- Run the targeted test and confirm the gateway-local delivery boundary is not implemented yet.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host gateway_dedupe_retry_boundary -- --exact
```

## Success Criteria

- A failing test exists for gateway-local dedupe and retry behavior.
- The runtime remains isolated from Matrix delivery mechanics.
