#!/bin/bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT_TEST_DIR="$ROOT_DIR/contract-tests"
LOCALNET_LOG="${LOCALNET_LOG:-$ROOT_DIR/contract-tests-localnet.log}"
LOCALNET_START_TIMEOUT="${LOCALNET_START_TIMEOUT:-300}"
CONTRACT_TEST_FILE_ATTEMPTS="${CONTRACT_TEST_FILE_ATTEMPTS:-1}"

echo "start run-ci.sh"

cleanup() {
  pkill node-subtensor >/dev/null 2>&1 || true
}

dump_localnet_log() {
  if [ -f "$LOCALNET_LOG" ]; then
    echo "---- last 200 localnet log lines ----"
    tail -n 200 "$LOCALNET_LOG" || true
    echo "-------------------------------------"
  fi
}

trap cleanup EXIT

cd "$CONTRACT_TEST_DIR/bittensor"

rustup component add rust-src
if ! command -v cargo-contract >/dev/null 2>&1; then
  cargo install cargo-contract
else
  echo "cargo-contract already installed"
fi
cargo contract build --release

cd "$ROOT_DIR"

scripts/localnet.sh --build-only
BUILD_BINARY=0 scripts/localnet.sh >"$LOCALNET_LOG" 2>&1 &

for i in $(seq 1 "$LOCALNET_START_TIMEOUT"); do
  if nc -z localhost 9944; then
    echo "node subtensor is running after $i seconds"
    break
  fi
  sleep 1
done

if ! nc -z localhost 9944; then
  echo "node subtensor did not start within ${LOCALNET_START_TIMEOUT}s"
  dump_localnet_log
  exit 1
fi

sleep 10

if ! nc -z localhost 9944; then
  echo "node subtensor exited, port not available"
  dump_localnet_log
  exit 1
fi

cd "$CONTRACT_TEST_DIR"

# Required for papi in get-metadata.sh; yarn install cannot run before papi
# because package.json references the generated descriptors package.
npm i -g polkadot-api

if ! command -v yarn >/dev/null 2>&1; then
  npm install --global yarn
fi

bash get-metadata.sh

sleep 5

yarn install --frozen-lockfile

if [ "$#" -gt 0 ]; then
  test_files=("$@")
else
  mapfile -t test_files < <(find test -maxdepth 1 -name "*.ts" -print | sort)
fi

failed_files=()
for test_file in "${test_files[@]}"; do
  echo "Running $test_file"
  passed=0

  for attempt in $(seq 1 "$CONTRACT_TEST_FILE_ATTEMPTS"); do
    if [ "$attempt" -gt 1 ]; then
      echo "Retrying $test_file (attempt $attempt/$CONTRACT_TEST_FILE_ATTEMPTS)"
    fi

    if yarn run test:ci:file "$test_file"; then
      passed=1
      break
    fi
  done

  if [ "$passed" -ne 1 ]; then
    failed_files+=("$test_file")
  fi
done

if [ "${#failed_files[@]}" -gt 0 ]; then
  echo "Contract test files failed:"
  printf ' - %s\n' "${failed_files[@]}"
  dump_localnet_log
  exit 1
fi

exit 0
