#!/bin/bash
set -euo pipefail
cd /home/johnreed/Desktop/work/subtensor
R=/tmp/ms-release-85a857922f
trap 'echo $? > "$R/additional.exit"' EXIT
while [ ! -f "$R/preflight.exit" ]; do sleep 5; done
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=4
export PYTHONPATH="$R/native:$PWD/sdk/python"
SKIP_WASM_BUILD=1 cargo test -p node-subtensor-runtime --all-features every_metadata_call_has_explicit_envelope_and_dispatch_coverage --locked -- --nocapture > "$R/dispatch-matrix.log" 2>&1
SKIP_WASM_BUILD=1 MS_FEE_QUOTE_OUTPUT="$R/fee-quotes.json" cargo test -p node-subtensor-runtime --test mldsa_reference_fees --locked > "$R/production-tests.log" 2>&1
SKIP_WASM_BUILD=1 cargo test -p node-subtensor-runtime neuron_evm_association_honors_production_cooldown_for_every_wallet_mode --locked > "$R/default-cooldown.log" 2>&1
SKIP_WASM_BUILD=1 cargo test -p node-subtensor-runtime evm_mapping_meter_tests --locked > "$R/default-evm-meter.log" 2>&1
cargo check --locked --offline -p bittensor-core --no-default-features --target wasm32-unknown-unknown > "$R/wasm-core.log" 2>&1
cargo check --locked --offline -p bittensor-core-wasm --target wasm32-unknown-unknown > "$R/wasm-binding.log" 2>&1
wasm-pack build sdk/bittensor-core-wasm --target nodejs --out-dir pkg-node --no-opt --locked --offline > "$R/wasm-build.log" 2>&1
node sdk/bittensor-core-wasm/tests/smoke.mjs > "$R/wasm-smoke.log" 2>&1
env -u SKIP_WASM_BUILD cargo build --profile production -p node-subtensor-runtime --features try-runtime,metadata-hash --locked --offline > "$R/mainnet-build.log" 2>&1
cp target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm "$R/mainnet/runtime.wasm"
while [ ! -f "$R/mainnet/snapshot.exit" ]; do sleep 5; done
[ "$(cat "$R/mainnet/snapshot.exit")" = 0 ]
sdk/python/.venv/bin/python docs/design/evidence/ml-dsa-mainnet-upgrade.py --runtime "$R/mainnet/runtime.wasm" --snapshot "$R/mainnet/mainnet.snap" --source "$R/mainnet/source.json" --output-dir "$R/mainnet-verified" --try-runtime "$R/bin/try-runtime" > "$R/mainnet-verified.log" 2>&1
