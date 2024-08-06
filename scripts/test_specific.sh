pallet="${3:-pallet-subtensor}"
features="${4:-pow-faucet}"

SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release --features=$features -p $pallet --test $1 -- $2 --nocapture --exact