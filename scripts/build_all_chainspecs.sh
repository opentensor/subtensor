#!/bin/bash

# The genesis is not allowed to change. Since the wasm genesis will change
# depending on the system architecture used, we need to extract the genesis from
# the old chain specs and insert them into the new chain specs to ensure there
# are no genesis mismatch issues.

# This script updates the chain spec files keeping the genesis unchanged.

set -e

raw_finney="chainspecs/raw_spec_finney.json"
plain_finney="chainspecs/plain_spec_finney.json"
raw_testfinney="chainspecs/raw_spec_testfinney.json"
plain_testfinney="chainspecs/plain_spec_testfinney.json"
raw_devnet="chainspecs/raw_spec_devnet.json"
plain_devnet="chainspecs/plain_spec_devnet.json"

save_genesis() {
  jq -r ".genesis" "$1" >"$2"
}

buildspec() {
  local chain="$1"
  shift
  ./target/debug/node-subtensor build-spec --chain "$chain" --disable-default-bootnode "$@"
}

# Update genesis in new chainspecs using the extracted genesis data from the
# temporary files
update_genesis() {
  jq --slurpfile genesis "$1" '.genesis = $genesis[0]' "$2" >"$3"
}

update_spec() {
  local chain="$1"
  local raw_path="$2"
  local plain_path="$3"

  raw_genesis_temp=$(mktemp)
  plain_genesis_temp=$(mktemp)
  raw_spec_temp=$(mktemp)
  plain_spec_temp=$(mktemp)

  echo "*** Backing up genesis for '$chain'..."

  save_genesis "$raw_path" "$raw_genesis_temp"
  save_genesis "$plain_path" "$plain_genesis_temp"

  echo "*** Building new chainspec for '$chain'..."

  # Build new chainspecs
  buildspec "$chain" >"$plain_spec_temp"
  buildspec "$chain" --raw >"$raw_spec_temp"

  echo "*** Restoring genesis in '$chain'..."

  update_genesis "$raw_genesis_temp" "$raw_spec_temp" "$raw_path"
  update_genesis "$plain_genesis_temp" "$plain_spec_temp" "$plain_path"

  # cleanup
  rm -f "$raw_genesis_temp" "$plain_genesis_temp" "$raw_spec_temp" \
    "$plain_spec_temp"
}

# SCRIPT

echo "*** Building node..."
cargo build -p node-subtensor

update_spec finney "$raw_finney" "$plain_finney"
update_spec test_finney "$raw_testfinney" "$plain_testfinney"
update_spec devnet "$raw_devnet" "$plain_devnet"

echo "*** Done!"
