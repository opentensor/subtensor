#!/bin/bash
# This file patches the code in the repository to create a docker image with the ability to run tests in non-fast-runtime
# mode.

set -e

InitialStartCallDelay="runtime/src/lib.rs"
DefaultPendingCooldown="pallets/subtensor/src/lib.rs"
SetChildren="pallets/subtensor/src/utils/rate_limiting.rs"

# Checkers
if ! grep -q 'pub const InitialStartCallDelay: u64' "$InitialStartCallDelay"; then
  echo "Error: Target string 'pub const InitialStartCallDelay: u64' not found in $InitialStartCallDelay"
  echo "This may indicate the codebase has changed. Please verify the target code exists."
  exit 1
fi

if ! grep -q 'pub fn DefaultPendingCooldown<T: Config>() -> u64 {' "$DefaultPendingCooldown"; then
  echo "Error: Target function 'DefaultPendingCooldown' not found in $DefaultPendingCooldown"
  echo "This may indicate the codebase has changed. Please verify the target code exists."
  exit 1
fi

if ! grep -q 'Self::SetChildren => 150, // 30 minutes' "$SetChildren"; then
  echo "Error: Target string 'Self::SetChildren => 150' not found in $SetChildren"
  echo "This may indicate the codebase has changed. Please verify the target code exists."
  exit 1
fi

# Replace - with error handling
echo "Applying patches..."

if ! perl -0777 -i -pe 's|pub const InitialStartCallDelay: u64 = prod_or_fast!\(7 \* 24 \* 60 \* 60 / 12, 10\);|pub const InitialStartCallDelay: u64 = prod_or_fast!(5, 10);|' "$InitialStartCallDelay"; then
  echo "Error: Failed to replace InitialStartCallDelay in $InitialStartCallDelay"
  exit 1
fi

if ! perl -0777 -i -pe 's|pub fn DefaultPendingCooldown<T: Config>\(\) -> u64 \{\s*prod_or_fast!\(7_200, 15\)\s*\}|pub fn DefaultPendingCooldown<T: Config>() -> u64 {\n        prod_or_fast!(15, 15)\n    }|g' "$DefaultPendingCooldown"; then
  echo "Error: Failed to replace DefaultPendingCooldown in $DefaultPendingCooldown"
  exit 1
fi

if ! perl -0777 -i -pe 's|Self::SetChildren => 150, // 30 minutes|Self::SetChildren => 15, // 3 min|' "$SetChildren"; then
  echo "Error: Failed to replace SetChildren rate limit in $SetChildren"
  exit 1
fi

echo "✓ All patches applied successfully."
