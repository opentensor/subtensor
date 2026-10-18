#!/bin/bash
set -u
cd /home/johnreed/Desktop/work/subtensor/sdk/python
export PYTHONPATH=/tmp/ms-release-85a857922f/native:$PWD
.venv/bin/python -m pytest tests/unit -n 8 -q --junitxml=/tmp/ms-release-85a857922f/sdk.xml > /tmp/ms-release-85a857922f/sdk.log 2>&1
status=$?
printf '%s\n' "$status" > /tmp/ms-release-85a857922f/sdk.exit
uv run --no-sync ty check --exit-zero-on-warning bittensor > /tmp/ms-release-85a857922f/sdk-types.log 2>&1
printf '%s\n' "$?" > /tmp/ms-release-85a857922f/sdk-types.exit
exit "$status"
