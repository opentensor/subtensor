#!/usr/bin/env bash
set -euo pipefail

VERSION="v0.1.9"
BIN_URL="https://github.com/UniqueNetwork/baedeker/releases/download/${VERSION}/baedeker"
INSTALL_PATH="/usr/local/bin/baedeker"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "⬇️ Downloading baedeker ${VERSION}..."

curl -fsSL "$BIN_URL" -o "$TMP_DIR/baedeker"

chmod +x "$TMP_DIR/baedeker"

sudo mv "$TMP_DIR/baedeker" "$INSTALL_PATH"

echo "✅ baedeker installed at $INSTALL_PATH"

baedeker --version