#!/bin/bash

set -e

cd $(dirname $0)/..

mkdir -p specs

../target/release/node-subtensor build-spec --disable-default-bootnode --raw --chain local > specs/chain-spec.json