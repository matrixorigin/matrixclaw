# Task 011: [TEST] Compaction role semantics

**depends-on**: task-010-overflow-compaction-retry-impl

## Description

Create a failing session-runtime test proving compaction summaries are inserted as runtime/system artifacts rather than user-authored messages, while the full pre-compaction history remains recoverable.

## Execution Context

**Task Number**: 011 of 019 (test)  
**Phase**: Session Runtime  
**Prerequisites**: Overflow-triggered compaction exists and can emit summary artifacts.

## BDD Scenario

```gherkin
Scenario: Compaction preserves role semantics
  Given MatrixClaw compacts old conversation state
  When it inserts a summary artifact into active context
  Then that artifact is represented as a system or runtime summary message
  And it is not persisted as a user-authored message
  And the full pre-compaction history remains recoverable
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Create: `crates/session-runtime/tests/compaction_preserves_role_semantics.rs`
- Modify: `crates/session-runtime/src/compaction.rs`
- Modify: `crates/session-runtime/src/message_projection.rs`
- Create: `crates/session-runtime/src/compaction_record.rs`

## Steps

### Step 1: Verify Scenario

- Confirm compaction summaries must never be persisted as `user` messages.

### Step 2: Create the failing Red test

- Compact a session in a test database and inspect both active context and durable transcript.
- Assert that the summary artifact has runtime/system semantics and that original messages remain recoverable through compaction metadata.
- Keep the failure semantic by checking for user-role contamination or destructive history loss.

### Step 3: Lock the compaction-record contract

- Define the summary artifact type and compaction record shape needed by the test.
- Do not implement semantic fixes in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime compaction_preserves_role_semantics -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- One failing test covers role-safe compaction.
- The failure demonstrates incorrect role semantics or lost history.
- Compaction record structure is explicit and testable.
