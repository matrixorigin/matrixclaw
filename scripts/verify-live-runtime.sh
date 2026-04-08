#!/usr/bin/env bash
set -euo pipefail

MODEL="${MATRIXCLAW_LIVE_MODEL:-moonshotai/kimi-k2.5}"
BASE_URL="${MATRIXCLAW_BASE_URL:-http://127.0.0.1:38495}"
HTTP_SENTINEL="${MATRIXCLAW_HTTP_SENTINEL:-MATRIXCLAW_ENDPOINT_OK}"
UI_SENTINEL="${MATRIXCLAW_LIVE_SENTINEL:-MATRIXCLAW_UI_E2E_OK}"
SERVER_PID=""
TEMP_HOME=""

cleanup() {
  if [[ -n "${SERVER_PID}" ]]; then
    kill "${SERVER_PID}" >/dev/null 2>&1 || true
  fi
  if [[ -n "${TEMP_HOME}" ]]; then
    rm -rf "${TEMP_HOME}" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT

if [[ -z "${OPENROUTER_API_KEY:-}" ]]; then
  echo "OPENROUTER_API_KEY must be set" >&2
  exit 1
fi

if ! curl -fsS "${BASE_URL}/healthz" >/dev/null 2>&1; then
  TEMP_HOME="$(mktemp -d)"
  HOME="${TEMP_HOME}" MATRIXCLAW_LLM_MODEL="${MODEL}" \
    target/debug/zstar serve --fixture demo >/tmp/zstar-live-runtime.log 2>&1 &
  SERVER_PID="$!"

  for _ in $(seq 1 30); do
    if curl -fsS "${BASE_URL}/healthz" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done

  curl -fsS "${BASE_URL}/healthz" >/dev/null
fi

target/debug/zstar llm-smoke --model "${MODEL}"

HTTP_RESPONSE="$(
  curl -fsS \
    -X POST \
    "${BASE_URL}/api/agent/run" \
    -H 'content-type: application/json' \
    -d "{\"prompt\":\"Reply with exactly ${HTTP_SENTINEL} and nothing else.\"}"
)"

printf '%s' "${HTTP_RESPONSE}" | rg -q "\"final_message\": \"${HTTP_SENTINEL}\""

(
  cd ui
  MATRIXCLAW_BASE_URL="${BASE_URL}" \
  MATRIXCLAW_LIVE_E2E=1 \
  MATRIXCLAW_LIVE_SENTINEL="${UI_SENTINEL}" \
    bunx playwright test tests/live-llm.spec.ts
)
