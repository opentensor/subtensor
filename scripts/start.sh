#!/bin/bash
# This line specifies that this script should be run using the bash shell interpreter.

# Determine the directory this script resides in. This allows invoking it from any location.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
# This line sets the SCRIPT_DIR variable to the directory where the script is located.
# It uses 'cd' to change to the script's directory, then 'pwd' to get the full path.

# Function to parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        # This loop continues as long as there are arguments left to process
        case $1 in
            # This case statement checks the current argument
            --base_dir)
                BASE_DIR="$2"
                # If the argument is --base_dir, set BASE_DIR to the next argument
                shift 2
                # Move past these two arguments
                ;;
            --chain)
                CHAIN="$2"
                # If the argument is --chain, set CHAIN to the next argument
                shift 2
                ;;
            --spec_path)
                SPEC_PATH="$2"
                # If the argument is --spec_path, set SPEC_PATH to the next argument
                shift 2
                ;;
            --features)
                FEATURES="$2"
                # If the argument is --features, set FEATURES to the next argument
                shift 2
                ;;
            --keys)
                KEYS_FILE="$2"
                # If the argument is --keys, set KEYS_FILE to the next argument
                shift 2
                ;;
            *)
                echo "Unknown option: $1"
                # If the argument doesn't match any known option, print an error
                exit 1
                # Exit the script with an error code
                ;;
        esac
    done
}

# Function to set default values
set_defaults() {
    : "${BASE_DIR:="$SCRIPT_DIR/.."}"
    # Set BASE_DIR to parent directory of SCRIPT_DIR if not already set
    : "${CHAIN:=greg}"
    # Set CHAIN to 'greg' if not already set
    : "${SPEC_PATH:="${SCRIPT_DIR}/specs/"}"
    # Set SPEC_PATH to 'specs' subdirectory in SCRIPT_DIR if not already set
    : "${FEATURES:="pow-faucet runtime-benchmarks fast-blocks"}"
    # Set FEATURES to default values if not already set
    : "${BUILD_BINARY:=1}"
    # Set BUILD_BINARY to 1 if not already set
    : "${KEYS_FILE:="keys.json"}"
    # Set KEYS_FILE to 'keys.json' if not already set
    FULL_PATH="$SPEC_PATH$CHAIN.json"
    # Set FULL_PATH to the complete path of the chain specification file
}

# Function to print setup information
print_setup_info() {
    echo "Setup Information:"
    echo "Base Directory: $BASE_DIR"
    echo "Chain: $CHAIN"
    echo "Spec Path: $SPEC_PATH"
    echo "Features: $FEATURES"
    echo "Keys File: $KEYS_FILE"
    # Print out the current setup information
}

# Function to build the binary
build_binary() {
    if [[ $BUILD_BINARY == "1" ]]; then
        # If BUILD_BINARY is set to 1
        echo "*** Building substrate binary..."
        cargo build --release --features "$FEATURES" --manifest-path "$BASE_DIR/Cargo.toml"
        # Build the substrate binary with specified features
        echo "*** Binary compiled"
    fi
}

# Function to build the chain specification
build_chainspec() {
    echo "*** Building chainspec..."
    if [ ! -d "$SPEC_PATH" ]; then
        # If SPEC_PATH directory doesn't exist
        echo "*** Creating directory ${SPEC_PATH}..."
        mkdir $SPEC_PATH
        # Create the directory
    fi
    "$BASE_DIR/target/release/node-subtensor" build-spec --disable-default-bootnode --raw --chain $CHAIN >$FULL_PATH
    # Build the chain specification and output it to FULL_PATH
    echo "*** Chainspec built and output to file"
}

# Function to purge the chain
purge_chain() {
    echo "*** Purging previous state..."
    for i in $(seq 1 $(jq '.keys | length' $KEYS_FILE)); do
        "$BASE_DIR/target/release/node-subtensor" purge-chain -y --base-path /tmp/validator$i --chain="$FULL_PATH" >/dev/null 2>&1
    done
    # Purge the chain data for all validators
    echo "*** Previous chainstate purged"
}

# Function to run the nodes
run_nodes() {
    echo "*** Starting localnet nodes..."
    export RUST_LOG=subtensor=trace
    # Set the log level for subtensor to trace

    trap 'pkill -P $$' EXIT SIGINT SIGTERM
    # Set up a trap to kill all child processes when the script exits

    (
        for i in $(seq 0 $(($(jq '.keys | length' $KEYS_FILE) - 1))); do
            validator_start=(
                "$BASE_DIR/target/release/node-subtensor"
                --base-path /tmp/validator$((i+1))
                --chain="$FULL_PATH"
                --port $((30344 + i))
                --rpc-port $((9944 + i))
                --validator
                --rpc-cors=all
                --rpc-external
                --unsafe-rpc-external
                --rpc-methods=unsafe  
                --allow-private-ipv4
                --bootnodes /ip4/104.171.201.172/tcp/30345/p2p/12D3KooWK7N5CznrhErMethD9B8wamfnabnu5vXxmWurE4rKgj4n /ip4/104.171.201.172/tcp/30344/p2p/12D3KooWEnfmHWpKvRXJMBYoy1E7rjDDrxiSbqTcUGWVZY9Kvcq2
            )
            # Configuration for each validator node

            aura_key=$(jq -r ".keys[$i].aura.secret_phrase" $KEYS_FILE)
            gran_key=$(jq -r ".keys[$i].grandpa.secret_phrase" $KEYS_FILE)
            echo "Aura key: $aura_key"
            echo "Grandpa key: $gran_key"

            insert_validator_aura_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
                --base-path /tmp/validator$((i+1))
                --chain="$FULL_PATH"
                --scheme Sr25519
                --suri "$aura_key"
                --key-type aura
            )
            # Command to insert the Aura key for each validator

            insert_validator_gran_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
                --base-path /tmp/validator$((i+1))
                --chain="$FULL_PATH"
                --scheme Ed25519
                --suri "$gran_key"
                --key-type gran
            )
            # Command to insert the Grandpa key for each validator

            "${insert_validator_aura_key[@]}" 2>&1
            "${insert_validator_gran_key[@]}" 2>&1
            "${validator_start[@]}" 2>&1 &
        done

        wait
    )
    # Start all the processes in the background and wait for them to finish
}

# Main execution
if [ "$1" = "build" ]; then
    # If the first argument is "build"
    shift
    parse_args "$@"
    set_defaults
    print_setup_info
    build_binary
    build_chainspec
elif [ "$1" = "run" ]; then
    # If the first argument is "run"
    shift
    parse_args "$@"
    set_defaults
    print_setup_info
    purge_chain
    run_nodes
else
    # If the first argument is neither "build" nor "run"
    echo "Usage: $0 [build|run] [options]"
    echo "Options:"
    echo "  --base_dir <dir>    Set the base directory"
    echo "  --chain <chain>     Set the chain"
    echo "  --spec_path <path>  Set the spec path"
    echo "  --features <list>   Set the features"
    echo "  --keys <file>       Set the keys file"
    exit 1
    # Print usage information and exit with an error code
fi
