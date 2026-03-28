# Task 002: Implement Matrix ingress normalization

**depends-on**: task-002-write-matrix-ingress-normalization-test-test

## Description

Implement Matrix-specific inbound event normalization, room/thread to session mapping, and reply-routing preservation on top of the gateway contract.

## Execution Context

**Task Number**: 002 of 006  
**Phase**: Core Features  
**Prerequisites**: failing Matrix normalization test exists

## BDD Scenario

```gherkin
Scenario: Matrix room message resumes the mapped persisted session
  Given MatrixClaw has a stored mapping from a Matrix room and thread to a runtime session id
  When a new Matrix message arrives for that room and thread
  Then the Matrix gateway reuses the mapped session id
  And the shared live runtime continues the existing conversation
  And the Matrix adapter preserves room and thread routing for the reply path
```

**Spec Source**: `../2026-03-28-matrix-gateway-design/bdd-specs.md`

## Files to Modify/Create

- Create or modify: `crates/app-host/src/gateway/matrix.rs`
- Create or modify: `crates/app-host/src/gateway/store.rs`
- Modify: `crates/app-host/src/gateway/mod.rs`
- Modify: `crates/app-host/tests/matrix_ingress_normalization.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the Matrix normalization test still fails.

### Step 2: Implement Matrix mapping and normalization
- Add Matrix event parsing inputs, room/thread mapping lookup, and conversion into the existing ingress envelope.
- Keep Matrix field names contained to the gateway layer.

### Step 3: Verify
- Run the targeted test.
- Re-run app-host tests affected by ingress or session mapping changes.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host matrix_ingress_normalization -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Matrix events normalize into ingress without runtime-specific branching.
- Room/thread routing and persisted session reuse are preserved.
