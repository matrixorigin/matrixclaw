# Task 008: [IMPL] Transcript parity

**depends-on**: task-008-transcript-parity-test

## Description

Implement durable message persistence and transcript projection so stored history matches the visible run exactly.

## Execution Context

**Task Number**: 008 of 019 (impl)  
**Phase**: Session Runtime  
**Prerequisites**: The paired Red test fails because durable transcript state is incomplete or mis-ordered.

## BDD Scenario

```gherkin
Scenario: Stored transcript matches visible behavior
  Given a user has completed a conversation with tool calls, retries, and warnings
  When MatrixClaw persists the session
  Then every user-visible assistant message is present in session storage
  And every tool result that influenced the assistant is present in session storage
  And terminal warning or failure messages are also persisted when shown to the user
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/session-runtime/src/storage.rs`
- Modify: `crates/session-runtime/src/sqlite.rs`
- Modify: `crates/session-runtime/src/message_projection.rs`
- Create: `crates/session-runtime/src/event_sink.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the transcript-parity test still fails before implementation.

### Step 2: Implement minimal persistence behavior

- Persist finalized assistant messages, tool results, and visible warnings/errors as first-class durable records.
- Project the transcript from durable storage rather than ephemeral in-memory UI state.
- Preserve event ordering as stored message ordering.

### Step 3: Verify Pass

- Run the targeted transcript-parity test and confirm it passes.

### Step 4: Regression sweep

- Re-run the session-runtime package tests to protect queue and persistence behavior together.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime transcript_matches_visible_behavior -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- Durable transcript records everything the user saw.
- Tool results and terminal warnings persist in the same conversation history.
- The targeted scenario passes.
