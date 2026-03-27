# Task 003: Implement workspace streaming transcript UI

**depends-on**: task-003-write-workspace-streaming-transcript-ui-test-test

## Description

Implement the workspace transcript so live assistant deltas, finalization, and run metadata render correctly from the shared runtime service.

## Execution Context

**Task Number**: 003 of 009  
**Phase**: Browser Integration  
**Prerequisites**: failing streaming transcript UI test exists

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

- Modify: `ui/src/routes/workspace/+page.svelte`
- Modify: `ui/src/lib/http.ts`
- Modify: `ui/tests/ui-smoke.spec.ts`
- Optionally Create: `ui/src/lib/chat/stream.ts`

## Steps

### Step 1: Re-run the Red browser test
- Confirm the transcript streaming UI test still fails before implementation.

### Step 2: Implement the Green path
- Subscribe to the live run event stream or equivalent progressive runtime response.
- Render partial assistant text and finalize it without duplicating the message.
- Preserve provider/backend metadata in the transcript or run detail surface.

### Step 3: Verify
- Run the targeted browser test.
- Re-run the existing workspace browser smoke.
- Re-run frontend type/build checks.

## Verification Commands

```bash
pnpm --dir ui exec playwright test ui/tests/workspace-streaming-transcript.spec.ts
pnpm --dir ui test:e2e
pnpm --dir ui check
pnpm --dir ui build
```

## Success Criteria

- The browser transcript is event-driven instead of final-message-only.
- Final assistant rendering is replay-safe and non-duplicating.
- Existing workspace smoke remains green.
