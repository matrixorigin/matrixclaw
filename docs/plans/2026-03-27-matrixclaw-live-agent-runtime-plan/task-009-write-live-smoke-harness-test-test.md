# Task 009: Write live smoke harness test

**depends-on**: task-003-implement-workspace-streaming-transcript-ui-impl, task-008-implement-compatibility-runtime-reuse-impl

## Description

Add failing env-gated smoke definitions for CLI, HTTP, and browser live-provider validation so the completed runtime can be checked against a real OpenRouter model without making normal CI nondeterministic.

## Execution Context

**Task Number**: 009 of 009  
**Phase**: Verification  
**Prerequisites**: browser transcript and compatibility reuse exist

## BDD Scenario

```gherkin
Scenario: Live provider/browser smoke validates CLI, HTTP, and composer paths against one runtime
  Given OpenRouter credentials are available
  When a maintainer runs the live smoke commands
  Then MatrixClaw exercises the CLI smoke path against a real provider
  And exercises the HTTP agent endpoint against the same runtime
  And exercises the browser composer path against the same runtime
```

**Spec Source**: scope-specific scenario derived from current product validation requirements

## Files to Modify/Create

- Create: `ui/tests/live-llm.spec.ts`
- Modify: `crates/app-host/src/llm_smoke.rs`
- Modify: `ui/package.json`
- Optionally Create: `scripts/verify-live-runtime.sh`

## Steps

### Step 1: Define the live smoke contract
- Lock the environment variables, model selection, and artifact paths used by live validation.
- Keep the tests opt-in so local/CI runs are not forced to require secrets.

### Step 2: Write the Red test
- Define CLI, HTTP, and browser smoke commands and expected sentinel behavior.
- Assert the harness expects one shared runtime path, not three independent shortcuts.

### Step 3: Keep the task Red-only
- Do not finish the live harness implementation in this task.

## Verification Commands

```bash
pnpm --dir ui check
cargo test -p matrixclaw-app-host
```

## Success Criteria

- Live validation expectations are explicit and documented in executable form.
- The smoke harness is env-gated and safe for normal development.
- The task fails only because the final integrated harness is not yet implemented.
