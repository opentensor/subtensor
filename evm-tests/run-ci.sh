#!/bin/bash

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

echo "go to evm-tests"
cd evm-tests

npm install --global yarn

yarn

sleep 5

sh get-metadata.sh

yarn run test

pkill node-subtensor