#!/bin/bash
set -e

MANIFEST="../Cargo.toml"
BINARY="../target/release/node-subtensor"

echo "==> Building node-subtensor..."
pnpm build-node

echo "==> Starting dev node..."
"$BINARY" --one --dev 2>&1 &
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
    echo "==> ERROR: Node failed to start within ${TIMEOUT}s"
    exit 1
  fi
done

echo "==> Generating papi types..."
pnpm generate-types

echo "==> Done."
