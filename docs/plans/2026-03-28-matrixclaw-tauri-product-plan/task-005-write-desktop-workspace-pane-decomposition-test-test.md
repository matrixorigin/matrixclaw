# Task 005: Write desktop workspace pane decomposition test

**depends-on**: `task-003-implement-desktop-app-shell-architecture-impl`

## Goal
Create a failing test proving the workspace behaves like a desktop tool with persistent panes and contextual inspectors instead of one monolithic page.

## Scenario
Scenario: Workspace feels like a desktop tool with persistent panes
  Given the user is in the main MatrixClaw workspace
  When they browse files, read transcript output, and inspect run state
  Then the left sidebar, center workspace, and right inspector remain stable regions
  And file references, queue controls, and execution provenance appear in task-oriented places
  And diagnostics move behind detail surfaces instead of dominating the main layout

## Files
- Create or modify: `ui/tests/workspace_pane_layout.spec.ts`
- Expected future production files: `ui/src/routes/workspace/+page.svelte`, pane components under `ui/src/lib/`

## Verification
- `pnpm --dir ui exec playwright test ui/tests/workspace_pane_layout.spec.ts`
