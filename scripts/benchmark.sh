#!/bin/bash
set -e

# 1) Clean and rebuild the node with runtime-benchmarks.
cargo build \
  --profile production \
  --package node-subtensor \
  --bin node-subtensor \
  --all-features

# 2) Locate your freshly-built runtime Wasm blob.
RUNTIME_WASM=./target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm

# 3) Run the benchmark using the runtime blob and genesis builder,
#    and explicitly override the default preset by passing an empty preset.
./target/production/node-subtensor benchmark pallet \
  --runtime "$RUNTIME_WASM" \
  --genesis-builder=runtime \
  --genesis-builder-preset=benchmark  \
  --wasm-execution=compiled \
  --pallet=pallet_subtensor \
  --extrinsic=benchmark_register \
  --steps 50 \
  --repeat 5 \
  --output benchmarking.txt
