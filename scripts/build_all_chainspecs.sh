#!/bin/bash

echo "*** Building node..."
cargo build

echo "*** Building chainspecs..."
./target/debug/node-subtensor build-spec --raw --chain finney >raw_spec_finney.json
./target/debug/node-subtensor build-spec --chain finney >plain_spec_finney.json

./target/debug/node-subtensor build-spec --raw --chain test_finney >raw_spec_testfinney.json
./target/debug/node-subtensor build-spec --chain test_finney >plain_spec_testfinney.json

echo "*** Done"
