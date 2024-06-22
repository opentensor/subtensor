#!/bin/bash

# This line specifies that this script should be run using the bash shell interpreter.

# Determine the directory this script resides in. This allows invoking it from any location.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
# This line sets the SCRIPT_DIR variable to the directory where the script is located.
# It does this by changing to the directory of the script and then getting the current working directory.
# This allows the script to be run from any location while still referencing files relative to its own location.
# Example: If the script is in /home/user/projects/myscript.sh, SCRIPT_DIR will be set to /home/user/projects

# The base directory of the subtensor project
BASE_DIR="$SCRIPT_DIR/.."
# This sets the BASE_DIR variable to the parent directory of the script's location.
# It's using the .. notation to go up one directory level from SCRIPT_DIR.
# Example: If SCRIPT_DIR is /home/user/projects/scripts, BASE_DIR will be /home/user/projects

# These lines set default values for variables if they're not already set in the environment
: "${CHAIN:=greg}"
# This sets the CHAIN variable to "greg" if it's not already set.
# The chain name is used later in the script to determine which blockchain to work with.
# Example: If CHAIN is not set, it will be "greg". If it's set to "testnet", it will remain "testnet".

: "${BUILD_BINARY:=1}"
# This sets BUILD_BINARY to 1 (true) if it's not already set.
# This flag determines whether the script should build the blockchain binary or not.
# Example: If BUILD_BINARY is not set, it will be 1. If it's set to 0, it will remain 0.

: "${FEATURES:="pow-faucet runtime-benchmarks fast-blocks"}"
# This sets the FEATURES variable to a space-separated list of features to be used when building the binary.
# These features add specific functionalities to the blockchain.
# Example: If FEATURES is not set, it will be "pow-faucet runtime-benchmarks fast-blocks".
# If it's set to "custom-feature1 custom-feature2", it will remain that value.

SPEC_PATH="${SCRIPT_DIR}/specs/"
# This sets the SPEC_PATH to a 'specs' subdirectory within the script's directory.
# This is where the chain specification files will be stored.
# Example: If SCRIPT_DIR is /home/user/projects/scripts, SPEC_PATH will be /home/user/projects/scripts/specs/

FULL_PATH="$SPEC_PATH$CHAIN.json"
# This creates the full path to the chain specification file by combining SPEC_PATH and CHAIN.
# Example: If SPEC_PATH is /home/user/projects/scripts/specs/ and CHAIN is "greg",
# FULL_PATH will be /home/user/projects/scripts/specs/greg.json

# Kill any existing nodes which may have not exited correctly after a previous run.
pkill -9 'node-subtensor'
# This command forcefully terminates any running processes named 'node-subtensor'.
# It's used to ensure no old instances of the blockchain node are running before starting new ones.
# Example: If there are two processes named 'node-subtensor' running, both will be terminated.

if [ ! -d "$SPEC_PATH" ]; then
  echo "*** Creating directory ${SPEC_PATH}..."
  mkdir $SPEC_PATH
fi
# This block checks if the SPEC_PATH directory exists. If it doesn't, it creates it.
# It's ensuring that there's a place to store the chain specification files.
# Example: If /home/user/projects/scripts/specs/ doesn't exist, it will be created.

# Check if we need to build the binary
if [[ $BUILD_BINARY == "1" ]]; then
  # Inform the user that we're starting to build the substrate binary
  echo "*** Building substrate binary..."
  # Use cargo to build the release version with specified features
  # --release: Build optimized artifacts with the release profile
  # --features: Build with the features specified in the $FEATURES variable
  # --manifest-path: Specify the path to the Cargo.toml file
  cargo build --release --features "$FEATURES" --manifest-path "$BASE_DIR/Cargo.toml"
  # This command builds the blockchain binary using Cargo (Rust's package manager).
  # It creates an optimized release build, includes the specified features, and uses the Cargo.toml file in the BASE_DIR.
  # Example: If BASE_DIR is /home/user/projects and FEATURES is "pow-faucet runtime-benchmarks fast-blocks",
  # the command would be similar to:
  # cargo build --release --features "pow-faucet runtime-benchmarks fast-blocks" --manifest-path "/home/user/projects/Cargo.toml"
  
  # Inform the user that the binary compilation is complete
  echo "*** Binary compiled"
fi

# Inform the user that we're starting to build the chain specification
echo "*** Building chainspec..."
# Use the compiled node-subtensor binary to build a raw chain specification
# build-spec: Generate a chain specification
# --disable-default-bootnode: Don't include the default bootnode in the spec
# --raw: Output the chain spec in raw format (as JSON)
# --chain: Specify which predefined chain specification to use
# > $FULL_PATH: Redirect the output to the file specified by $FULL_PATH
"$BASE_DIR/target/release/node-subtensor" build-spec --disable-default-bootnode --raw --chain $CHAIN >$FULL_PATH
# This command uses the newly built binary to generate a chain specification file.
# It creates a raw JSON format specification without default bootnodes, based on the specified chain.
# The output is saved to the file specified by FULL_PATH.
# Example: If BASE_DIR is /home/user/projects, CHAIN is "greg", and FULL_PATH is /home/user/projects/scripts/specs/greg.json,
# the command would be similar to:
# /home/user/projects/target/release/node-subtensor build-spec --disable-default-bootnode --raw --chain greg > /home/user/projects/scripts/specs/greg.json

# Inform the user that the chain specification has been built and saved
echo "*** Chainspec built and output to file"

# Inform the user that we're starting to purge the previous blockchain state
echo "*** Purging previous state..."
# Purge the chain data for validator1
# purge-chain: Remove all chain data
# -y: Answer yes to all prompts
# --base-path: Specify the base path where chain data is stored
# --chain: Specify the chain specification file to use
# >/dev/null 2>&1: Redirect both standard output and error to /dev/null (i.e., discard them)
"$BASE_DIR/target/release/node-subtensor" purge-chain -y --base-path /tmp/validator1 --chain="$FULL_PATH" >/dev/null 2>&1
# This command removes all existing blockchain data for validator1.
# It uses the newly created chain specification and stores data in /tmp/validator1.
# Any output or errors are discarded to keep the console clean.
# Example: If BASE_DIR is /home/user/projects and FULL_PATH is /home/user/projects/scripts/specs/greg.json,
# the command would be similar to:
# /home/user/projects/target/release/node-subtensor purge-chain -y --base-path /tmp/validator1 --chain="/home/user/projects/scripts/specs/greg.json" >/dev/null 2>&1

# Purge the chain data for validator2 (same process as for validator1)
"$BASE_DIR/target/release/node-subtensor" purge-chain -y --base-path /tmp/validator2 --chain="$FULL_PATH" >/dev/null 2>&1
# This does the same thing as the previous command, but for validator2, using /tmp/validator2 as the base path.

# Inform the user that the previous chain state has been purged
echo "*** Previous chainstate purged"

echo "*** Starting localnet nodes..."
export RUST_LOG=subtensor=trace
# This sets the RUST_LOG environment variable to enable detailed logging for the subtensor module.

validator1_start=(
  "$BASE_DIR/target/release/node-subtensor"
  --base-path /tmp/validator1
  --chain="$FULL_PATH"
  --port 30334
  --rpc-port 9946
  --validator
  --rpc-cors=all
  --rpc-external
  --unsafe-rpc-external
  --rpc-methods=unsafe  
  --allow-private-ipv4
  --bootnodes /ip4/104.171.201.172/tcp/30335/p2p/12D3KooWEnfmHWpKvRXJMBYoy1E7rjDDrxiSbqTcUGWVZY9Kvcq2 /ip4/104.171.201.172/tcp/30334/p2p/12D3KooWK7N5CznrhErMethD9B8wamfnabnu5vXxmWurE4rKgj4n \
  --discover-local
)
# This creates an array of command-line arguments for starting validator1.
# It specifies various parameters like the data directory, chain specification, network ports, and RPC settings.
# Example: If BASE_DIR is /home/user/projects and FULL_PATH is /home/user/projects/scripts/specs/greg.json,
# the array would contain elements like:
# /home/user/projects/target/release/node-subtensor
# --base-path /tmp/validator1
# --chain="/home/user/projects/scripts/specs/greg.json"
# --port 30334
# ... (and so on for the other parameters)

validator2_start=(
  "$BASE_DIR"/target/release/node-subtensor
  --base-path /tmp/validator2
  --chain="$FULL_PATH"
  --port 30335
  --rpc-port 9945
  --validator
  --rpc-cors=all
  --rpc-external
  --unsafe-rpc-external
  --rpc-methods=unsafe  
  --allow-private-ipv4
  --bootnodes /ip4/104.171.201.172/tcp/30335/p2p/12D3KooWEnfmHWpKvRXJMBYoy1E7rjDDrxiSbqTcUGWVZY9Kvcq2 /ip4/104.171.201.172/tcp/30334/p2p/12D3KooWK7N5CznrhErMethD9B8wamfnabnu5vXxmWurE4rKgj4n \
  --discover-local
)
# This creates a similar array for validator2, with some different parameters (like different ports).

insert_validator_1_aura_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
  --base-path /tmp/validator1 
  --chain="$FULL_PATH"
  --scheme Sr25519 \
  --suri "subject one mention gown inside fluid recycle essence hair robot ozone point" \
  --key-type aura
)
# This array contains the command to insert the Aura (authority round) key for validator1.
# It specifies the key type, the secret phrase (seed) for the key, and where to store it.
# Example: If BASE_DIR is /home/user/projects and FULL_PATH is /home/user/projects/scripts/specs/greg.json,
# the array would contain elements like:
# /home/user/projects/target/release/node-subtensor key insert
# --base-path /tmp/validator1
# --chain="/home/user/projects/scripts/specs/greg.json"
# --scheme Sr25519
# --suri "subject one mention gown inside fluid recycle essence hair robot ozone point"
# --key-type aura

insert_validator_1_gran_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
  --base-path /tmp/validator1 
  --chain="$FULL_PATH"
  --scheme Ed25519 \
  --suri "subject one mention gown inside fluid recycle essence hair robot ozone point" \
  --key-type gran
)
# This is similar to the previous array, but for inserting the GRANDPA (finality gadget) key for validator1.

insert_validator_2_aura_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
  --base-path /tmp/validator2 
  --chain="$FULL_PATH"
  --scheme Sr25519 
  --suri "coach force devote mule oppose awesome type pelican bone concert tiger reduce" \
  --key-type aura
)
# This array is for inserting the Aura key for validator2, using a different secret phrase.

insert_validator_2_gran_key=( "$BASE_DIR"/target/release/node-subtensor key insert 
  --base-path /tmp/validator2 
  --chain="$FULL_PATH"
  --scheme Ed25519 
  --suri "coach force devote mule oppose awesome type pelican bone concert tiger reduce" \
  --key-type gran
)
# This array is for inserting the GRANDPA key for validator2.

trap 'pkill -P $$' EXIT SIGINT SIGTERM
# This sets up a trap to kill all child processes when the script exits or is interrupted.
# It ensures that all started processes are properly terminated when the script ends.

(
  ("${validator1_start[@]}" 2>&1) &
  ("${validator2_start[@]}" 2>&1) &
  ("${insert_validator_1_aura_key[@]}" 2>&1) &
  ("${insert_validator_1_gran_key[@]}" 2>&1) &
  ("${insert_validator_2_aura_key[@]}" 2>&1) &
  ("${insert_validator_2_gran_key[@]}" 2>&1) &

  wait
)
# This block starts all the processes in the background:
# - It starts both validator nodes
# - It inserts the Aura and GRANDPA keys for both validators
# The '&' at the end of each line makes the process run in the background.
# The 'wait' command at the end makes the script wait for all background processes to finish.
# The 2>&1 redirects both standard output and error to the same place (usually the console).