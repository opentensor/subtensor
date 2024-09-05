#!/bin/bash
set -ex
cd support/macros
cargo publish --token $1
cd ../..
cd pallets/commitments
cargo publish --token $1
cd ..
cd collective
cargo publish --token $1
cd ..
cd registry
cargo publish --token $1
cd ..
cd subtensor
cargo publish --token $1
cd runtime-api
cargo publish --token $1
cd ../..
cd admin-utils
cargo publish --token $1
cd ../..
cd runtime
cargo publish --token $1
cd ..
cd node
cargo publish --token $1
echo "published successfully."
