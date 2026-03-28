# Task 003: Implement desktop app shell architecture

**depends-on**: `task-003-write-desktop-app-shell-contract-test-test`

## Goal
Replace the current preview-style shell with a persistent desktop app shell that can host setup, workspace, skills, and settings coherently.

## Scenario
Scenario: Desktop shell feels like an app rather than a preview site
  Given the user opens MatrixClaw after setup or restore
  When the main shell renders
  Then navigation is persistent inside the app frame
  And the workspace remains the primary center surface
  And inspector and status regions are available without leaving the app
  And the shell does not present itself as a developer preview or marketing surface

## Files
- Modify: `ui/src/routes/+layout.svelte`
- Modify: `ui/src/routes/+page.svelte`
- Add or modify: nested route/layout files under `ui/src/routes/`
- Add or modify: shared app-shell components under `ui/src/lib/`

## Verification
- `pnpm --dir ui exec playwright test ui/tests/desktop_app_shell.spec.ts`
