# Task 005: Implement workspace explorer and file reference flow

**depends-on**: task-005-write-workspace-explorer-and-file-reference-test-test

## Description

Implement the workspace explorer view and file-reference insertion flow on top of the local API contract.

## Execution Context

**Task Number**: 005 of 008  
**Phase**: Core Features  
**Prerequisites**: Red explorer/reference test exists

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

- Modify: `crates/app-host/src/http/workspace_api.rs`
- Modify: `ui/src/routes/workspace/+page.svelte`
- Modify: `ui/src/lib/workspace/`
- Optionally Create: `ui/src/lib/chat/composer.ts`

## Steps

### Step 1: Re-run the failing test
- Confirm the explorer/reference test still fails before implementation.

### Step 2: Implement the Green path
- Expose workspace listing data from `app-host`.
- Render explorer UI and wire reference insertion into the composer state.
- Preserve file immutability for reference-only actions.

### Step 3: Verify
- Run the targeted test.
- Re-run `app-host` tests.
- Run the chosen frontend test command if the feature introduces frontend unit tests.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host workspace_explorer_file_reference -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The workspace surface can browse files and insert references.
- Reference insertion does not mutate disk state.
- The feature remains compatible with the embedded shell model.
