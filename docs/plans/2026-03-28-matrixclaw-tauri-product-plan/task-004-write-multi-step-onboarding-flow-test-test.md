# Task 004: Write multi-step onboarding flow test

**depends-on**: `task-003-implement-desktop-app-shell-architecture-impl`

## Goal
Create a failing test proving setup is a multi-step, resumable onboarding flow inside the app shell rather than a single-page form.

## Scenario
Scenario: First-run setup behaves like a real onboarding flow
  Given MatrixClaw launches without saved configuration
  When the user enters setup
  Then provider, workspace, auth, execution, and review are distinct steps
  And step validation prevents incomplete progress
  And the user can resume the flow without losing draft state
  And completion transitions into the same app shell rather than a separate product mode

## Files
- Create or modify: `ui/tests/setup_onboarding_flow.spec.ts`
- Expected future production files: `ui/src/routes/setup/`, `ui/src/lib/setup/`

## Verification
- `pnpm --dir ui exec playwright test ui/tests/setup_onboarding_flow.spec.ts`
