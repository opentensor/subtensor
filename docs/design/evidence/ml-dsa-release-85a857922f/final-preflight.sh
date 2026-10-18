#!/bin/bash
set -u
R=/tmp/ms-release-85a857922f
while [ ! -f "$R/additional.exit" ]; do sleep 5; done
cp "$R/preflight.log" "$R/preflight-initial.log"
cp "$R/preflight.exit" "$R/preflight-initial.exit"
bash "$R/preflight.sh"
