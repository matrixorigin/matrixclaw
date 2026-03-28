# Task 005: Write optional Matrix gateway startup test

## Description

Write a failing test for Matrix gateway configuration and startup behavior so the connector stays disabled unless explicitly configured.

## Execution Context

**Task Number**: 005 of 006  
**Phase**: Foundation  
**Prerequisites**: none

## BDD Scenario

```gherkin
Scenario: Matrix gateway remains disabled without explicit configuration
  Given MatrixClaw is installed with no Matrix gateway credentials or homeserver settings
  When MatrixClaw starts
  Then the core browser and served transports still start normally
  And the Matrix gateway runner stays disabled
  And configuration clearly reports that the gateway is optional and not active
```

**Spec Source**: `../2026-03-28-matrix-gateway-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/optional_matrix_gateway_startup.rs`

## Steps

### Step 1: Write the failing startup test
- Assert disabled-by-default startup and explicit configuration requirements.
- Keep the test isolated from real Matrix network calls by using doubles or fixture settings.

### Step 2: Confirm Red state
- Run the targeted test and confirm Matrix gateway startup wiring does not yet exist.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host optional_matrix_gateway_startup -- --exact
```

## Success Criteria

- A failing startup/configuration test exists.
- Disabled-by-default behavior is explicit.
