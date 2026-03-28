# Task 004: Implement gateway dedupe and retry boundary

**depends-on**: task-004-write-gateway-dedupe-and-retry-boundary-test-test

## Description

Implement gateway-local dedupe and retry storage so Matrix delivery concerns stay outside the live runtime.

## Execution Context

**Task Number**: 004 of 006  
**Phase**: Integration  
**Prerequisites**: failing dedupe/retry boundary test exists

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

- Create or modify: `crates/app-host/src/gateway/runtime.rs`
- Create or modify: `crates/app-host/src/gateway/store.rs`
- Modify: `crates/app-host/src/gateway/matrix.rs`
- Modify: `crates/app-host/tests/gateway_dedupe_retry_boundary.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the dedupe/retry test still fails.

### Step 2: Implement gateway-local state handling
- Add dedupe and retry storage/logic in the gateway layer only.
- Keep runtime calls idempotent from the gateway perspective.

### Step 3: Verify
- Run the targeted test.
- Re-run app-host tests that exercise ingress and delivery paths.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host gateway_dedupe_retry_boundary -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Duplicate gateway events do not create duplicate runtime turns.
- Retry behavior is isolated to the gateway layer.
