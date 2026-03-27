# Task 006: Write queued steering controls UI test

**depends-on**: task-003-implement-local-setup-server-and-shell-routing-impl

## Description

Add a failing test proving the local UI can submit steering and follow-up intents through the runtime queue boundary while rendering their queued state distinctly.

## Execution Context

**Task Number**: 006 of 008  
**Phase**: Core Features  
**Prerequisites**: local shell routing and HTTP surface exist

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

- Create: `crates/app-host/tests/queued_controls_ui.rs`
- Create: `crates/app-host/src/http/queue_api.rs`
- Create: `ui/src/lib/queue/`
- Modify: `ui/src/routes/workspace/+page.svelte`

## Steps

### Step 1: Verify Scenario
- Reconcile the UI scenario with existing session-runtime queue semantics before writing the test.

### Step 2: Create the failing Red test
- Assert steering and follow-up requests hit distinct backend routes or payload states.
- Assert returned queue state differentiates immediate-next-turn vs deferred-next-run behavior.
- Confirm the test fails for missing UI contract behavior.

### Step 3: Lock queue UI contracts
- Define queue request and queue state response shapes.
- Do not implement real queue handling in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host queued_controls_ui -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- A Red test captures UI-visible queue semantics accurately.
- Steering and follow-up are not conflated in the contract.
- The failure is isolated to missing implementation.
