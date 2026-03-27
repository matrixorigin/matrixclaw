# Task 003: Write workspace streaming transcript UI test

**depends-on**: task-002-implement-session-backed-live-run-service-impl

## Description

Add a failing UI test proving the workspace transcript can render live assistant deltas and finalize them without duplication.

## Execution Context

**Task Number**: 003 of 009  
**Phase**: Browser Integration  
**Prerequisites**: session-backed run service exists

## BDD Scenario

```gherkin
Scenario: Browser transcript streams deltas without duplicating the final assistant message
  Given the browser workspace is connected to a running session
  When the user sends a prompt that yields assistant deltas
  Then the transcript renders those deltas in order
  And the final assistant message replaces the partial buffer exactly once
  And run detail metadata remains visible beside the transcript
```

**Spec Source**: scope-specific scenario derived from `../2026-03-26-rust-openclaw-runtime-design/runtime-model.md`

## Files to Modify/Create

- Create: `ui/tests/workspace-streaming-transcript.spec.ts`
- Modify: `ui/src/routes/workspace/+page.svelte`
- Modify: `crates/app-host/src/http/agent_api.rs`

## Steps

### Step 1: Define the browser contract
- Lock the event shape or stream shape the workspace page will consume.
- Use deterministic test doubles instead of a live provider.

### Step 2: Write the Red test
- Assert that partial assistant text appears before finalization.
- Assert that the final message is present only once when the run completes.
- Assert that provider/backend metadata remains visible in the right rail or transcript detail area.

### Step 3: Avoid implementation in the test task
- Do not wire the real browser runtime in this task.

## Verification Commands

```bash
pnpm --dir ui exec playwright test ui/tests/workspace-streaming-transcript.spec.ts
pnpm --dir ui check
```

## Success Criteria

- The streaming transcript behavior is defined by a failing browser test.
- The test distinguishes partial and final assistant rendering.
- The test fails for the intended missing streaming UI behavior.
