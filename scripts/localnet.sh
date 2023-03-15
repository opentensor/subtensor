#!/bin/bash

: "${CHAIN:=local}"
: "${BUILD_BINARY:=1}"
: "${SPEC_PATH:=specs/}"

FULL_PATH="$SPEC_PATH$CHAIN.json"

if [[ $BUILD_BINARY == "1" ]]; then
	echo "*** Building substrate binary..."
	cargo build --release --features runtime-benchmarks 1>/dev/null
	echo "*** Binary compiled"
fi

echo "*** Building chainspec..."
./target/release/node-subtensor build-spec --disable-default-bootnode --raw --chain $CHAIN > $FULL_PATH
echo "*** Chainspec built and output to file"

echo "*** Purging previous state..."
./target/debug/node-subtensor purge-chain -y --base-path /tmp/bob --chain=$FULL_PATH 2>&1 > /dev/null
./target/debug/node-subtensor purge-chain -y --base-path /tmp/alice --chain=$FULL_PATH 2>&1 > /dev/null
echo "*** Previous chainstate purged"

echo "*** Starting localnet nodes..."
alice_start=(
	./target/release/node-subtensor
	--base-path /tmp/alice
	--chain=$FULL_PATH
	--alice
	--port 30334
	--ws-port 9946
	--rpc-port 9934
	--validator
	--rpc-cors=all
	--execution native
)

bob_start=(
	./target/release/node-subtensor
	--base-path /tmp/bob
	--chain=$FULL_PATH
	--bob
	--port 30335
	--ws-port 9947
	--rpc-port 9935
	--validator
	--execution native
	--bootnodes "/ip4/127.0.0.1/tcp/30334/p2p/12D3KooWBBUaVWE5SYj3UvnoXojfS8fvPorw5biRDaDQV7XXwCXm"
)

(trap 'kill 0' SIGINT; ("${alice_start[@]}" 2>&1) & ("${bob_start[@]}" 2>&1))
