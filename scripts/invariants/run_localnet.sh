#!/usr/bin/env bash
set -euo pipefail

exec > >(awk '{ print strftime("[%Y-%m-%d %H:%M:%S]"), $0 }') 2>&1

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

# ----------------------------------------
# Derived paths
# ----------------------------------------

SPEC_PATH="$BDK_ENV_DIR/specs/subtensor.json"
SECRET_DIR="$BDK_ENV_DIR/secret"

KEYSTORE_DIR="$SECRET_DIR/keystore"
NODE_KEY_DIR="$SECRET_DIR/node"

# ----------------------------------------
# Validation
# ----------------------------------------

[[ -x "$BIN" ]] || { echo "❌ node-subtensor not found: $BIN"; exit 1; }
[[ -f "$SPEC_PATH" ]] || { echo "❌ spec not found: $SPEC_PATH"; exit 1; }

for d in "$KEYSTORE_DIR" "$NODE_KEY_DIR"; do
  [[ -d "$d" ]] || { echo "❌ missing directory: $d"; exit 1; }
done

# ----------------------------------------
# Node commands
# ----------------------------------------

echo "🚀 Starting localnet nodes (Alice / Bob / Charlie)..."

alice_start=(
  "$BIN"
  --base-path /tmp/alice
  --chain="$SPEC_PATH"
  --keystore-path="$KEYSTORE_DIR/subtensor-node-alice"
  --node-key-file="$NODE_KEY_DIR/subtensor-node-alice"
  --port 30334
  --rpc-port 9946
  --validator
  --rpc-cors=all
  --rpc-external
  --unsafe-rpc-external
  --rpc-methods=unsafe
  --allow-private-ipv4
  --discover-local
)

bob_start=(
  "$BIN"
  --base-path /tmp/bob
  --chain="$SPEC_PATH"
  --keystore-path="$KEYSTORE_DIR/subtensor-node-bob"
  --node-key-file="$NODE_KEY_DIR/subtensor-node-bob"
  --port 30335
  --rpc-port 9935
  --validator
  --allow-private-ipv4
  --discover-local
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWMJ5Gmn2SPfx2TEFfvido1X8xhUZUnC2MbD2yTwKPQak8
)

charlie_start=(
  "$BIN"
  --base-path /tmp/charlie
  --chain="$SPEC_PATH"
  --keystore-path="$KEYSTORE_DIR/subtensor-node-charlie"
  --node-key-file="$NODE_KEY_DIR/subtensor-node-charlie"
  --port 30336
  --rpc-port 9936
  --validator
  --allow-private-ipv4
  --discover-local
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWMJ5Gmn2SPfx2TEFfvido1X8xhUZUnC2MbD2yTwKPQak8
)

# ----------------------------------------
# Launch (background, detached)
# ----------------------------------------

#("${alice_start[@]}"   > /tmp/alice.log   2>&1 &)
#("${bob_start[@]}"     > /tmp/bob.log     2>&1 &)
#("${charlie_start[@]}" > /tmp/charlie.log 2>&1 &)

#echo "✅ Localnet started"
#echo "   Logs:"
#echo "     /tmp/alice.log"
#echo "     /tmp/bob.log"
#echo "     /tmp/charlie.log"

# ----------------------------------------
# Exit so CI can continue
# ----------------------------------------

#exit 0
"${alice_start[@]}"
"${bob_start[@]}"
"${charlie_start[@]}"