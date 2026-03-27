# Task 006: Implement queued steering controls UI

**depends-on**: task-006-write-queued-steering-controls-ui-test-test

## Description

Implement the queue-control UI and `app-host` API boundary for steering and follow-up submission, preserving the already-defined runtime delivery semantics.

## Execution Context

**Task Number**: 006 of 008  
**Phase**: Core Features  
**Prerequisites**: Red queue-controls UI test exists

## BDD Scenario

```gherkin
Scenario: Queued steering and follow-up controls are visible and correct in the local UI
  Given MatrixClaw has an active run with queue-capable session runtime state
  When the user submits a steering or follow-up message from the local UI
  Then steering is queued for the next assistant turn
  And follow-up remains deferred until the current run completes
  And the UI renders the queued state without misrepresenting delivery timing
```

**Spec Source**: scope-specific scenario derived from `../2026-03-26-rust-openclaw-runtime-design/delivery-plan.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/http/queue_api.rs`
- Modify: `ui/src/lib/queue/`
- Modify: `ui/src/routes/workspace/+page.svelte`
- Optionally Modify: `crates/app-host/src/lib.rs`

## Steps

### Step 1: Re-run the failing test
- Confirm the queue-controls UI test still fails before implementation.

### Step 2: Implement the Green path
- Expose queue submission endpoints or commands from `app-host`.
- Render steering and follow-up controls with distinct status presentation.
- Reuse existing session-runtime semantics instead of inventing new queue behavior in the UI.

### Step 3: Verify
- Run the focused queue-controls test.
- Re-run `app-host` tests.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host queued_controls_ui -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- The UI exposes steering and follow-up distinctly.
- Runtime semantics remain correct.
- The feature is additive and does not regress the setup or explorer slices.
