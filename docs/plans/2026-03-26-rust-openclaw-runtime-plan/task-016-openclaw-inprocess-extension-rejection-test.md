# Task 016: [TEST] OpenClaw in-process extension rejection

**depends-on**: task-015-openclaw-subprocess-plugin-install-impl

## Description

Create a failing compatibility test proving an in-process OpenClaw extension is rejected with a precise diagnostic instead of being silently accepted or partially supported.

## Execution Context

**Task Number**: 016 of 019 (test)  
**Phase**: Ecosystem Compatibility  
**Prerequisites**: Shimmed plugin classification and manifest normalization already exist.

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

- Create: `crates/manifests/tests/reject_inprocess_extension.rs`
- Modify: `crates/manifests/src/plugin_manifest.rs`
- Modify: `crates/app-host/src/commands/install_plugin.rs`

## Steps

### Step 1: Verify Scenario

- Confirm in-process JS/Bun extensions are outside native compatibility promises.

### Step 2: Create the failing Red test

- Use an extension fixture with `openclaw.plugin.json`, TypeScript entrypoints, and in-process assumptions.
- Assert that install refuses the artifact with a bridge/manual-port diagnostic.
- Keep the failure semantic by checking for silent acceptance or vague errors.

### Step 3: Lock the rejection contract

- Define only the classifier reason codes and install-result diagnostics needed by the test.
- Do not implement rejection behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-manifests reject_inprocess_extension -- --exact
cargo test -p matrixclaw-manifests
```

## Success Criteria

- One failing test covers unsupported in-process extensions.
- The failure demonstrates missing diagnostics or incorrect compatibility claims.
- Rejection remains explicit and machine-readable.
