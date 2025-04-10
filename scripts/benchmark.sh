#!/usr/bin/env bash

DEFAULT_BIN_PATH='./target/production/node-subtensor'
BIN_PATH="$DEFAULT_BIN_PATH"
OUTPUT_FILE='benchmarking.txt'

# Getting arguments from user
while [[ $# -gt 0 ]]; do
    case $1 in
        -p | --bin-path)
            BIN_PATH="$2"
            shift
            shift
        ;;
        -* | --*)
            echo "Unknown option $1"
            exit 1
        ;;
        *)
            POSITIONAL_ARGS+=("$1")
            shift
        ;;
    esac
done

echo "*** Building all chain specs using 'build_all_chainspecs.sh' ***"
./scripts/build_all_chainspecs.sh
CHAIN_SPEC='chainspecs/raw_spec_finney.json'

echo "*** Building node-subtensor with 'runtime-benchmarks' ***"
cargo build \
    --profile production \
    --package node-subtensor \
    --bin node-subtensor \
    --features "runtime-benchmarks,try-runtime,pow-faucet"

if [ ! -f "$BIN_PATH" ]; then
    echo "ERROR: Node binary '$BIN_PATH' not found after build."
    exit 1
fi

echo "*** Running benchmark ***"
"$BIN_PATH" benchmark pallet \
    --chain "$CHAIN_SPEC" \
    --wasm-execution=compiled \
    --pallet pallet-subtensor \
    --extrinsic 'benchmark_register' \
    --output "$OUTPUT_FILE"

echo "*** Benchmark completed. Results saved to '$OUTPUT_FILE' ***"
