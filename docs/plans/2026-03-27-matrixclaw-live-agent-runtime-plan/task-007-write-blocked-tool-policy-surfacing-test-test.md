# Task 007: Write blocked tool policy surfacing test

**depends-on**: task-006-implement-live-tool-execution-impl

## Description

Add a failing test proving blocked tool decisions are visible and durable in the live runtime path instead of being swallowed or treated like silent failures.

## Execution Context

**Task Number**: 007 of 009  
**Phase**: Safety  
**Prerequisites**: live tool execution exists

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

- Create: `crates/app-host/tests/blocked_tool_policy_surfacing.rs`
- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `ui/src/routes/workspace/+page.svelte`

## Steps

### Step 1: Define blocked-tool product behavior
- Lock how blocked-tool events appear in persisted transcript and product responses.
- Use doubles for policy/tool/provider behavior.

### Step 2: Write the Red test
- Assert blocked tools do not execute.
- Assert a visible block result is persisted and returned.
- Assert the run continues cleanly after the block.

### Step 3: Keep the task Red-only
- Do not implement blocked-tool surfacing yet.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host blocked_tool_policy_surfacing -- --exact
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Blocked-tool behavior is explicit at the live runtime boundary.
- The test fails for the intended missing safety surfacing.
- The design promise against silent failure is preserved.
