#!/usr/bin/env bash
set -euo pipefail

# -------------------------------
# Usage
# -------------------------------
# ./run_localnet.sh <BUILD_DIRECTORY> <SPEC_PATH>
# Example:
# ./run_localnet.sh ./subtensor/target/release ./.bdk-env/specs/subtensor.json
# -------------------------------

BUILD_DIR="${1:?Build directory missing}"
SPEC_PATH="${2:?Spec path missing}"

BIN="${BUILD_DIR}/node-subtensor"

# -------------------------------
# Purge previous chain state
# -------------------------------
echo "*** Purging previous state..."

for NODE in alice bob charlie; do
  "$BIN" purge-chain -y --base-path "/tmp/$NODE" --chain="$SPEC_PATH" >/dev/null 2>&1
done

echo "*** Previous chain state purged"

# -------------------------------
# Define nodes
# -------------------------------
ALICE_BASE="/tmp/alice"
BOB_BASE="/tmp/bob"
CHARLIE_BASE="/tmp/charlie"

alice_start=(
  "$BIN"
  --base-path "$ALICE_BASE"
  --chain="$SPEC_PATH"
  --keystore-path="./.bdk-env/secret/keystore/subtensor-node-alice"
  --node-key-file="./.bdk-env/secret/node/subtensor-node-alice"
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
  --base-path "$BOB_BASE"
  --chain="$SPEC_PATH"
  --keystore-path="./.bdk-env/secret/keystore/subtensor-node-bob"
  --node-key-file="./.bdk-env/secret/node/subtensor-node-bob"
  --port 30335
  --rpc-port 9935
  --validator
  --allow-private-ipv4
  --discover-local
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWMJ5Gmn2SPfx2TEFfvido1X8xhUZUnC2MbD2yTwKPQak8
)

charlie_start=(
  "$BIN"
  --base-path "$CHARLIE_BASE"
  --chain="$SPEC_PATH"
  --keystore-path="./.bdk-env/secret/keystore/subtensor-node-charlie"
  --node-key-file="./.bdk-env/secret/node/subtensor-node-charlie"
  --port 30336
  --rpc-port 9936
  --validator
  --allow-private-ipv4
  --discover-local
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWMJ5Gmn2SPfx2TEFfvido1X8xhUZUnC2MbD2yTwKPQak8
)

# -------------------------------
# Start nodes in background
# -------------------------------

echo "*** Starting localnet nodes (Alice/Bob/Charlie)..."
echo "Press Ctrl+C to terminate"

# trap ensures all background nodes are killed if script is interrupted
trap 'kill 0' SIGINT

# Run nodes concurrently
("${alice_start[@]}" 2>&1 &)
("${bob_start[@]}" 2>&1 &)
("${charlie_start[@]}" 2>&1 &)

# Keep script alive to allow external checks / JS tests
# CI runner will terminate at job end
sleep infinity