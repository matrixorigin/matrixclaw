# Task 015: [IMPL] OpenClaw subprocess plugin install

**depends-on**: task-015-openclaw-subprocess-plugin-install-test

## Description

Implement shimmed process-boundary plugin import, normalized plugin manifests, and adapter-based plugin launch.

## Execution Context

**Task Number**: 015 of 019 (impl)  
**Phase**: Ecosystem Compatibility  
**Prerequisites**: The paired Red test fails because shimmed plugin installation behavior is missing.

## BDD Scenario

```gherkin
Scenario: User installs a subprocess-compatible plugin originally built for OpenClaw
  Given the user has an OpenClaw plugin that communicates through a stable subprocess protocol
  When the user installs the plugin into MatrixClaw
  Then MatrixClaw classifies it as shim-compatible
  And launches it through the appropriate adapter layer
  And exposes the plugin capabilities to the runtime as native tool or channel abstractions
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/manifests/src/plugin_manifest.rs`
- Modify: `crates/app-host/src/commands/install_plugin.rs`
- Modify: `crates/app-host/src/plugin_launcher.rs`
- Modify: `crates/app-host/src/compat_registry.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the shimmed plugin-install test still fails before implementation.

### Step 2: Implement minimal shimmed plugin behavior

- Classify process-boundary plugins as `shimmed`.
- Generate `matrixclaw.plugin.json` with adapter metadata and provenance.
- Launch the plugin through a native adapter contract and expose its capabilities to the runtime surface.

### Step 3: Verify Pass

- Run the targeted subprocess-plugin install test and confirm it passes.

### Step 4: Regression sweep

- Re-run manifest and app-host tests covering skill/plugin import flows.

## Verification Commands

```bash
cargo test -p matrixclaw-manifests install_subprocess_plugin -- --exact
cargo test -p matrixclaw-manifests
```

## Success Criteria

- Shimmed plugins install and launch through an explicit adapter.
- Runtime capability exposure remains process-boundary based.
- The targeted scenario passes.
