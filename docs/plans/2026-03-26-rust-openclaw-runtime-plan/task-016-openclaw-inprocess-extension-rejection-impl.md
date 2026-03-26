# Task 016: [IMPL] OpenClaw in-process extension rejection

**depends-on**: task-016-openclaw-inprocess-extension-rejection-test

## Description

Implement explicit rejection and diagnostics for in-process OpenClaw extensions that depend on JS or Bun runtime internals.

## Execution Context

**Task Number**: 016 of 019 (impl)  
**Phase**: Ecosystem Compatibility  
**Prerequisites**: The paired Red test fails because unsupported extensions are not rejected clearly.

## BDD Scenario

```gherkin
Scenario: User tries to install an in-process OpenClaw extension tied to JS internals
  Given the user has an OpenClaw extension that depends on in-process TypeScript or Bun runtime APIs
  When the user attempts to install it into MatrixClaw
  Then MatrixClaw refuses native installation
  And explains that this artifact requires a bridge runtime or manual rewrite
  And does not claim partial compatibility silently
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/manifests/src/plugin_manifest.rs`
- Modify: `crates/app-host/src/commands/install_plugin.rs`
- Modify: `crates/manifests/src/provenance.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the in-process extension rejection test still fails before implementation.

### Step 2: Implement minimal rejection behavior

- Detect in-process runtime assumptions from plugin metadata and source layout.
- Refuse install with explicit `bridge_only` or `unsupported` diagnostics as appropriate.
- Ensure install output and machine-readable results do not imply partial compatibility.

### Step 3: Verify Pass

- Run the targeted rejection test and confirm it passes.

### Step 4: Regression sweep

- Re-run manifest tests covering native, shimmed, and unsupported artifact classes.

## Verification Commands

```bash
cargo test -p matrixclaw-manifests reject_inprocess_extension -- --exact
cargo test -p matrixclaw-manifests
```

## Success Criteria

- Unsupported in-process extensions fail with precise diagnostics.
- Compatibility tiers stay honest.
- The targeted scenario passes.
