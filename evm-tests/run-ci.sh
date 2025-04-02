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

cd evm-tests

sudo apt-get install -y nodejs

yarn

npm install polkadot-api

sh get-metadata.sh

sleep 5

yarn run test

pkill node-subtensor
