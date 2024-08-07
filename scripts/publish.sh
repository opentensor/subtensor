#!/bin/bash
set -ex
cd support/macros
cargo publish
cd ../..
cd pallets/commitments
cargo publish
cd ..
cd collective
cargo publish
cd ..
cd registry
cargo publish
cd ..
cd subtensor
cargo publish
cd runtime-api
cargo publish
cd ../..
cd admin-utils
cargo publish
cd ../..
cd runtime
cargo publish
cd ..
cd node
cargo publish
echo "published successfully."
