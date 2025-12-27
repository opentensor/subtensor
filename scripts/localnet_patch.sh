#!/bin/bash
# This file patches the code in the repository to create a docker image with the ability to run tests in non-fast-runtime
# mode.

set -e

# Function to check for a pattern and apply a replacement
# Args: file_path, search_pattern, replacement_pattern, description
patch_file() {
  local file_path="$1"
  local search_pattern="$2"
  local replacement_pattern="$3"
  local description="$4"

  # Check if the search pattern exists
  if ! grep -qF "$search_pattern" "$file_path" 2>/dev/null && ! grep -qP "$search_pattern" "$file_path" 2>/dev/null; then
    echo "Error: Target pattern '$search_pattern' not found in $file_path"
    echo "Description: $description"
    echo "This may indicate the codebase has changed. Please verify the target code exists."
    exit 1
  fi

  # Apply the replacement
  if ! perl -0777 -i -pe "$replacement_pattern" "$file_path"; then
    echo "Error: Failed to apply replacement in $file_path"
    echo "Description: $description"
    exit 1
  fi
}

echo "Applying patches..."

# Patch 1: InitialStartCallDelay
patch_file \
  "runtime/src/lib.rs" \
  "pub const InitialStartCallDelay: u64" \
  's|pub const InitialStartCallDelay: u64 = prod_or_fast!\(7 \* 24 \* 60 \* 60 / 12, 10\);|pub const InitialStartCallDelay: u64 = prod_or_fast!(5, 10);|' \
  "Reduce InitialStartCallDelay for local testing"

# Patch 2: DefaultPendingCooldown
patch_file \
  "pallets/subtensor/src/lib.rs" \
  "pub fn DefaultPendingCooldown<T: Config>() -> u64 {" \
  's|pub fn DefaultPendingCooldown<T: Config>\(\) -> u64 \{\s*prod_or_fast!\(7_200, 15\)\s*\}|pub fn DefaultPendingCooldown<T: Config>() -> u64 {\n        prod_or_fast!(15, 15)\n    }|g' \
  "Reduce DefaultPendingCooldown for local testing"

# Patch 3: SetChildren rate limit
patch_file \
  "pallets/subtensor/src/utils/rate_limiting.rs" \
  "Self::SetChildren => 150, // 30 minutes" \
  's|Self::SetChildren => 150, // 30 minutes|Self::SetChildren => 15, // 3 min|' \
  "Reduce SetChildren rate limit for local testing"

echo "✓ All patches applied successfully."
