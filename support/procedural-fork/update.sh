#!/bin/sh

# Enable error handling
set -e

# Set the repository and tag
REPO_URL="git@github.com:paritytech/polkadot-sdk.git"
POLKADOT_SDK_TAG="v1.16.0-rc1"

# Create a temporary directory for cloning
TMP_DIR=$(mktemp -d)

# Define source and destination directories
SRC_DIR="substrate/frame/support/procedural/src"
DEST_DIR="$(pwd)/src"  # Absolute path to `src` directory of procedural-fork

# Check if DEST_DIR exists
if [ ! -d "$DEST_DIR" ]; then
    echo "Error: Destination directory $DEST_DIR does not exist."
    rm -rf "$TMP_DIR"
    exit 1
fi

# Clone only the required directory from the repository
echo "Cloning $REPO_URL at tag $POLKADOT_SDK_TAG ..."
git clone --depth 1 --branch "$POLKADOT_SDK_TAG" --filter=blob:none --sparse "$REPO_URL" "$TMP_DIR"

# Navigate to the cloned directory
cd "$TMP_DIR"

# Initialize sparse-checkout and set the correct directory
git sparse-checkout init --cone
git sparse-checkout set "$SRC_DIR"

# Debugging: List the contents of the sparse-checked-out directory
echo "Contents of $TMP_DIR/$SRC_DIR after sparse-checkout:"
ls -l "$TMP_DIR/$SRC_DIR" || { echo "Error: Sparse checkout failed, $SRC_DIR not found."; rm -rf "$TMP_DIR"; exit 1; }

# Copy all files from `src` except `lib.rs` to the destination folder
echo "Copying files to $DEST_DIR ..."
rsync -a --exclude='lib.rs' "$TMP_DIR/$SRC_DIR/" "$DEST_DIR/"

# Clean up the temporary directory
rm -rf "$TMP_DIR"

echo "Update completed successfully."
