#!/bin/sh

# List of pallets you want to benchmark
pallets=("admin-utils", "collective", "commitments", "registry", "subtensor")

# Chain spec and output directory
chain_spec="dev"  # or your specific chain spec

for pallet in "${pallets[@]}"
do
  echo "Benchmarking $pallet..."
  cargo run --profile=production --features=runtime-benchmarks -- benchmark pallet \
    --chain $chain_spec \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet $pallet \
    --extrinsic '*' \
    --steps 50 \
    --repeat 20 \
    --output "pallets/$pallet/src/$pallet.rs" \
    --template ./.maintain/frame-weight-template.hbs  # Adjust this path to your template file
done

echo "All pallets have been benchmarked and weights updated."
