cargo build --release --features runtime-benchmarks
./target/release/node-subtensor benchmark pallet \
    --chain=local \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet=pallet_registry \
    --extrinsic="*" \
    --output=pallets/registry/src/weights.rs \
    --template=./.maintain/frame-weight-template.hbs