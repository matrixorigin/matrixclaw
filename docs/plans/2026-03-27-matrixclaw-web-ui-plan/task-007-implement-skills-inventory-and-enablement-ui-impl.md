# Task 007: Implement skills inventory and enablement UI

**depends-on**: task-007-write-skills-inventory-and-enablement-test-test

## Description

Implement the Skills management view and its backend boundary so operators can inspect installed skills and toggle agent-local enablement safely.

## Execution Context

**Task Number**: 007 of 008  
**Phase**: Core Features  
**Prerequisites**: Red skills inventory test exists

## BDD Scenario

```gherkin
Scenario: Skills inventory reflects installed and enabled state separately
  Given MatrixClaw has imported skills and agent-local enablement metadata
  When the user opens the Skills view and changes enablement for the current agent
  Then the UI shows installed skill inventory separately from enabled state
  And the runtime updates only agent-local enablement metadata
  And imported skill source files remain unchanged
```

**Spec Source**: scope-specific scenario derived from `../2026-03-26-rust-openclaw-runtime-design/install-and-layout.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/http/skills_api.rs`
- Modify: `ui/src/routes/skills/+page.svelte`
- Modify: `ui/src/lib/skills/`
- Optionally Modify: `crates/app-host/src/commands/install_skill.rs`

## Steps

### Step 1: Re-run the failing test
- Confirm the skills inventory test still fails before implementation.

### Step 2: Implement the Green path
- Expose installed and enabled states distinctly.
- Render the Skills view with clear status separation.
- Implement enable/disable mutations against agent-local metadata only.

### Step 3: Verify
- Run the targeted skills test.
- Re-run `app-host` tests.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host skills_inventory_enablement -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Skills inventory and enablement state are both visible.
- Enablement changes are safe and non-destructive.
- The Skills view aligns with the broader browser-first operator surface.
