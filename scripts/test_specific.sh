pallet="${3:-pallet-subtensor}"
features="${4:-pow-faucet}"

RUST_LOG=pallet_subtensor=trace cargo test --release --features=$features -p $pallet --test $1 -- $2 --nocapture --exact