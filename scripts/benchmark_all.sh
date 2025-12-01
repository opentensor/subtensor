#!/usr/bin/env bash
set -e

pallets=(
  "pallet_subtensor"
  "pallet_commitments"
  "pallet_drand"
  "pallet_admin_utils"
  "pallet_shield"
)

RUNTIME_WASM=./target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm

cargo build \
  --profile production \
  -p node-subtensor \
  --features runtime-benchmarks

for pallet in "${pallets[@]}"; do
  echo "--------------------------------------------------------"
  echo " Benchmarking all extrinsics for $pallet..."
  echo "--------------------------------------------------------"

  ./target/production/node-subtensor benchmark pallet \
    --runtime "$RUNTIME_WASM" \
    --genesis-builder=runtime \
    --genesis-builder-preset=benchmark \
    --wasm-execution=compiled \
    --pallet "$pallet" \
    --extrinsic "*" \
    --steps 50 \
    --repeat 5
done