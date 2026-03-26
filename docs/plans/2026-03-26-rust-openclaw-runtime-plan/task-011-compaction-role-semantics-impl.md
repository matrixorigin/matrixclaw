# Task 011: [IMPL] Compaction role semantics

**depends-on**: task-011-compaction-role-semantics-test

## Description

Implement role-safe compaction summaries and durable compaction records so summaries remain runtime artifacts and original history stays recoverable.

## Execution Context

**Task Number**: 011 of 019 (impl)  
**Phase**: Session Runtime  
**Prerequisites**: The paired Red test fails because compaction artifacts use the wrong role or destroy history.

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

- Modify: `crates/session-runtime/src/compaction.rs`
- Modify: `crates/session-runtime/src/compaction_record.rs`
- Modify: `crates/session-runtime/src/message_projection.rs`
- Modify: `crates/session-runtime/src/sqlite.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the role-semantics test still fails before implementation.

### Step 2: Implement minimal role-safe compaction

- Represent summary artifacts as runtime/system messages in active context.
- Persist compaction records separately from user-authored transcript messages.
- Preserve a recoverable mapping from compacted history to summary artifact.

### Step 3: Verify Pass

- Run the targeted compaction role-semantics test and confirm it passes.

### Step 4: Regression sweep

- Re-run session-runtime tests to protect compaction, retry, and transcript behavior together.

## Verification Commands

```bash
cargo test -p matrixclaw-session-runtime compaction_preserves_role_semantics -- --exact
cargo test -p matrixclaw-session-runtime
```

## Success Criteria

- Compaction summaries are never stored as user-authored messages.
- Original history remains recoverable through compaction records.
- The targeted scenario passes.
