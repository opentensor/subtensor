#!/usr/bin/env bash
set -euo pipefail

# -------------------------------
# Configurable via environment
# -------------------------------

RPC_HOST="${RPC_HOST:-127.0.0.1}"
RPC_PORT="${RPC_PORT:-9946}"
RPC_URL="http://${RPC_HOST}:${RPC_PORT}"

MAX_RETRIES="${MAX_RETRIES:-30}"
SLEEP_INTERVAL="${SLEEP_INTERVAL:-2}"  # seconds

# -------------------------------
# Wait for RPC availability
# -------------------------------

echo "Waiting for node RPC at $RPC_URL..."

ps aux | grep node-subtensor

for ((i=1; i<=MAX_RETRIES; i++)); do
  if curl -sf -H "Content-Type: application/json" \
      --data '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}' \
      "$RPC_URL" > /tmp/health.json; then

    PEERS=$(jq '.result.peers // 0' /tmp/health.json)
    SYNCING=$(jq '.result.isSyncing // true' /tmp/health.json)

    echo "[Attempt $i/$MAX_RETRIES] Peers=$PEERS, Syncing=$SYNCING"

    if [[ "$PEERS" -gt 0 ]]; then
      echo "✅ Node has peers connected"
      break
    fi
  fi

  sleep "$SLEEP_INTERVAL"
done

# Final check if peers never connected
PEERS=$(jq '.result.peers // 0' /tmp/health.json || echo 0)
if [[ "$PEERS" -le 0 ]]; then
  echo "❌ Node failed to connect to any peers after $MAX_RETRIES retries"
  exit 1
fi

# -------------------------------
# Check chain height
# -------------------------------

echo "Checking chain height..."

HEADER_JSON=$(curl -sf -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"chain_getHeader","params":[],"id":1}' \
  "$RPC_URL")

HEIGHT_HEX=$(echo "$HEADER_JSON" | jq -r '.result.number')
HEIGHT=$((HEIGHT_HEX))

if [[ "$HEIGHT" -le 0 ]]; then
  echo "❌ Chain is not progressing. Height=$HEIGHT"
  exit 1
fi

echo "✅ Chain is producing blocks. Current height=$HEIGHT"