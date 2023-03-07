#!/bin/bash

_script_name='support_install.sh'
_valid_hash='7296b9d45a89e973528c3ae31719ff08'

if [[ -f "${_script_name:?Undfined script name}" ]]; then
	printf >&2 'Script already exists.\n'
	exit 1
fi

echo "*** Local testnet installation"
echo "*** Installing substrate support libraries"

# Install support libraries for compiling substrate binaries
# verify md5
curl https://getsubstrate.io -sSf > "${_script_name}"
if ! md5sum --status --check <<< "${_valid_hash:?Undfined hash} ${_script_name}"; then
	_status="${?}"
	printf >&2 'Substrate library script checksum not valid, exiting.\n'
	exit "${_status}"
fi
chmod +rx "${_script_name}"
bash "${_script_name}"
rm "${_script_name}"

echo "*** Building node binary..."

# Build binary
cargo build

echo "*** Setup complete, use localnet.sh in scripts to start a local network."

