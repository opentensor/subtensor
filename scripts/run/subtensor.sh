#!/usr/bin/env bash

#
# Helper functions
#

function run_command()
{
    F_NETWORK=$1
    F_NODE_TYPE=$2

    # Different command options by network and node type
    MAINNET_BOOTNODE='--bootnodes /dns/bootnode.finney.opentensor.ai/tcp/30333/ws/p2p/12D3KooWRwbMb85RWnT8DSXSYMWQtuDwh4LJzndoRrTDotTR5gDC'
    TESTNET_BOOTNODE='--bootnodes /dns/bootnode.test.finney.opentensor.ai/tcp/30333/ws/p2p/12D3KooWRwbMb85RWnT8DSXSYMWQtuDwh4LJzndoRrTDotTR5gDC'
    NODE_TYPE_ARCHIVE='--pruning=archive'
    NODE_TYPE_LITE='--sync warp'

    # Options by the type of node we offer
    MAINNET_ARCHIVE_OPTIONS="$MAINNET_BOOTNODE $NODE_TYPE_ARCHIVE"
    MAINNET_LITE_OPTIONS="$MAINNET_BOOTNODE $NODE_TYPE_LITE"
    TESTNET_ARCHIVE_OPTIONS="$TESTNET_BOOTNODE $NODE_TYPE_ARCHIVE"
    TESTNET_LITE_OPTIONS="$TESTNET_BOOTNODE $NODE_TYPE_LITE"

    # Checking options to use
    if [[ "$F_NETWORK" == "mainnet" ]] && [[ "$F_NODE_TYPE" == "archive" ]]; then
        SPECIFIC_OPTIONS=$MAINNET_ARCHIVE_OPTIONS
    elif [[ "$F_NETWORK" == "mainnet" ]] && [[ "$F_NODE_TYPE" == "lite" ]]; then
        SPECIFIC_OPTIONS=$MAINNET_LITE_OPTIONS 
    elif [[ "$F_NETWORK" == "testnet" ]] && [[ "$F_NODE_TYPE" == "archive" ]]; then
        SPECIFIC_OPTIONS=$TESTNET_ARCHIVE_OPTIONS 
    elif [[ "$F_NETWORK" == "testnet" ]] && [[ "$F_NODE_TYPE" == "lite" ]]; then
        SPECIFIC_OPTIONS=$TESTNET_LITE_OPTIONS 
    fi

    if [ ! -f ./target/release/node-subtensor ]; then
        echo 'Binary ./target/release/node-subtensor does not exist'
        echo 'Please ensure you have compiled the binary first'
        exit 1
    fi

    # Command to run subtensor
    ./target/release/node-subtensor \
        --base-path /tmp/blockchain \
        --chain ./raw_spec.json \
        --rpc-external --rpc-cors all \
        --ws-external --no-mdns \
        --ws-max-connections 10000 --in-peers 500 --out-peers 500 \
        $SPECIFIC_OPTIONS
}


# Default values
EXEC_TYPE="docker"
NETWORK="mainnet"
NODE_TYPE="lite"
BUILD=""

# Getting arguments from user
while [[ $# -gt 0 ]]; do
  case $1 in
    -h|--help)
      help
      exit 0
      ;;
    -e|--execution)
      EXEC_TYPE="$2"
      shift # past argument
      shift # past value
      ;;
    -b|--build)
      BUILD="--build"
      shift # past argument
      ;;
    -n|--network)
      NETWORK="$2"
      shift
      shift
      ;;
    -t|--node-type)
      NODE_TYPE="$2"
      shift
      shift
      ;;
    -*|--*)
      echo "Unknown option $1"
      exit 1
      ;;
    *)
      POSITIONAL_ARGS+=("$1")
      shift
      ;;
  esac
done

# Verifying arguments values
if ! [[ "$EXEC_TYPE" =~ ^(docker|binary)$ ]]; then
    echo "Exec type not expected: $EXEC_TYPE"
    exit 1
fi

if ! [[ "$NETWORK" =~ ^(mainnet|testnet)$ ]]; then
    echo "Network not expected: $NETWORK"
    exit 1
fi

if ! [[ "$NODE_TYPE" =~ ^(lite|archive)$ ]]; then
    echo "Node type not expected: $NODE_TYPE"
    exit 1
fi

# Running subtensor
case $EXEC_TYPE in
    docker)
        docker compose down --remove-orphans
        echo "Running docker compose up $BUILD --detach $NETWORK-$NODE_TYPE"
        docker compose up $BUILD --detach $NETWORK-$NODE_TYPE
    ;;
    binary)
        run_command $NETWORK $NODE_TYPE
    ;;
esac
