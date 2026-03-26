# Task 014: [IMPL] OpenClaw text skill import

**depends-on**: task-014-openclaw-text-skill-import-test

## Description

Implement native `SKILL.md` import, manifest normalization, and provenance recording for OpenClaw text skills.

## Execution Context

**Task Number**: 014 of 019 (impl)  
**Phase**: Ecosystem Compatibility  
**Prerequisites**: The paired Red test fails because native skill import behavior is missing.

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

- Modify: `crates/manifests/src/skill_manifest.rs`
- Create: `crates/manifests/src/provenance.rs`
- Modify: `crates/app-host/src/commands/install_skill.rs`
- Create: `crates/app-host/src/compat_registry.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the skill-import test still fails before implementation.

### Step 2: Implement minimal native skill import

- Detect `SKILL.md`-style skill roots.
- Normalize metadata into `matrixclaw.skill.json`.
- Materialize the imported skill into the runtime home and record provenance/compatibility tier in runtime state.

### Step 3: Verify Pass

- Run the targeted skill-import test and confirm it passes.

### Step 4: Regression sweep

- Re-run manifest tests to protect schema and import behavior.

## Verification Commands

```bash
cargo test -p matrixclaw-manifests import_openclaw_skill -- --exact
cargo test -p matrixclaw-manifests
```

## Success Criteria

- OpenClaw `SKILL.md` artifacts import natively with normalized metadata.
- Provenance is recorded explicitly.
- The targeted scenario passes without adding JS runtime dependencies.
