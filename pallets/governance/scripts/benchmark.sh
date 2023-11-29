cargo build --release --features runtime-benchmarks
./target/release/node-subtensor benchmark pallet \
    --chain=local \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet=pallet_governance \
    --extrinsic="*" \
    --output=pallets/governance/src/weights.rs \
    --template=./.maintain/frame-weight-template.hbs