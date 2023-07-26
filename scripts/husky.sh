#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Cleaning repository..."

cargo clean -p cargo-husky
cargo clean -p integration-tests

echo "*** Running test to trigger husky hook insertion..."

cargo test -p integration-tests