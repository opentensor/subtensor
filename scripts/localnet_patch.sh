#!/bin/bash
# This file patches the code in the repository to create a docker image with the ability to run tests in non-fast-runtime
# mode.

set -e

DurationOfStartCall="runtime/src/lib.rs"
DefaultPendingCooldown="pallets/subtensor/src/lib.rs"
SetChildren="pallets/subtensor/src/utils/rate_limiting.rs"

# Checkers
if ! grep -q '7 \* 24 \* 60 \* 60 / 12 // 7 days' "$DurationOfStartCall"; then
  echo "Error: Target string not found in $DurationOfStartCall"
  exit 1
fi

if ! grep -q 'pub fn DefaultPendingCooldown<T: Config>() -> u64 {' "$DefaultPendingCooldown"; then
  echo "Error: Target function not found in $DefaultPendingCooldown"
  exit 1
fi

if ! grep -q 'TransactionType::SetChildren => 150, // 30 minutes' "$SetChildren"; then
  echo "Error: Target string not found in $SetChildren"
  exit 1
fi

# replace
perl -0777 -i -pe 's|7 \* 24 \* 60 \* 60 / 12 // 7 days|5 // Only 5 blocks for tests|' "$DurationOfStartCall"
perl -0777 -i -pe 's|pub fn DefaultPendingCooldown<T: Config>\(\) -> u64 \{\s*if cfg!\(feature = "fast-runtime"\) \{\s*return 15;\s*\}\s*7_200\s*\}|pub fn DefaultPendingCooldown<T: Config>() -> u64 {\n        15\n    }|g' "$DefaultPendingCooldown"
perl -0777 -i -pe 's|TransactionType::SetChildren => 150, // 30 minutes|TransactionType::SetChildren => 15, // 3 min|' "$SetChildren"

echo "Patch applied successfully."
