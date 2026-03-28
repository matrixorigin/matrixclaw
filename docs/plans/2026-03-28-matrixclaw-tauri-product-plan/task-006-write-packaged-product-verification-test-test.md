# Task 006: Write packaged product verification test

**depends-on**: `task-002-implement-tauri-embedded-runtime-startup-impl`, `task-004-implement-multi-step-onboarding-flow-impl`, `task-005-implement-desktop-workspace-pane-decomposition-impl`

## Goal
Create a failing product-level verification harness proving a packaged MatrixClaw app can launch, render setup, complete onboarding, and reach the workspace without source-tree assumptions.

## Scenario
Scenario: Packaged product verification catches regressions before release
  Given a clean-home packaged MatrixClaw installation
  When the app launches and a first-run user enters the product
  Then bundled assets are present
  And setup renders and can complete
  And the workspace opens in the same app shell
  And missing bundled assets or broken launch wiring fail the release gate

## Files
- Create or modify: packaged-product smoke script or test harness under `scripts/` and/or `apps/desktop-shell/`
- Expected future production files: product verification harnesses and install/build docs

## Verification
- packaged-product verification command to be introduced by the implementation task
