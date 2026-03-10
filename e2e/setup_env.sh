#!/bin/bash
#
# Verify and set up the development environment for e2e tests.
# Checks for nvm, the correct Node.js version (.nvmrc), pnpm, jq, and yq.
# Installs what it can, exits with an error for what it cannot.
#
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
NVMRC="$SCRIPT_DIR/.nvmrc"

check() {
  local name="$1"
  if command -v "$name" &>/dev/null; then
    echo "  $name: $(command -v "$name")"
    return 0
  fi
  return 1
}

echo "==> Checking prerequisites..."

# -- nvm --
NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
if [ -s "$NVM_DIR/nvm.sh" ]; then
  echo "  nvm: $NVM_DIR"
  # shellcheck source=/dev/null
  source "$NVM_DIR/nvm.sh"
else
  echo "ERROR: nvm not found. Install it from https://github.com/nvm-sh/nvm"
  exit 1
fi

# -- Node.js (version from .nvmrc) --
REQUIRED_NODE="$(cat "$NVMRC")"
if ! nvm ls "$REQUIRED_NODE" &>/dev/null; then
  echo "  Node $REQUIRED_NODE not installed, installing..."
  nvm install "$REQUIRED_NODE"
fi
nvm use "$REQUIRED_NODE"
echo "  node: $(node --version)"

# -- pnpm --
if ! check pnpm; then
  echo "  pnpm not found, installing..."
  npm install -g pnpm
  check pnpm || { echo "ERROR: Failed to install pnpm"; exit 1; }
fi

# -- jq --
if ! check jq; then
  echo "ERROR: jq not found. Install it:"
  echo "  macOS:  brew install jq"
  echo "  Ubuntu: sudo apt install jq"
  exit 1
fi

# -- yq --
if ! check yq; then
  echo "ERROR: yq not found. Install it:"
  echo "  macOS:  brew install yq"
  echo "  Ubuntu: sudo snap install yq"
  exit 1
fi

echo "==> All prerequisites satisfied."
