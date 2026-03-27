# Task 005: Implement session resume over HTTP

**depends-on**: task-005-write-session-resume-over-http-test-test

## Description

Implement session continuation for the live HTTP/browser path so persisted runs survive restart and remain usable for subsequent prompts.

## Execution Context

**Task Number**: 005 of 009  
**Phase**: Persistence  
**Prerequisites**: failing session resume test exists

## BDD Scenario

```gherkin
Scenario: Session resumes after restart
  Given an existing persisted session
  When MatrixClaw restarts
  Then the session runtime reloads the prior message history
  And the next prompt continues from the persisted state
  And the runtime can reconstruct queued metadata needed for further processing
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `crates/app-host/src/live_runtime.rs`
- Modify: `crates/session-runtime/src/session.rs`
- Modify: `crates/session-runtime/src/sqlite.rs`
- Modify: `ui/src/routes/workspace/+page.svelte`

## Steps

### Step 1: Re-run the Red test
- Confirm the resume-over-HTTP test still fails before implementation.

### Step 2: Implement the Green path
- Accept or derive a stable session identifier in the live chat boundary.
- Reload persisted transcript and queue metadata after restart.
- Keep browser state aligned with the resumed session id.

### Step 3: Verify
- Run the targeted session resume test.
- Re-run affected session-runtime persistence tests.
- Re-run frontend checks if browser state changes.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host session_resume_over_http -- --exact
cargo test -p matrixclaw-session-runtime session_resume_after_restart -- --exact
cargo test -p matrixclaw-app-host
pnpm --dir ui check
```

## Success Criteria

- Live chat requests can continue a persisted session.
- Restarted runtime state remains coherent for transcript and queue behavior.
- Existing session-runtime resume guarantees remain intact.
