#!/bin/bash

echo "start run-ci.sh"

cd contract-tests

cd bittensor

rustup component add rust-src
cargo install cargo-contract 
cargo contract build --release 

cd ../..

scripts/localnet.sh &

i=1
while [ $i -le 2000 ]; do
  if nc -z localhost 9944; then
    echo "node subtensor is running after $i seconds"
    break
  fi
  sleep 1
  i=$((i + 1))
done

# port not available exit with error
if [ "$i" -eq 2000 ]; then
    exit 1
fi

sleep 10

if ! nc -z localhost 9944; then
    echo "node subtensor exit, port not available"
    exit 1
fi

cd contract-tests

# required for papi in get-metadata.sh, but we cannot run yarn before papi as it adds the descriptors to the package.json which won't resolve
npm i -g polkadot-api

bash get-metadata.sh

sleep 5

yarn install --frozen-lockfile

yarn run test
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo "Tests failed with exit code $TEST_EXIT_CODE"
    pkill node-subtensor
    exit $TEST_EXIT_CODE
fi

pkill node-subtensor

exit 0