#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Initializing WASM build environment"

if ! (( ${#CI_PROJECT_NAME} )) ; then
   rustup update stable
fi

rustup target add wasm32v1-none
