#!/usr/bin/env bash

# Tries runtime upgrade (via try-runtime).
#
# Usage:
#   try-runtime-upgrade.sh [-p <runtime-path>] [-u <live-chain-url>] [-s <snapshot-path>]
#
# Dependencies:
#   - rust toolchain
#   - try-runtime-cli

set -eou pipefail

runtime_wasm_path="./target/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.wasm"
live_chain_url="wss://dev.chain.opentensor.ai"
snapshot_path=""

parse_args() {
  u_provided=false

  while getopts "r:u:s:" opt; do
    case "${opt}" in
    r) runtime_wasm_path="${OPTARG}" ;;
    u)
      live_chain_url="${OPTARG}"
      u_provided=true
      ;;
    s) snapshot_path="${OPTARG}" ;;
    *) echo "Usage: $(basename "$0") [-r <runtime-path>] [-u <live-chain-url>] [-s <snapshot-path>]" && exit 1 ;;
    esac
  done

  # Prevent specifying URI if snapshot is specified
  if [ -n "$snapshot_path" ] && [ "$u_provided" = true ]; then
    echo "Error: Either live URI or snapshot path should be specified, but not both."
    exit 1
  fi
}

build_runtime() {
  cargo build -p node-subtensor-runtime --release --features "metadata-hash,try-runtime"
}

do_try_runtime() {
  if [ -n "$snapshot_path" ]; then
    chain_state="snap --path $snapshot_path"
  else
    chain_state="live --uri $live_chain_url"
  fi

  eval "try-runtime --runtime $runtime_wasm_path on-runtime-upgrade \
    --no-weight-warnings --disable-spec-version-check --disable-idempotency-checks --checks=all \
    --blocktime 12000 \
    $chain_state"
}

parse_args "$@"
build_runtime
do_try_runtime
