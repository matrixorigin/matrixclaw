# Task 003: Write desktop app shell contract test

**depends-on**: `task-001-implement-bundled-asset-packaging-impl`

## Goal
Create a failing UI contract test proving MatrixClaw renders as a persistent app shell instead of a landing page plus route cards.

## Scenario
Scenario: Desktop shell feels like an app rather than a preview site
  Given the user opens MatrixClaw after setup or restore
  When the main shell renders
  Then navigation is persistent inside the app frame
  And the workspace remains the primary center surface
  And inspector and status regions are available without leaving the app
  And the shell does not present itself as a developer preview or marketing surface

## Files
- Create or modify: `ui/tests/desktop_app_shell.spec.ts`
- Expected future production files: `ui/src/routes/+layout.svelte`, nested app-shell route files

## Verification
- `pnpm --dir ui exec playwright test ui/tests/desktop_app_shell.spec.ts`
