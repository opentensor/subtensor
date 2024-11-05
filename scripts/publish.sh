#!/bin/bash
set -ex
cd support/macros
cargo publish $1
cd ../..
cd pallets/commitments
cargo publish $1
cd ..
cd collective
cargo publish $1
cd ..
cd registry
cargo publish $1
cd ..
cd subtensor
cargo publish $1
cd runtime-api
cargo publish $1
cd ../..
cd admin-utils
cargo publish $1
cd ../..
cd runtime
cargo publish $1
cd ..
cd node
cargo publish $1
echo "published successfully."
