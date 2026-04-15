#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BASE_DIR="${CLONE_BASE_DIR:-$REPO_ROOT/target/rate-limits-test}"
CHAIN_SPEC="${PATCHED_CHAIN_SPEC:-$BASE_DIR/patched-finney.json}"
RUN_BASE_PATH="${CLONE_RUN_BASE_PATH:-$BASE_DIR/run/alice}"
BINARY_PATH="${BINARY_PATH:-$REPO_ROOT/target/release/node-subtensor}"
NODE_PORT="${CLONE_NODE_PORT:-30633}"
RPC_PORT="${CLONE_NODE_RPC_PORT:-9964}"

exec "$BINARY_PATH" \
  --alice \
  --chain "$CHAIN_SPEC" \
  --base-path "$RUN_BASE_PATH" \
  --database paritydb \
  --force-authoring \
  --port "$NODE_PORT" \
  --rpc-port "$RPC_PORT" \
  --rpc-cors=all \
  --rpc-methods=unsafe \
  --unsafe-rpc-external \
  --unsafe-force-node-key-generation \
  --no-telemetry \
  --no-prometheus \
  --validator
