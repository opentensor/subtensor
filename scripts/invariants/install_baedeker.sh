#!/usr/bin/env bash
set -euo pipefail

# -----------------------------
# Configuration
# -----------------------------

# Set to a tag like "v0.7.3" to pin, or leave empty for latest
BAEDEKER_VERSION="v0.1.9"

INSTALL_DIR="/usr/local/bin"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Installing baedeker (version: $BAEDEKER_VERSION)"

# -----------------------------
# Resolve release metadata
# -----------------------------

if [[ "$BAEDEKER_VERSION" == "latest" ]]; then
  API_URL="https://api.github.com/repos/UniqueNetwork/baedeker/releases/latest"
else
  API_URL="https://api.github.com/repos/UniqueNetwork/baedeker/releases/tags/${BAEDEKER_VERSION}"
fi

ASSET_URL=$(
  curl -fsSL "$API_URL" |
    jq -r '.assets[]
      | select(.name | test("linux.*(x86_64|amd64).*tar.gz"))
      | .browser_download_url' |
    head -n 1
)

if [[ -z "$ASSET_URL" || "$ASSET_URL" == "null" ]]; then
  echo "❌ Failed to find Linux baedeker release asset"
  exit 1
fi

echo "Downloading: $ASSET_URL"

# -----------------------------
# Download & install
# -----------------------------

curl -fsSL "$ASSET_URL" -o "$TMP_DIR/baedeker.tar.gz"

tar -xzf "$TMP_DIR/baedeker.tar.gz" -C "$TMP_DIR"

if [[ ! -f "$TMP_DIR/baedeker" ]]; then
  echo "❌ baedeker binary not found in archive"
  exit 1
fi

chmod +x "$TMP_DIR/baedeker"

sudo mv "$TMP_DIR/baedeker" "${INSTALL_DIR}/baedeker"

# -----------------------------
# Verification
# -----------------------------

echo "Installed baedeker to ${INSTALL_DIR}/baedeker"
baedeker --version

echo "✅ baedeker installation complete"