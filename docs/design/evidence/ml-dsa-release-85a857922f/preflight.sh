#!/bin/bash
set -u
cd /home/johnreed/Desktop/work/subtensor
export PATH="/tmp/ms-release-85a857922f/bin:/tmp/ms-reference-39060946210/bin:$PATH"
export PREFLIGHT_PUSH_ACTOR=UnArbosSix
export PREFLIGHT_BASE=6a972693ae9580541225bd37696ac2fee26b8ae9
export PREFLIGHT_CACHE_DIR=/tmp/ms-release-85a857922f/mainnet
export PYTHONPATH=/tmp/ms-release-85a857922f/native:$PWD/sdk/python
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=4
scripts/preflight.sh --all > /tmp/ms-release-85a857922f/preflight.log 2>&1
status=$?
printf '%s\n' "$status" > /tmp/ms-release-85a857922f/preflight.exit
exit "$status"
