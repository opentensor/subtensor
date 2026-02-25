#!/usr/bin/env bash
set -e

EXTRINSIC="${1:-register}"

cargo build \
  --profile production \
  -p node-subtensor \
  --features runtime-benchmarks

RUNTIME_WASM=./target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm

./target/production/node-subtensor benchmark pallet \
  --runtime "$RUNTIME_WASM" \
  --genesis-builder=runtime \
  --genesis-builder-preset=benchmark \
  --wasm-execution=compiled \
  --pallet=pallet_subtensor \
  --extrinsic="$dissolve_network" \
  --steps 50 \
  --repeat 5 \
