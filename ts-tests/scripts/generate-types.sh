#!/bin/bash
#
# (Re)generate polkadot-api type descriptors using a running node.
# Checks that the node binary exists before running.
# Generates types only if they are missing or empty.
#
# Usage:
#   ./generate-types.sh
#
set -e

BASE_DIR="./tmp"
mkdir -p "$BASE_DIR"

BINARY="${BINARY_PATH:-../target/release/node-subtensor}"
NODE_LOG="${BASE_DIR}/node.log"

if [ ! -f "$BINARY" ]; then
  echo "ERROR: Node binary not found at $BINARY"
  echo "Please build it first, e.g.: cargo build --release -p node-subtensor"
  exit 1
fi

DESCRIPTORS_DIR="./.papi/descriptors"
GENERATE_TYPES=false
if [ ! -d "$DESCRIPTORS_DIR" ] || [ -z "$(ls -A "$DESCRIPTORS_DIR" 2>/dev/null)" ]; then
  echo "==> Type descriptors not found or empty, will generate..."
  GENERATE_TYPES=true
else
  echo "==> Type descriptors already exist, skipping generation."
fi

if [ "$GENERATE_TYPES" = true ]; then
  echo "==> Starting dev node (logs at $NODE_LOG)..."
  "$BINARY" --one --dev &>"$NODE_LOG" &
  NODE_PID=$!
  trap "kill $NODE_PID 2>/dev/null; wait $NODE_PID 2>/dev/null || true; exit 0" EXIT

  TIMEOUT=60
  ELAPSED=0
  echo "==> Waiting for node to be ready (timeout: ${TIMEOUT}s)..."
  until curl -sf -o /dev/null \
    -H "Content-Type: application/json" \
    -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
    http://localhost:9944; do
    sleep 1
    ELAPSED=$((ELAPSED + 1))
    if [ "$ELAPSED" -ge "$TIMEOUT" ]; then
      echo "ERROR: Node failed to start within ${TIMEOUT}s. Check $NODE_LOG"
      exit 1
    fi
  done

  echo "==> Generating papi types..."
  pnpm generate-types

  echo "==> Done generating types."
  exit 0
else
  echo "==> Types are up-to-date, nothing to do."
fi
