# Task 005: Implement optional Matrix gateway startup

**depends-on**: task-005-write-optional-matrix-gateway-startup-test-test, task-002-implement-matrix-ingress-normalization-impl, task-004-implement-gateway-dedupe-and-retry-boundary-impl

## Description

Add optional Matrix gateway configuration and startup wiring without changing the existing browser and served transport startup path.

## Execution Context

**Task Number**: 005 of 006  
**Phase**: Integration  
**Prerequisites**: failing startup test exists and core Matrix gateway pieces are implemented

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

- Modify: `crates/manifests/src/config.rs`
- Modify: `crates/app-host/src/lib.rs`
- Modify: `crates/app-host/src/server.rs`
- Modify: `crates/app-host/src/gateway/mod.rs`
- Modify: `crates/app-host/tests/optional_matrix_gateway_startup.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the startup/configuration test still fails.

### Step 2: Implement optional gateway settings and startup wiring
- Add explicit Matrix gateway configuration structures.
- Keep the gateway disabled when config is absent or incomplete.

### Step 3: Verify
- Run the targeted test.
- Re-run app-host and manifest tests impacted by config changes.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host optional_matrix_gateway_startup -- --exact
cargo test -p matrixclaw-app-host
cargo test -p matrixclaw-manifests
```

## Success Criteria

- Matrix gateway startup is opt-in only.
- Existing transports still start normally when Matrix is disabled.
