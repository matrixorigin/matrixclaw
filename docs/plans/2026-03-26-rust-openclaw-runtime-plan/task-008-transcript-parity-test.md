# Task 008: [TEST] Transcript parity

**depends-on**: task-005-tool-preflight-block-impl

## Description

Create a failing session-runtime persistence test that proves the durable transcript exactly matches user-visible behavior, including tool results and visible terminal warnings.

## Execution Context

**Task Number**: 008 of 019 (test)  
**Phase**: Session Runtime  
**Prerequisites**: Core event sequencing, tool-result messages, and policy-block results exist.

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

- Create: `crates/session-runtime/tests/transcript_matches_visible_behavior.rs`
- Create: `crates/session-runtime/src/storage.rs`
- Create: `crates/session-runtime/src/sqlite.rs`
- Modify: `crates/session-runtime/src/message_projection.rs`

## Steps

### Step 1: Verify Scenario

- Confirm transcript durability must include all user-visible assistant, tool, warning, and failure outputs.

### Step 2: Create the failing Red test

- Simulate a run with assistant output, tool results, a visible warning, and a retry marker using test doubles.
- Persist the run into a temporary SQLite database and inspect the stored transcript.
- Keep the failure semantic by checking for missing terminal messages or omitted tool results.

### Step 3: Lock the storage contract

- Define message persistence records, storage trait signatures, and transcript projection interfaces needed by the test.
- Do not implement storage behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime transcript_matches_visible_behavior -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- A single failing test covers transcript parity.
- Storage behavior is isolated with a temporary SQLite test database and doubles.
- The failure reflects missing durable messages rather than schema setup issues.
