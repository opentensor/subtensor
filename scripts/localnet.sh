#!/bin/bash

: "${CHAIN:=local}"

echo "*** Building chainspec..."
./target/debug/node-subtensor build-spec -lnone --disable-default-bootnode --chain $CHAIN > $CHAIN.json

echo "*** Purging previous state..."
./target/debug/node-subtensor purge-chain -y --base-path /tmp/bob --chain=$CHAIN.json 2>&1 > /dev/null
./target/debug/node-subtensor purge-chain -y --base-path /tmp/alice --chain=$CHAIN.json 2>&1 > /dev/null

echo "*** Starting localnet nodes..."
alice_start="./target/debug/node-subtensor --base-path /tmp/alice --chain=$CHAIN.json --alice --port 30334 --ws-port 9946 --rpc-port 9934 --validator --rpc-cors=all"
bob_start="./target/debug/node-subtensor --base-path /tmp/bob --chain=$CHAIN.json --bob --port 30335 --ws-port 9947 --rpc-port 9935 --validator --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWBBUaVWE5SYj3UvnoXojfS8fvPorw5biRDaDQV7XXwCXm"

(trap 'kill 0' SIGINT; ($alice_start 2>&1) & ($bob_start 2>&1))