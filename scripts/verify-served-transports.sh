#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

echo "Running cross-transport session reuse smoke"
cargo test -p matrixclaw-app-host cross_transport_session_reuse -- --exact

echo "Running served transport HTTP and WebSocket checks"
cargo test -p matrixclaw-app-host openclaw_http_over_server -- --exact
cargo test -p matrixclaw-app-host openclaw_websocket_over_server -- --exact
cargo test -p matrixclaw-app-host openclaw_streaming_parity -- --exact

echo "Running broader app-host transport verification"
cargo test -p matrixclaw-app-host
