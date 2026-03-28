# Task 003: Implement Matrix streamed delivery

**depends-on**: task-003-write-matrix-streamed-delivery-test-test

## Description

Implement Matrix outbound delivery projection for streamed runtime events, including ordered incremental output and any gateway-local progress indicators.

## Execution Context

**Task Number**: 003 of 006  
**Phase**: Core Features  
**Prerequisites**: failing streamed delivery test exists

## BDD Scenario

```gherkin
Scenario: Matrix gateway streams assistant progress without changing runtime semantics
  Given the live runtime emits streamed assistant events for a gateway-driven run
  When the Matrix gateway projects those events back to the room
  Then the gateway sends incremental assistant output in order
  And the gateway can emit typing or progress updates without changing runtime event ordering
  And the final visible Matrix reply matches the persisted assistant completion
```

**Spec Source**: `../2026-03-28-matrix-gateway-design/bdd-specs.md`

## Files to Modify/Create

- Create or modify: `crates/app-host/src/gateway/delivery.rs`
- Modify: `crates/app-host/src/gateway/matrix.rs`
- Modify: `crates/app-host/tests/matrix_streamed_delivery.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the streamed delivery test still fails.

### Step 2: Implement Matrix delivery projection
- Translate streamed runtime events into Matrix-facing delivery operations.
- Keep delivery/progress behavior within the gateway layer.

### Step 3: Verify
- Run the targeted test.
- Re-run app-host tests related to streaming and transport delivery.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host matrix_streamed_delivery -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Streamed runtime output reaches Matrix in the correct order.
- Final Matrix-visible reply matches the persisted assistant completion.
