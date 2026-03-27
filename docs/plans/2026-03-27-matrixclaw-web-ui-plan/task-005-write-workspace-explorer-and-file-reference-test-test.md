# Task 005: Write workspace explorer and file reference test

**depends-on**: task-003-implement-local-setup-server-and-shell-routing-impl

## Description

Add a failing test for the workspace surface proving the UI can list workspace files and generate prompt-safe file references without mutating file contents.

## Execution Context

**Task Number**: 005 of 008  
**Phase**: Core Features  
**Prerequisites**: local shell routing exists

## BDD Scenario

```gherkin
Scenario: Workspace explorer inserts file references without mutating files
  Given MatrixClaw has a configured workspace with files and directories
  When the user browses the workspace and selects a file to reference
  Then the UI lists the available paths
  And the selected file is inserted into the composer as a stable reference token
  And the file contents on disk remain unchanged
```

**Spec Source**: scope-specific scenario derived from `../2026-03-26-rust-openclaw-runtime-design/delivery-plan.md`

## Files to Modify/Create

- Create: `crates/app-host/tests/workspace_explorer_file_reference.rs`
- Create: `crates/app-host/src/http/workspace_api.rs`
- Create: `ui/src/routes/workspace/+page.svelte`
- Create: `ui/src/lib/workspace/`

## Steps

### Step 1: Verify Scenario
- Confirm the design wants a workspace-first UI, not only a settings panel.

### Step 2: Create the failing Red test
- Use a temp workspace with nested files.
- Assert the backend can enumerate paths for the UI contract.
- Assert the reference output is stable and does not alter files.

### Step 3: Lock the explorer contract
- Define path list and file reference payload structures.
- Avoid implementing real explorer behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host workspace_explorer_file_reference -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The workspace explorer contract is test-defined before implementation.
- File reference insertion is explicit and side-effect free.
- The test fails for the intended missing behavior.
