#!/bin/bash

echo "*** Local testnet installation"
echo "*** Installing substrate support libraries"

# Install support libraries for compiling substrate binaries
# verify md5
curl https://getsubstrate.io -sSf > support_install.sh
if ! md5sum --status --check <<< "7296b9d45a89e973528c3ae31719ff08 support_install.sh"; then
	echo "Substrate library script checksum not valid, exiting."
	exit
fi
chmod +rx support_install.sh
bash support_install.sh
rm support_install.sh

echo "*** Building node binary..."

# Build binary
cargo build

echo "*** Setup complete, use localnet.sh in scripts to start a local network."
exit