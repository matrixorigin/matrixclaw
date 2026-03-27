# Task 007: Write skills inventory and enablement test

**depends-on**: task-003-implement-local-setup-server-and-shell-routing-impl

## Description

Add a failing test proving the Skills view distinguishes globally installed skills from agent-local enabled state and updates only the enablement metadata.

## Execution Context

**Task Number**: 007 of 008  
**Phase**: Core Features  
**Prerequisites**: local shell routing and API base exist

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

- Create: `crates/app-host/tests/skills_inventory_enablement.rs`
- Create: `crates/app-host/src/http/skills_api.rs`
- Create: `ui/src/routes/skills/+page.svelte`
- Create: `ui/src/lib/skills/`

## Steps

### Step 1: Verify Scenario
- Confirm the design requires separate installed/enabled lifecycle stages for skills.

### Step 2: Create the failing Red test
- Assert the API exposes installed skill metadata and enabled state separately.
- Assert toggling enablement changes only `enabled-skills` metadata.
- Assert imported source artifacts are unchanged.

### Step 3: Lock Skills UI contracts
- Define skill list and enablement mutation payloads.
- Do not implement UI or mutation behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host skills_inventory_enablement -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- A failing test captures the installed-vs-enabled distinction.
- The mutation boundary is explicit and non-destructive.
- The contract is ready for UI implementation.
