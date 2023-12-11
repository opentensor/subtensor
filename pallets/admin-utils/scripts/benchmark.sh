cargo build --release --features runtime-benchmarks
./target/release/node-subtensor benchmark pallet \
    --chain=local \
    --execution=wasm \
    --wasm-execution=compiled \
    --pallet=pallet_admin_utils \
    --extrinsic="*" \
    --steps 50 \
    --repeat 20 \
    --output=pallets/admin-utils/src/weights.rs \
    --template=./.maintain/frame-weight-template.hbs