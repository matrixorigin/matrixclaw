# Task 002: Write Matrix ingress normalization test

**depends-on**: task-001-implement-gateway-adapter-contract-impl

## Description

Write a failing test for Matrix room and thread events being normalized into ingress requests and mapped onto persisted runtime sessions.

## Execution Context

**Task Number**: 002 of 006  
**Phase**: Core Features  
**Prerequisites**: generic gateway contract exists

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

- Create: `crates/app-host/tests/matrix_ingress_normalization.rs`

## Steps

### Step 1: Write the failing normalization test
- Model Matrix room/thread input, expected session mapping reuse, and preserved reply routing metadata.
- Keep the test focused on normalization and mapping rather than delivery mechanics.

### Step 2: Confirm Red state
- Run the targeted test and confirm Matrix-specific normalization does not yet exist.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host matrix_ingress_normalization -- --exact
```

## Success Criteria

- A failing Matrix normalization test exists.
- Session mapping expectations are explicit before implementation.
