# Task 005: Implement desktop workspace pane decomposition

**depends-on**: `task-005-write-desktop-workspace-pane-decomposition-test-test`

## Goal
Decompose the current monolithic workspace into a desktop-grade pane model with clearer ownership and room for future keyboard, inspector, and Node-oriented capability surfaces.

## Scenario
Scenario: Workspace feels like a desktop tool with persistent panes
  Given the user is in the main MatrixClaw workspace
  When they browse files, read transcript output, and inspect run state
  Then the left sidebar, center workspace, and right inspector remain stable regions
  And file references, queue controls, and execution provenance appear in task-oriented places
  And diagnostics move behind detail surfaces instead of dominating the main layout

## Files
- Modify: `ui/src/routes/workspace/+page.svelte`
- Add: pane components and stores under `ui/src/lib/`
- Modify: related route/layout files as needed for persistent inspector and status regions

## Verification
- `pnpm --dir ui exec playwright test ui/tests/workspace_pane_layout.spec.ts`
