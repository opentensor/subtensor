#!/bin/bash

# Determine the directory this script resides in. This allows invoking it from any location.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# The base directory of the subtensor project
BASE_DIR="$SCRIPT_DIR/.."

: "${CHAIN:=greg}"
: "${BUILD_BINARY:=1}"
: "${FEATURES:="pow-faucet runtime-benchmarks fast-blocks"}"

SPEC_PATH="${SCRIPT_DIR}/specs/"
FULL_PATH="$SPEC_PATH$CHAIN.json"

# Kill any existing nodes which may have not exited correctly after a previous
# run.
pkill -9 'node-subtensor'

if [ ! -d "$SPEC_PATH" ]; then
  echo "*** Creating directory ${SPEC_PATH}..."
  mkdir $SPEC_PATH
fi

if [[ $BUILD_BINARY == "1" ]]; then
  echo "*** Building substrate binary..."
  cargo build --release --features "$FEATURES" --manifest-path "$BASE_DIR/Cargo.toml"
  echo "*** Binary compiled"
fi

echo "*** Building chainspec..."
"$BASE_DIR/target/release/node-subtensor" build-spec --disable-default-bootnode --raw --chain $CHAIN >$FULL_PATH
echo "*** Chainspec built and output to file"

echo "*** Purging previous state..."
"$BASE_DIR/target/release/node-subtensor" purge-chain -y --base-path /tmp/validator1 --chain="$FULL_PATH" >/dev/null 2>&1
"$BASE_DIR/target/release/node-subtensor" purge-chain -y --base-path /tmp/validator2 --chain="$FULL_PATH" >/dev/null 2>&1
echo "*** Previous chainstate purged"

echo "*** Starting localnet nodes..."
export RUST_LOG=subtensor=trace
alice_start=(
  "$BASE_DIR/target/release/node-subtensor"
  --base-path /tmp/validator1
  --chain="$FULL_PATH"
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
  "$BASE_DIR"/target/release/node-subtensor
  --base-path /tmp/validator2
  --chain="$FULL_PATH"
  --port 30335
  --rpc-port 9945
  --validator
  --allow-private-ipv4
  --discover-local
)

insert_validator_1_aura_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
  --base-path /tmp/validator1 
  --chain="$FULL_PATH"
  --scheme Sr25519 \
  --suri "subject one mention gown inside fluid recycle essence hair robot ozone point" \
  --key-type aura
)

insert_validator_1_gran_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
  --base-path /tmp/validator1 
  --chain="$FULL_PATH"
  --scheme Ed25519 \
  --suri "subject one mention gown inside fluid recycle essence hair robot ozone point" \
  --key-type gran
)

insert_validator_2_aura_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
  --base-path /tmp/validator2 
  --chain="$FULL_PATH"
  --scheme Sr25519 
  --suri "coach force devote mule oppose awesome type pelican bone concert tiger reduce" \
  --key-type aura
)

insert_validator_2_gran_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
  --base-path /tmp/validator2 
  --chain="$FULL_PATH"
  --scheme Ed25519 
  --suri "coach force devote mule oppose awesome type pelican bone concert tiger reduce" \
  --key-type gran
)

trap 'pkill -P $$' EXIT SIGINT SIGTERM

(
  ("${alice_start[@]}" 2>&1) &
  ("${bob_start[@]}" 2>&1) &
  ("${insert_validator_1_aura_key[@]}" 2>&1) &
  ("${insert_validator_1_gran_key[@]}" 2>&1) &
  ("${insert_validator_2_aura_key[@]}" 2>&1) &
  ("${insert_validator_2_gran_key[@]}" 2>&1) &

  wait
)