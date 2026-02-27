#!/bin/bash

# If binaries are compiled in CI then skip this script
if [ -n "$BUILT_IN_CI" ]; then
  echo "[*] BUILT_IN_CI is set to '$BUILT_IN_CI'. Skipping script..."
  exit 0
fi

# Check if `--no-purge` passed as a parameter
NO_PURGE=0

# Check if `--build-only` passed as parameter
BUILD_ONLY=0

CHAIN="local"

for arg in "$@"; do
  if [ "$arg" = "--no-purge" ]; then
    NO_PURGE=1
  elif [ "$arg" = "--build-only" ]; then
    BUILD_ONLY=1
  # start local network with 5 nodes
  elif [ "$arg" = "--local5" ]; then
    CHAIN="local5"
  fi
done

# Determine the directory this script resides in. This allows invoking it from any location.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# The base directory of the subtensor project
BASE_DIR="$SCRIPT_DIR/.."

# Get the value of fast_runtime from the first argument
fast_runtime=${1:-"True"}

# Define the target directory for compilation
if [ "$fast_runtime" == "False" ]; then
  # Block of code to execute if fast_runtime is False
  echo "fast_runtime is Off"
  : "${BUILD_BINARY:=1}"
  : "${FEATURES:="pow-faucet"}"
  BUILD_DIR="$BASE_DIR/target/non-fast-runtime"
else
  # Block of code to execute if fast_runtime is not False
  echo "fast_runtime is On"
  : "${BUILD_BINARY:=1}"
  : "${FEATURES:="pow-faucet fast-runtime"}"
  BUILD_DIR="$BASE_DIR/target/fast-runtime"
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

if [[ "$BUILD_BINARY" == "1" ]]; then
  echo "*** Building substrate binary..."

  BUILD_CMD=(
    cargo build
    --workspace
    --profile=release
    --features "$FEATURES"
    --manifest-path "$BASE_DIR/Cargo.toml"
  )

  if [[ -n "$CARGO_BUILD_TARGET" ]]; then
    echo "[+] Cross-compiling for target: $CARGO_BUILD_TARGET"
    BUILD_CMD+=(--target "$CARGO_BUILD_TARGET")
  else
    echo "[+] Building for host architecture"
  fi

  CARGO_TARGET_DIR="$BUILD_DIR" "${BUILD_CMD[@]}"
  echo "*** Binary compiled"
fi

echo "*** Building chainspec..."
"$BUILD_DIR/release/node-subtensor" build-spec --disable-default-bootnode --raw --chain "$CHAIN" >"$FULL_PATH"
echo "*** Chainspec built and output to file"

NODES=("dave" "eve" "ferdie" "one" "two")

# Generate node keys
for i in "${NODES[@]}"; do
  echo /tmp/$i
  "$BUILD_DIR/release/node-subtensor" key generate-node-key --chain="$FULL_PATH" --base-path /tmp/$i
done


if [ $NO_PURGE -eq 1 ]; then
  echo "*** Purging previous state skipped..."
else
  echo "*** Purging previous state..."
  for i in "${NODES[@]}"; do
    "$BUILD_DIR/release/node-subtensor" purge-chain -y --base-path /tmp/$i --chain="$FULL_PATH" >/dev/null 2>&1
  done
  echo "*** Previous chainstate purged"
fi

command_base=(
    "$BUILD_DIR/release/node-subtensor"
    --chain="$FULL_PATH"
    --validator
    --rpc-cors=all
    --allow-private-ipv4
    --discover-local
    --unsafe-force-node-key-generation
    --unsafe-rpc-external 
  )

if [ $BUILD_ONLY -eq 0 ]; then
  echo "*** Starting localnet nodes..."

  dave_start=("${command_base[@]}" --dave --base-path /tmp/dave --port 30331 --rpc-port 9941) 
  eve_start=("${command_base[@]}" --eve --base-path /tmp/eve --port 30332 --rpc-port 9942)
  ferdie_start=("${command_base[@]}" --ferdie --base-path /tmp/ferdie --port 30333 --rpc-port 9943)
  one_start=("${command_base[@]}" --one --base-path /tmp/one --port 30334 --rpc-port 9944)
  two_start=("${command_base[@]}" --two --base-path /tmp/two --port 30335 --rpc-port 9945)

  trap 'pkill -P $$' EXIT SIGINT SIGTERM

  if [ "$CHAIN" = "local5" ]; then
    (
      ("${dave_start[@]}" 2>&1) &
      ("${eve_start[@]}" 2>&1) &
      ("${ferdie_start[@]}" 2>&1) &
      ("${one_start[@]}" 2>&1) &
      ("${two_start[@]}" 2>&1)
      wait
    )
  fi
  (
    ("${one_start[@]}" 2>&1) &
    ("${two_start[@]}" 2>&1)
    wait
  )
fi
