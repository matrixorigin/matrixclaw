# Task 014: [TEST] OpenClaw text skill import

**depends-on**: task-001-install-without-privileged-writes-impl

## Description

Create a failing compatibility/import test proving an OpenClaw-style `SKILL.md` package is detected, normalized, and installed without requiring Node.js or Bun.

## Execution Context

**Task Number**: 014 of 019 (test)  
**Phase**: Ecosystem Compatibility  
**Prerequisites**: Runtime-home layout and install command conventions exist.

## BDD Scenario

```gherkin
Scenario: User installs a text skill originally built for OpenClaw
  Given the user has an OpenClaw-style markdown or text skill artifact
  When the user installs that skill into MatrixClaw
  Then MatrixClaw recognizes and imports the skill metadata
  And the skill becomes available to the runtime without requiring Node.js or Bun
  And the runtime records the artifact as an imported compatibility skill
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/manifests/Cargo.toml`
- Create: `crates/manifests/tests/import_openclaw_skill.rs`
- Create: `crates/manifests/src/lib.rs`
- Create: `crates/manifests/src/skill_manifest.rs`
- Create: `crates/app-host/src/commands/install_skill.rs`

## Steps

### Step 1: Verify Scenario

- Confirm skill import is a native compatibility tier and must preserve provenance.

### Step 2: Create the failing Red test

- Use a fixture directory containing `SKILL.md`.
- Assert that `matrixclaw skill install` classifies the artifact as native, writes `matrixclaw.skill.json`, and records provenance in runtime state.
- Keep the failure semantic by checking for missed classification or hidden Node/Bun assumptions.

### Step 3: Lock the skill-manifest contract

- Define manifest types, classification result types, and install-command signatures needed by the test.
- Do not implement import behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-manifests import_openclaw_skill -- --exact
cargo test -p matrixclaw-manifests
```

## Success Criteria

- One failing test covers native OpenClaw skill import.
- The failure demonstrates missing classification or provenance recording.
- Skill support remains data-first and runtime-agnostic.
