#!/bin/bash
#
# Build the node binary and (re)generate polkadot-api type descriptors.
# Installs polkadot-api globally for the CLI and type resolution.
# Run this whenever the runtime changes to keep descriptors in sync.
#
# Usage:
#   ./bootstrap_types.sh              # build + generate types
#   ./bootstrap_types.sh --skip-build # generate types only (binary must exist)
#
set -e

BINARY="../target/release/node-subtensor"
NODE_LOG="/tmp/e2e-bootstrap-node.log"

if [ "$1" != "--skip-build" ]; then
  echo "==> Building node-subtensor..."
  pnpm build-node
fi

echo "==> Starting dev node (logs at $NODE_LOG)..."
"$BINARY" --one --dev &>"$NODE_LOG" &
NODE_PID=$!
trap "kill $NODE_PID 2>/dev/null; wait $NODE_PID 2>/dev/null" EXIT

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

echo "==> Installing polkadot-api globally..."
pnpm add -g polkadot-api

echo "==> Generating papi types..."
pnpm generate-types

echo "==> Done."
