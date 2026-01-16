#!/usr/bin/env bash
set -euo pipefail

# ----------------------------------------
# Check setup
# ----------------------------------------

command -v baedeker >/dev/null 2>&1 || {
  echo "❌ baedeker is not installed"
  exit 1
}

# ----------------------------------------
# Paths
# ----------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(realpath "$SCRIPT_DIR/..")"

BDK_ENV_DIR="$SCRIPT_DIR/.bdk-env"
VENDOR_DIR="$SCRIPT_DIR/vendor"
SECRET_DIR="$BDK_ENV_DIR/secret"

# ----------------------------------------
# Environment defaults
# ----------------------------------------

export RUST_LOG="${RUST_LOG:-info}"
export CHAINQL_WORKERS="${CHAINQL_WORKERS:-2}"
export CHAINQL_KEYS_CHUNK_SIZE="${CHAINQL_KEYS_CHUNK_SIZE:-20000}"

echo "🧩 Generating chain spec"
echo "  RUST_LOG=$RUST_LOG"
echo "  CHAINQL_WORKERS=$CHAINQL_WORKERS"
echo "  CHAINQL_KEYS_CHUNK_SIZE=$CHAINQL_KEYS_CHUNK_SIZE"

# ----------------------------------------
# Prepare .bdk-env structure
# ----------------------------------------

echo "📁 Preparing .bdk-env directory structure..."

mkdir -p \
  "$BDK_ENV_DIR" \
  "$VENDOR_DIR" \
  "$SECRET_DIR" \
  "$BDK_ENV_DIR/specs" \
  "$BDK_ENV_DIR/discover.env"

# ----------------------------------------
# Generate spec via baedeker
# ----------------------------------------

echo "🚀 Running baedeker..."

baedeker \
  --spec=docker \
  -J"$VENDOR_DIR" \
  --generator=docker_compose="$BDK_ENV_DIR" \
  --generator=docker_compose_discover="$BDK_ENV_DIR/discover.env" \
  --secret=file="$SECRET_DIR" \
  --tla-str=relay_spec=rococo-local \
  --tla-str=repoDir="$REPO_DIR" \
  --input-modules='lib:baedeker-library/ops/nginx.libsonnet' \
  --input-modules='lib:baedeker-library/ops/devtools.libsonnet' \
  "$SCRIPT_DIR/forkless-data.jsonnet" \
  --tla-str=forked_spec=subtensor \
  --tla-str=fork_source=wss://entrypoint-finney.opentensor.ai \
  "$SCRIPT_DIR/rewrites.jsonnet"

# ----------------------------------------
# Validate output
# ----------------------------------------

SPEC_PATH="$BDK_ENV_DIR/specs/subtensor.json"

if [[ ! -f "$SPEC_PATH" ]]; then
  echo "❌ Expected spec not found: $SPEC_PATH"
  exit 1
fi

echo "✅ Chain spec generated at:"
echo "   $SPEC_PATH"