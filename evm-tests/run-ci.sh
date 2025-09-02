#!/bin/bash

echo "start run-ci.sh"

scripts/localnet.sh &>/dev/null &

i=1
while [ $i -le 1000 ]; do
  if nc -z localhost 9944; then
    echo "node subtensor is running after $i seconds"
    break
  fi
  sleep 1
  i=$((i + 1))
done

# port not available exit with error
if [ "$i" -eq 1000 ]; then
    exit 1
fi

sleep 5

if ! nc -z localhost 9944; then
    echo "node subtensor exit, port not available"
    exit 1
fi

cd evm-tests

npm install -g polkadot-api

bash get-metadata.sh

sleep 5

yarn

yarn run test
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo "Tests failed with exit code $TEST_EXIT_CODE"
    pkill node-subtensor
    exit $TEST_EXIT_CODE
fi

pkill node-subtensor

exit 0