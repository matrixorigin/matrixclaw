# Task 015: [TEST] OpenClaw subprocess plugin install

**depends-on**: task-014-openclaw-text-skill-import-impl

## Description

Create a failing plugin-installation test proving a process-boundary OpenClaw plugin is classified as shim-compatible, installed, and launched through an adapter layer.

## Execution Context

**Task Number**: 015 of 019 (test)  
**Phase**: Ecosystem Compatibility  
**Prerequisites**: Skill import, provenance, and runtime-home layout already exist.

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

- Create: `crates/manifests/tests/install_subprocess_plugin.rs`
- Create: `crates/manifests/src/plugin_manifest.rs`
- Create: `crates/app-host/src/commands/install_plugin.rs`
- Create: `crates/app-host/src/plugin_launcher.rs`

## Steps

### Step 1: Verify Scenario

- Confirm subprocess and MCP-style plugins belong to the shimmed support tier.

### Step 2: Create the failing Red test

- Use a plugin fixture containing process-boundary metadata.
- Assert that install classification returns `shimmed`, writes `matrixclaw.plugin.json`, and can launch the plugin through a stub adapter.
- Keep the failure semantic by checking for wrong tiering, missing adapter metadata, or no launcher contract.

### Step 3: Lock the plugin contract

- Define plugin manifest types, adapter launch signatures, and install-command interfaces needed by the test.
- Do not implement install or launch behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-manifests install_subprocess_plugin -- --exact
cargo test -p matrixclaw-manifests
```

## Success Criteria

- One failing test covers shimmed subprocess plugin installation.
- The failure shows missing classification or adapter wiring.
- Plugin support remains process-boundary based.
