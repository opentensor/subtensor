#!/bin/bash
# This file patches the code in the repository to create a docker image with the ability to run tests in non-fast-runtime
# mode.

set -e

DurationOfStartCall="runtime/src/lib.rs"
DefaultPendingCooldown="pallets/subtensor/src/lib.rs"
SetChildren="pallets/subtensor/src/utils/rate_limiting.rs"

# Checkers
if ! grep -q 'pub const DurationOfStartCall: u64' "$DurationOfStartCall"; then
  echo "Error: Target string not found in $DurationOfStartCall"
  exit 1
fi

if ! grep -q 'pub fn DefaultPendingCooldown<T: Config>() -> u64 {' "$DefaultPendingCooldown"; then
  echo "Error: Target function not found in $DefaultPendingCooldown"
  exit 1
fi

if ! grep -q 'Self::SetChildren => 150, // 30 minutes' "$SetChildren"; then
  echo "Error: Target string not found in $SetChildren"
  exit 1
fi

# replace
perl -0777 -i -pe 's|pub const DurationOfStartCall: u64 = prod_or_fast!\(7 \* 24 \* 60 \* 60 / 12, 10\);|pub const DurationOfStartCall: u64 = prod_or_fast!(5, 10);|' "$DurationOfStartCall"
perl -0777 -i -pe 's|pub fn DefaultPendingCooldown<T: Config>\(\) -> u64 \{\s*prod_or_fast!\(7_200, 15\)\s*\}|pub fn DefaultPendingCooldown<T: Config>() -> u64 {\n        prod_or_fast!(15, 15)\n    }|g' "$DefaultPendingCooldown"
perl -0777 -i -pe 's|Self::SetChildren => 150, // 30 minutes|Self::SetChildren => 30, // 6 min|' "$SetChildren"

echo "Patch applied successfully."
