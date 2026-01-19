#!/usr/bin/env bash
set -euo pipefail

SPECGEN_IMAGE="ghcr.io/opentensor/mainnet-genspec:latest"

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
BDK_ENV_DIR="$SCRIPT_DIR/.bdk-env"
SECRET_DIR="$BDK_ENV_DIR/secret"

# ----------------------------------------
# Environment defaults
# ----------------------------------------
echo "🧩 Generating chain spec"
echo "  RUST_LOG=$RUST_LOG"

# ----------------------------------------
# Prepare .bdk-env structure
# ----------------------------------------
echo "📁 Preparing .bdk-env directory structure..."
mkdir -p \
  "$BDK_ENV_DIR" \
  "$SECRET_DIR" \
  "$BDK_ENV_DIR/specs"

# ----------------------------------------
# Generate spec via baedeker
# ----------------------------------------
echo "🚀 Storing state..."
docker run --rm \
  -v "$(command -v baedeker):/usr/local/bin/baedeker:ro" \
  -v "$BDK_ENV_DIR:/app/.bdk-env" \
  "$SPECGEN_IMAGE"

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