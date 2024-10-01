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
PARENT_DIR="$(dirname "$DEST_DIR")"  # Get the parent directory of DEST_DIR

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

# Copy all files from `src` except `$DEST_DIR/lib.rs` to the destination folder
echo "Copying files to $DEST_DIR ..."
rsync -a --exclude='lib.rs' "$TMP_DIR/$SRC_DIR/" "$DEST_DIR/"

# Prepend only `#![cfg(not(doc))]` to the top of each Rust file, except the `lib.rs` in $DEST_DIR
find "$DEST_DIR" -name '*.rs' -not -path "$DEST_DIR/lib.rs" | while read -r file; do
    echo "Prepending configuration to $file ..."
    # Use awk to prepend only `#![cfg(not(doc))]`
    awk 'BEGIN {print "#![cfg(not(doc))]"} {print}' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
done

# Remove all `#[cfg(test)]` lines from `pallet/parse/mod.rs`
MOD_RS="$DEST_DIR/pallet/parse/mod.rs"
if [ -f "$MOD_RS" ]; then
    echo "Removing #[cfg(test)] from $MOD_RS ..."
    grep -v '#\[cfg(test)\]' "$MOD_RS" > "$MOD_RS.tmp" && mv "$MOD_RS.tmp" "$MOD_RS"
fi

# Replace all #[test] with #[test]\n#[ignore] using awk
echo "Replacing #[test] with #[ignore] ..."
find "$DEST_DIR" -name '*.rs' | while read -r file; do
    awk '{if ($0 == "#[test]") print $0 "\n#[ignore]"; else print $0}' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
done

# Change directory to the parent of $DEST_DIR to run cargo fmt
echo "Changing directory to $PARENT_DIR and running cargo fmt --all ..."
cd "$PARENT_DIR" && cargo fmt --all

# Clean up the temporary directory
rm -rf "$TMP_DIR"

echo "Update and formatting completed successfully."
