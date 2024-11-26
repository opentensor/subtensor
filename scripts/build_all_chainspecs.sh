#!/bin/bash

set -e

raw_finney="chainspecs/raw_spec_finney.json"
raw_testfinney="chainspecs/raw_spec_testfinney.json"
plain_finney="chainspecs/plain_spec_finney.json"
plain_testfinney="chainspecs/plain_spec_testfinney.json"

finney_genesis_temp=$(mktemp)
testfinney_genesis_temp=$(mktemp)
raw_spec_finney_temp=$(mktemp)
raw_spec_testfinney_temp=$(mktemp)

save_genesis() {
  jq -r ".genesis" "$1" >"$2"
}

buildspec() {
  local chain="$1"
  shift
  ./target/debug/node-subtensor build-spec --chain "$chain" "$@"
}

# Update genesis in new chainspecs using the extracted genesis data from the
# temporary files
update_genesis() {
  jq --slurpfile genesis "$1" '.genesis = $genesis[0]' "$2" >"$3"
}

cleanup() {
  rm -f "$finney_genesis_temp" \
    "$testfinney_genesis_temp" \
    "$raw_spec_finney_temp" \
    "$raw_spec_testfinney_temp"
}

# SCRIPT

echo "*** Building new chainspecs..."

echo "*** Building node..."
cargo build -p node-subtensor

# Save old genesis state before doing anything
save_genesis "$raw_finney" "$finney_genesis_temp"
save_genesis "$raw_testfinney" "$testfinney_genesis_temp"

# Build new chainspecs
buildspec finney --raw >"$raw_spec_finney_temp"
buildspec finney >"$plain_finney"

buildspec test_finney --raw  >"$raw_spec_testfinney_temp"
buildspec test_finney >"$plain_testfinney"

echo "*** Updating genesis..."

# The genesis is not allowed to change. Since the wasm genesis will change
# depending on the system architecture used, we need to extract the genesis from
# the old chain specs and insert them into the new chain specs to ensure there
# are no genesis mismatch issues.
update_genesis "$finney_genesis_temp" "$raw_spec_finney_temp" "$raw_finney"
update_genesis "$testfinney_genesis_temp" "$raw_spec_testfinney_temp" \
  "$raw_testfinney"

cleanup

echo "*** Done!"
