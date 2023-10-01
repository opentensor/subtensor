cargo build --release --features runtime-benchmarks
./target/release/node-subtensor build-spec --disable-default-bootnode --raw --chain local > temp.json
./target/release/node-subtensor benchmark pallet --chain=temp.json --execution=native  --wasm-execution=compiled --pallet pallet-subtensor --extrinsic 'benchmark_dissolve_network' --output benchmarking.txt
rm temp.json