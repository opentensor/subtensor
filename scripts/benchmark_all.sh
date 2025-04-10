#!/usr/bin/env bash
set -e

pallets=(
  "pallet_subtensor"
  "pallet_collective"
  "pallet_commitments"
  "pallet_drand"
  "pallet_admin_utils"
)

# 1) Build/Refresh the Chain Specs
echo "*** Building all chain specs with your existing script ***"
./scripts/build_all_chainspecs.sh

# 2) Build the Node in Production Mode with Benchmarking Features
echo "*** Building node-subtensor with 'runtime-benchmarks' ***"
cargo build \
  --profile production \
  --package node-subtensor \
  --bin node-subtensor \
  --features "runtime-benchmarks,try-runtime,pow-faucet"

CHAIN_SPEC="chainspecs/raw_spec_finney.json"

# 3) Benchmark the Desired Pallets Using the Updated Chain Spec
echo "*** Starting benchmarks using $CHAIN_SPEC ***"
for pallet in "${pallets[@]}"; do
  echo "======================================================"
  echo "Benchmarking $pallet..."
  echo "======================================================"

  ./target/production/node-subtensor \
    benchmark pallet \
    --chain "$CHAIN_SPEC" \
    --wasm-execution=compiled \
    --pallet "$pallet" \
    --extrinsic '*' \
    --steps 50 \
    --repeat 5 \
    --output "pallets/$pallet/src/weights.rs" \
    --template ./.maintain/frame-weight-template.hbs
done

echo "*** All benchmarks completed successfully ***"