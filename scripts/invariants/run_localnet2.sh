#!/usr/bin/env bash
set -euo pipefail

# ----------------------------------------
# Args
# ----------------------------------------

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <build_dir> <bdk_env_dir>"
  exit 1
fi

BUILD_DIR="$(realpath "$1")"
BDK_ENV_DIR="$(realpath "$2")"

BIN="$BUILD_DIR/node-subtensor"

[[ -x "$BIN" ]] || { echo "❌ node-subtensor not found: $BIN"; exit 1; }

echo "🚀 Starting localnet nodes (Alice / Bob / Charlie)..."

alice_start=(
  "$BIN"
  --dev
  --port 30334
  --rpc-port 9946
)

("${alice_start[@]}"   > /tmp/alice.log   2>&1 &)

echo "✅ Localnet started"
echo "   Logs:"
echo "     /tmp/alice.log"

exit 0