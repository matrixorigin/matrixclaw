# Task 006: Implement packaged product verification

**depends-on**: `task-006-write-packaged-product-verification-test-test`

## Goal
Add a release-grade verification path for the actual packaged product so missing assets, broken startup, and first-run regressions are caught before shipping.

## Scenario
Scenario: Packaged product verification catches regressions before release
  Given a clean-home packaged MatrixClaw installation
  When the app launches and a first-run user enters the product
  Then bundled assets are present
  And setup renders and can complete
  And the workspace opens in the same app shell
  And missing bundled assets or broken launch wiring fail the release gate

## Files
- Modify or add: packaged-product smoke scripts under `scripts/`
- Modify or add: desktop-shell verification harnesses under `apps/desktop-shell/`
- Modify: relevant docs for product verification commands

## Verification
- packaged-product verification command introduced by this task
