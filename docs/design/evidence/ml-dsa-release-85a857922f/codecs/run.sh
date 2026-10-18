#!/bin/bash
set -euo pipefail
R=/tmp/ms-release-85a857922f
P="$PWD/sdk/python/.venv/bin/python"
export PYTHONPATH="$R/native:$PWD/sdk/python"
export E2E_WALLET_ENDPOINT=ws://127.0.0.1:27344
"$P" docs/design/evidence/ml-dsa-localnet-blocks.py "$R/codecs/capture.json" > "$R/codecs/capture.log" 2>&1
node "$R/codecs/check.cjs" "$R/codecs/capture.json" "$R/codecs/javascript.json" > "$R/codecs/javascript.log" 2>&1
"$P" docs/design/evidence/ml-dsa-localnet-native.py "$R/codecs/capture.json" "$R/codecs/native-current.json" current-head > "$R/codecs/native-current.log" 2>&1
PYTHONPATH=/tmp/pr3220-deployment-a673ce10/installed "$P" docs/design/evidence/ml-dsa-localnet-native.py "$R/codecs/capture.json" "$R/codecs/native-legacy.json" preserved-pre-mldsa > "$R/codecs/native-legacy.log" 2>&1
echo 0 > "$R/codecs/exit"
