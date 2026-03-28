#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

echo "Running browser/Matrix shared-session smoke"
cargo test -p matrixclaw-app-host --test browser_matrix_session_reuse browser_matrix_session_reuse -- --exact

echo "Running Matrix gateway contract and delivery checks"
cargo test -p matrixclaw-app-host --test gateway_adapter_contract gateway_adapter_contract -- --exact
cargo test -p matrixclaw-app-host --test matrix_ingress_normalization matrix_ingress_normalization -- --exact
cargo test -p matrixclaw-app-host --test matrix_streamed_delivery matrix_streamed_delivery -- --exact
cargo test -p matrixclaw-app-host --test matrix_gateway_transport_runner matrix_gateway_transport_runner_streams_deliveries -- --exact
cargo test -p matrixclaw-app-host --test matrix_gateway_transport_runner matrix_gateway_transport_runner_records_and_flushes_retries -- --exact
cargo test -p matrixclaw-app-host --test gateway_dedupe_retry_boundary gateway_dedupe_retry_boundary -- --exact
cargo test -p matrixclaw-app-host --test optional_matrix_gateway_startup optional_matrix_gateway_startup -- --exact

echo "Running full app-host verification"
cargo test -p matrixclaw-app-host
