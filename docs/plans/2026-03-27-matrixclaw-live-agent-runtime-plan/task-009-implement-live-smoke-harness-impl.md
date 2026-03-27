# Task 009: Implement live smoke harness

**depends-on**: task-009-write-live-smoke-harness-test-test

## Description

Implement the opt-in live smoke harness for CLI, HTTP, and browser composer validation against one real provider-backed runtime.

## Execution Context

**Task Number**: 009 of 009  
**Phase**: Verification  
**Prerequisites**: failing live smoke harness task exists

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

- Modify: `crates/app-host/src/llm_smoke.rs`
- Modify: `crates/app-host/src/http/agent_api.rs`
- Modify: `ui/tests/live-llm.spec.ts`
- Modify: `ui/tests/ui-smoke.spec.ts`
- Optionally Create: `scripts/verify-live-runtime.sh`

## Steps

### Step 1: Re-run the Red harness
- Confirm the live harness definitions still fail or remain incomplete before implementation.

### Step 2: Implement the Green path
- Finalize the env-gated CLI smoke command.
- Finalize the HTTP endpoint smoke command.
- Finalize the browser composer smoke flow and screenshot artifact generation.
- Use `moonshotai/kimi-k2.5` as the default live validation model unless overridden.

### Step 3: Verify
- Run normal non-live verification first.
- Run the env-gated live smoke commands with a real OpenRouter key.
- Preserve browser artifacts under `output/playwright/`.

## Verification Commands

```bash
cargo test -p matrixclaw-app-host
pnpm --dir ui check
pnpm --dir ui build
target/debug/matrixclaw llm-smoke --model moonshotai/kimi-k2.5
curl -sS -X POST http://127.0.0.1:38495/api/agent/run -H 'content-type: application/json' -d '{"prompt":"Reply with exactly MATRIXCLAW_ENDPOINT_OK and nothing else."}'
MATRIXCLAW_BASE_URL=http://127.0.0.1:38495 MATRIXCLAW_LIVE_E2E=1 MATRIXCLAW_LIVE_SENTINEL=MATRIXCLAW_UI_E2E_OK pnpm --dir ui exec playwright test ui/tests/live-llm.spec.ts
```

## Success Criteria

- One maintainer-facing harness can validate CLI, HTTP, and browser live paths.
- Live smoke remains opt-in and artifact-producing.
- The verified output proves one shared runtime rather than separate shortcuts.
