#!/bin/bash
set -euo pipefail
cd /home/johnreed/Desktop/work/subtensor
R=/tmp/ms-release-85a857922f
trap 'echo $? > "$R/build.exit"' EXIT
export PATH="/tmp/ms-reference-39060946210/bin:$PATH"
env -u SKIP_WASM_BUILD cargo build --release -p node-subtensor --features metadata-hash --locked > "$R/build-node.log" 2>&1
cp target/release/node-subtensor "$R/node-subtensor"
cp target/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm "$R/runtime.wasm"
cargo build --release -p bittensor-core-py --locked > "$R/build-native.log" 2>&1
cp target/release/libbittensor_core.so "$R/native/bittensor_core.so"
sha256sum "$R/node-subtensor" "$R/runtime.wasm" "$R/native/bittensor_core.so" > "$R/artifacts.sha256"
