#!/bin/sh
set -ex

# List of pallets you want to benchmark
pallets=("pallet_subtensor" "pallet_collective" "pallet_commitments" "pallet_registry" "pallet_admin_utils")

# Chain spec and output directory
chain_spec="finney"  # or your specific chain spec

for pallet in "${pallets[@]}"
do
  echo "Benchmarking $pallet..."
  cargo run --profile=production --features=runtime-benchmarks,try-runtime --bin node-subtensor -- benchmark pallet \
    --chain $chain_spec \
    --wasm-execution=compiled \
    --pallet $pallet \
    --extrinsic '*' \
    --steps 50 \
    --repeat 5 \
    --output "pallets/$pallet/src/weights.rs" \
    --template ./.maintain/frame-weight-template.hbs  # Adjust this path to your template file
done

echo "All pallets have been benchmarked and weights updated."
