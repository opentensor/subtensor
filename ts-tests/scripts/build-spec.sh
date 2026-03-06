#!/bin/bash

set -e

cd $(dirname $0)/..

../target/release/node-subtensor build-spec --disable-default-bootnode --raw --chain local > specs/chain-spec.json