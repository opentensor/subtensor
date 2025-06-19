#!/bin/bash

cargo build --profile production --features runtime-benchmarks
./target/production/node-subtensor benchmark pallet \
  --chain=local \
  --pallet=pallet_subtensor_swap \
  --extrinsic="*" \
  --steps 50 \
  --repeat 20 \
  --output=pallets/swap/src/weights.rs \
  --template=./.maintain/frame-weight-template.hbs