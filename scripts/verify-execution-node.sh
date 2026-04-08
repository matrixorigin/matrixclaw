#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

echo "Running execution node contract test"
cargo test -p zstar-app-host --test execution_node_contract execution_node_contract -- --exact

echo "Running execution node routing test"
cargo test -p zstar-app-host --test execution_node_routing execution_node_routing -- --exact

echo "Running runtime execution node integration test"
cargo test -p zstar-app-host --test runtime_execution_node_integration runtime_execution_node_integration -- --exact

echo "Running execution node smoke harness"
cargo test -p zstar-app-host --test execution_node_smoke_harness execution_node_smoke_harness -- --exact
