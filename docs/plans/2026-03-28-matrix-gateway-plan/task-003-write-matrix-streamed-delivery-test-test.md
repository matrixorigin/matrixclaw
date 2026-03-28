# Task 003: Write Matrix streamed delivery test

**depends-on**: task-001-implement-gateway-adapter-contract-impl, task-002-implement-matrix-ingress-normalization-impl

## Description

Write a failing test for projecting streamed runtime events into ordered Matrix room deliveries, including incremental assistant output and optional progress updates.

## Execution Context

**Task Number**: 003 of 006  
**Phase**: Core Features  
**Prerequisites**: gateway contract and Matrix normalization exist

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

- Create: `crates/app-host/tests/matrix_streamed_delivery.rs`

## Steps

### Step 1: Write the failing delivery test
- Model streamed runtime events and expected Matrix-facing deliveries.
- Include assertions around ordering and final reply equivalence.

### Step 2: Confirm Red state
- Run the targeted test and confirm streamed Matrix delivery projection does not yet exist.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host matrix_streamed_delivery -- --exact
```

## Success Criteria

- A failing streamed delivery test exists.
- Delivery ordering and final-reply guarantees are explicit.
