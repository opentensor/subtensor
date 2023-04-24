#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Initializing WASM build environment"

# remove old nightlies
rustup toolchain remove nightly

if ! (( ${#CI_PROJECT_NAME} )) ; then
   rustup update nightly-2023-01-18
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain nightly-2023-01-18
