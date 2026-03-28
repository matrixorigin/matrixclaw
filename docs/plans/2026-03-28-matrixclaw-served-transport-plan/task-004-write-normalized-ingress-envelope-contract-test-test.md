# Task 004: Write normalized ingress envelope contract test

**depends-on**: task-001-implement-served-openclaw-http-transport-impl

## Description

Write a failing test for a normalized ingress envelope that keeps transport-specific identities and channel metadata out of the live runtime core.

## Execution Context

**Task Number**: 004 of 005  
**Phase**: Gateway-Ready Normalization  
**Prerequisites**: served OpenClaw HTTP transport exists

## BDD Scenario

```gherkin
Scenario: External channel input is normalized before entering the live runtime core
  Given a served transport request with protocol-specific sender and conversation metadata
  When app-host accepts the transport request
  Then the transport layer converts it into a normalized ingress envelope
  And the live runtime consumes the normalized envelope without transport-specific branching
  And reply routing metadata remains available to transport and gateway adapters
```

## Files to Modify/Create

- Create: `crates/app-host/tests/normalized_ingress_envelope.rs`

## Steps

### Step 1: Write the failing contract test
- Define assertions around the normalized envelope shape and usage.
- Keep the test focused on contract and boundaries, not messenger-specific implementations.

### Step 2: Confirm Red state
- Run the targeted test and confirm the normalized contract does not yet exist.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host normalized_ingress_envelope -- --exact
```

## Success Criteria

- A failing test exists for the gateway-ready ingress contract.
- The contract separates runtime inputs from protocol-specific transport metadata.

