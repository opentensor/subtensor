#!/bin/sh
set -e

chown -R subtensor:subtensor /data

if [ -d "/tmp/blockchain" ]; then
    chown -R subtensor:subtensor /tmp/blockchain
fi

# Execute node-subtensor with any arguments passed to the script as subtensor user
exec gosu subtensor node-subtensor "$@"