#!/bin/bash

# Check if `--no-purge` passed as a parameter
NO_PURGE=0

# Check if `--build-only` passed as parameter
BUILD_ONLY=0

for arg in "$@"; do
  if [ "$arg" = "--no-purge" ]; then
    NO_PURGE=1
  elif [ "$arg" = "--build-only" ]; then
    BUILD_ONLY=1
  fi
done

# Determine the directory this script resides in. This allows invoking it from any location.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# The base directory of the subtensor project
BASE_DIR="$SCRIPT_DIR/.."

# Get the value of fast_blocks from the first argument
fast_blocks=${1:-"True"}

# Define the target directory for compilation
if [ "$fast_blocks" == "False" ]; then
  # Block of code to execute if fast_blocks is False
  echo "fast_blocks is Off"
  : "${CHAIN:=local}"
  : "${BUILD_BINARY:=1}"
  : "${FEATURES:="pow-faucet"}"
  BUILD_DIR="$BASE_DIR/target/non-fast-blocks"
else
  # Block of code to execute if fast_blocks is not False
  echo "fast_blocks is On"
  : "${CHAIN:=local}"
  : "${BUILD_BINARY:=1}"
  : "${FEATURES:="pow-faucet fast-blocks"}"
  BUILD_DIR="$BASE_DIR/target/fast-blocks"
fi

# Ensure the build directory exists
mkdir -p "$BUILD_DIR"

SPEC_PATH="${SCRIPT_DIR}/specs/"
FULL_PATH="$SPEC_PATH$CHAIN.json"

# Kill any existing nodes which may have not exited correctly after a previous run.
pkill -9 'node-subtensor'

if [ ! -d "$SPEC_PATH" ]; then
  echo "*** Creating directory ${SPEC_PATH}..."
  mkdir -p "$SPEC_PATH"
fi

if [[ $BUILD_BINARY == "1" ]]; then
  echo "*** Building substrate binary..."
  CARGO_TARGET_DIR="$BUILD_DIR" cargo build --workspace --profile=release --features "$FEATURES" --manifest-path "$BASE_DIR/Cargo.toml"
  echo "*** Binary compiled"
fi

echo "*** Building chainspec..."
"$BUILD_DIR/release/node-subtensor" build-spec --disable-default-bootnode --raw --chain $CHAIN >$FULL_PATH
echo "*** Chainspec built and output to file"

# Generate node keys
"$BUILD_DIR/release/node-subtensor" key generate-node-key --chain="$FULL_PATH" --base-path /tmp/alice
"$BUILD_DIR/release/node-subtensor" key generate-node-key --chain="$FULL_PATH" --base-path /tmp/bob

if [ $NO_PURGE -eq 1 ]; then
  echo "*** Purging previous state skipped..."
else
  echo "*** Purging previous state..."
  "$BUILD_DIR/release/node-subtensor" purge-chain -y --base-path /tmp/bob --chain="$FULL_PATH" >/dev/null 2>&1
  "$BUILD_DIR/release/node-subtensor" purge-chain -y --base-path /tmp/alice --chain="$FULL_PATH" >/dev/null 2>&1
  echo "*** Previous chainstate purged"
fi

if [ $BUILD_ONLY -eq 0 ]; then
  echo "*** Starting localnet nodes..."
  alice_start=(
    "$BUILD_DIR/release/node-subtensor"
    --base-path /tmp/alice
    --chain="$FULL_PATH"
    --alice
    --port 30334
    --rpc-port 9944
    --validator
    --rpc-cors=all
    --allow-private-ipv4
    --discover-local
    --unsafe-force-node-key-generation
  )

  bob_start=(
    "$BUILD_DIR/release/node-subtensor"
    --base-path /tmp/bob
    --chain="$FULL_PATH"
    --bob
    --port 30335
    --rpc-port 9945
    --validator
    --rpc-cors=all
    --allow-private-ipv4
    --discover-local
    --unsafe-force-node-key-generation
  )

  trap 'pkill -P $$' EXIT SIGINT SIGTERM

  (
    ("${alice_start[@]}" 2>&1) &
    ("${bob_start[@]}" 2>&1)
    wait
  )
fi