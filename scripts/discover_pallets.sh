#!/usr/bin/env bash
# Auto-discover benchmarked pallets.
#
# Finds all pallets under pallets/ that have both:
#   - src/benchmarking.rs (or src/benchmarks.rs)
#   - src/weights.rs
#
# Outputs one line per pallet: "pallet_name pallets/<dir>/src/weights.rs"
# The pallet name is derived from the Cargo.toml `name` field with dashes -> underscores.

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

for dir in "$ROOT_DIR"/pallets/*/; do
  [ -f "$dir/src/weights.rs" ] || continue
  [ -f "$dir/src/benchmarking.rs" ] || [ -f "$dir/src/benchmarks.rs" ] || continue

  name=$(grep '^name' "$dir/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/' | tr '-' '_')
  relpath="pallets/$(basename "$dir")/src/weights.rs"
  echo "$name $relpath"
done
