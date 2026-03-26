# Task 005: [TEST] Tool preflight block

**depends-on**: task-004-tool-calls-extend-loop-impl

## Description

Create a failing test showing that a policy or hook can block a tool call before execution while still producing a structured tool-result message and keeping the run alive.

## Execution Context

**Task Number**: 005 of 019 (test)  
**Phase**: Core Runtime  
**Prerequisites**: Tool execution lifecycle exists and can be intercepted by policy hooks.

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

- Create: `crates/agent-core/tests/tool_preflight_block.rs`
- Create: `crates/agent-core/src/policy.rs`
- Modify: `crates/agent-core/src/tool.rs`
- Modify: `crates/agent-core/src/loop.rs`

## Steps

### Step 1: Verify Scenario

- Confirm blocked tools must result in structured messages rather than silent drops or crashes.

### Step 2: Create the failing Red test

- Use a policy test double that denies one tool invocation.
- Assert that the tool process is never executed, a block result is emitted, and the run reaches a stable continuation point.
- Keep the failure semantic by checking for missing block messages or accidental tool execution.

### Step 3: Lock the policy contract

- Define the `before_tool_call` style interface and block-result shape needed by the test.
- Do not implement blocking behavior in this task.

## Verification Commands

```bash
cargo test -p matrixclaw-agent-core tool_preflight_block -- --exact
cargo test -p matrixclaw-agent-core
```

## Success Criteria

- One failing test represents the blocked-tool scenario.
- The failure demonstrates missing preflight/policy behavior.
- Tool blocking remains isolated through explicit interfaces and doubles.
