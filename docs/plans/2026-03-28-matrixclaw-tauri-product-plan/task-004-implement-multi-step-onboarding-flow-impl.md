# Task 004: Implement multi-step onboarding flow

**depends-on**: `task-004-write-multi-step-onboarding-flow-test-test`

## Goal
Turn setup into a desktop-grade onboarding flow with real steps, validation, and resumable draft state.

## Scenario
Scenario: First-run setup behaves like a real onboarding flow
  Given MatrixClaw launches without saved configuration
  When the user enters setup
  Then provider, workspace, auth, execution, and review are distinct steps
  And step validation prevents incomplete progress
  And the user can resume the flow without losing draft state
  And completion transitions into the same app shell rather than a separate product mode

## Files
- Modify: `ui/src/routes/setup/+page.svelte`
- Add or modify: nested setup step routes or state-machine support under `ui/src/routes/setup/`
- Modify: `ui/src/lib/setup/state.ts`
- Modify: setup-related runtime/API files if needed for draft or review support

## Verification
- `pnpm --dir ui exec playwright test ui/tests/setup_onboarding_flow.spec.ts`
