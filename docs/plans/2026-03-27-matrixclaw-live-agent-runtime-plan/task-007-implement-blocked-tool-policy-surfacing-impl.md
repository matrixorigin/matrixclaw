# Task 007: Implement blocked tool policy surfacing

**depends-on**: task-007-write-blocked-tool-policy-surfacing-test-test

## Description

Implement blocked-tool persistence and UI/runtime surfacing so unsafe tool calls are visibly rejected without breaking the live run.

## Execution Context

**Task Number**: 007 of 009  
**Phase**: Safety  
**Prerequisites**: failing blocked-tool surfacing test exists

## BDD Scenario

```gherkin
Scenario: Tool validation can block unsafe execution
  Given a tool call is emitted by the model
  And a policy or hook determines the tool should not run
  When the tool preflight step executes
  Then the tool is blocked before invocation
  And a tool result message describing the block is emitted
  And the loop continues without crashing
```

**Spec Source**: `../2026-03-26-rust-openclaw-runtime-design/bdd-specs.md`

## Files to Modify/Create

- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `crates/app-host/src/live_runtime.rs`
- Modify: `ui/src/routes/workspace/+page.svelte`
- Modify: `crates/session-runtime/src/message_projection.rs`

## Steps

### Step 1: Re-run the Red test
- Confirm the blocked-tool surfacing test still fails before implementation.

### Step 2: Implement the Green path
- Persist blocked-tool results in the same transcript path as visible warnings/tool results.
- Render blocked-tool information in the browser transcript without mislabeling it as normal success.
- Preserve run continuity after a blocked action.

### Step 3: Verify
- Run the targeted blocked-tool test.
- Re-run core safety tests from `agent-core`.
- Re-run frontend checks if transcript rendering changes.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host blocked_tool_policy_surfacing -- --exact
cargo test -p matrixclaw-agent-core tool_preflight_block -- --exact
cargo test -p matrixclaw-app-host
pnpm --dir ui check
```

## Success Criteria

- Unsafe tool calls are visibly blocked and persisted.
- The live run path matches the core safety design.
- Browser transcript and stored transcript stay aligned.
