#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------
# Resolve repo root
# ---------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ---------------------------------------
# Config (overridable via env)
# ---------------------------------------

export RUST_LOG="${RUST_LOG:-info}"
export CHAINQL_WORKERS="${CHAINQL_WORKERS:-2}"
export CHAINQL_KEYS_CHUNK_SIZE="${CHAINQL_KEYS_CHUNK_SIZE:-20000}"

FORK_SOURCE="${FORK_SOURCE:-wss://entrypoint-finney.opentensor.ai}"
FORKED_SPEC="${FORKED_SPEC:-subtensor}"
RELAY_SPEC="${RELAY_SPEC:-rococo-local}"

SPEC_OUTPUT="${REPO_ROOT}/.bdk-env/specs/subtensor.json"

# ---------------------------------------
# Preconditions
# ---------------------------------------

command -v baedeker >/dev/null 2>&1 || {
  echo "❌ baedeker is not installed"
  exit 1
}

[[ -d "${REPO_ROOT}/.bdk-env" ]] || {
  echo "❌ .bdk-env directory not found"
  exit 1
}

# ---------------------------------------
# Logging
# ---------------------------------------

echo "=== Generating Subtensor chain spec ==="
echo "Repo root:               $REPO_ROOT"
echo "Fork source:             $FORK_SOURCE"
echo "Forked spec name:        $FORKED_SPEC"
echo "Relay spec:              $RELAY_SPEC"
echo "Workers:                 $CHAINQL_WORKERS"
echo "Keys chunk size:         $CHAINQL_KEYS_CHUNK_SIZE"
echo "Rust log level:          $RUST_LOG"
echo "Output spec:             $SPEC_OUTPUT"
echo "======================================"

# ---------------------------------------
# Run Baedeker
# ---------------------------------------

baedeker \
  --spec=docker \
  -J"${REPO_ROOT}/vendor/" \
  --generator=docker_compose="${REPO_ROOT}/.bdk-env" \
  --generator=docker_compose_discover="${REPO_ROOT}/.bdk-env/discover.env" \
  --secret=file="${REPO_ROOT}/.bdk-env/secret" \
  --tla-str="relay_spec=${RELAY_SPEC}" \
  --tla-str="repoDir=$(realpath "${REPO_ROOT}")" \
  --input-modules='lib:baedeker-library/ops/nginx.libsonnet' \
  --input-modules='lib:baedeker-library/ops/devtools.libsonnet' \
  "${REPO_ROOT}/forkless-data.jsonnet" \
  --tla-str="forked_spec=${FORKED_SPEC}" \
  --tla-str="fork_source=${FORK_SOURCE}" \
  "${REPO_ROOT}/rewrites.jsonnet"

# ---------------------------------------
# Post-check
# ---------------------------------------

if [[ ! -f "$SPEC_OUTPUT" ]]; then
  echo "❌ Spec generation failed: $SPEC_OUTPUT not found"
  exit 1
fi

echo "✅ Chain spec generated successfully"