cargo build --release --features runtime-benchmarks
rm ./pallets/subtensor/src/extrinsic_weights.rs
./target/release/node-subtensor \
    benchmark \
    pallet \
    --chain=local \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet=pallet_subtensor \
    --extrinsic=* \
    --output=./pallets/subtensor/src/extrinsic_weights.rs \
    --template=./.maintain/frame-weight-template.hbs