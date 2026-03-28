# Task 004: Implement normalized ingress envelope contract

**depends-on**: task-004-write-normalized-ingress-envelope-contract-test-test

## Description

Add a normalized ingress envelope and adapter boundary so future IM gateways can feed the live runtime without changing its core request model.

## Execution Context

**Task Number**: 004 of 005  
**Phase**: Gateway-Ready Normalization  
**Prerequisites**: failing normalized ingress test exists

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

- Create or modify: `crates/app-host/src/ingress.rs`
- Modify: `crates/app-host/src/openclaw_transport.rs`
- Modify: `crates/app-host/src/http/openclaw_api.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the ingress contract test still fails.

### Step 2: Implement the normalized boundary
- Introduce a channel-agnostic ingress envelope.
- Preserve sender identity, channel/thread identity, target agent identity, payload, and reply-routing metadata.
- Keep the live runtime free of OpenClaw-specific field names.

### Step 3: Verify
- Run the targeted contract test.
- Re-run app-host tests.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host normalized_ingress_envelope -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Transport adapters normalize ingress before runtime execution.
- Future IM gateways have a stable contract to target.

