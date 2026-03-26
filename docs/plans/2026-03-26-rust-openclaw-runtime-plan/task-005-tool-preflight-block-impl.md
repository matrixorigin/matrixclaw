# Task 005: [IMPL] Tool preflight block

**depends-on**: task-005-tool-preflight-block-test

## Description

Implement tool preflight policy evaluation so denied tools emit structured block results and do not execute while the loop continues safely.

## Execution Context

**Task Number**: 005 of 019 (impl)  
**Phase**: Core Runtime  
**Prerequisites**: The paired Red test exists and fails on missing preflight behavior.

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

- Modify: `crates/agent-core/src/policy.rs`
- Modify: `crates/agent-core/src/tool.rs`
- Modify: `crates/agent-core/src/loop.rs`
- Modify: `crates/agent-core/src/message.rs`

## Steps

### Step 1: Re-run the failing test

- Confirm the blocked-tool test still fails before implementation.

### Step 2: Implement minimal preflight blocking

- Evaluate policy before tool execution.
- Produce a structured blocked-tool result and convert it into a first-class tool-result message.
- Ensure the run continues through the normal loop path rather than crashing or silently returning.

### Step 3: Verify Pass

- Run the targeted preflight-block test and confirm it passes.

### Step 4: Regression sweep

- Re-run core-loop tests to protect tool sequencing and stream parity.

## Verification Commands

```bash
cargo test -p matrixclaw-agent-core tool_preflight_block -- --exact
cargo test -p matrixclaw-agent-core
```

## Success Criteria

- Denied tools never execute.
- Blocked calls generate structured, reusable tool-result messages.
- The targeted scenario passes without destabilizing the loop.
