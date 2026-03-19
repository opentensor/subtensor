#!/usr/bin/env bash
set -eo pipefail

# Generate or update a placeholder weights.rs for a pallet.
#
# Usage:
#   ./scripts/benchmark_stub.sh pallet_drand
#   ./scripts/benchmark_stub.sh pallet_subtensor
#
# Automatically discovers the benchmarking file and weights.rs path
# by walking the pallets/ directory.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

die() { echo "ERROR: $1" >&2; exit 1; }

(( $# >= 1 )) || die "usage: $0 <pallet_name>  (e.g. pallet_drand)"
PALLET="$1"

# Find the benchmarking file by searching pallets/
BENCH=""
for candidate in "$ROOT_DIR"/pallets/*/src/benchmarking.rs "$ROOT_DIR"/pallets/*/src/benchmarks.rs "$ROOT_DIR"/pallets/*/src/pallet/benchmarking.rs; do
  [[ -f "$candidate" ]] || continue
  # Check if this file's Cargo.toml declares the right pallet name
  pallet_dir="$(dirname "$(dirname "$candidate")")"
  # Handle src/pallet/ nesting
  if [[ "$(basename "$pallet_dir")" == "pallet" ]]; then
    pallet_dir="$(dirname "$pallet_dir")"
  fi
  cargo_toml="$pallet_dir/Cargo.toml"
  [[ -f "$cargo_toml" ]] || continue
  crate_name=$(grep -m1 '^name' "$cargo_toml" | sed 's/.*"\(.*\)".*/\1/' | tr '-' '_')
  if [[ "$crate_name" == "$PALLET" ]]; then
    BENCH="$candidate"
    OUTPUT="$pallet_dir/src/weights.rs"
    break
  fi
done

[[ -n "$BENCH" ]] || die "could not find benchmarking file for '$PALLET' in pallets/"

cargo run -p subtensor-weight-tools --bin weight-stub -- \
  --benchmarks "$BENCH" \
  --output "$OUTPUT" \
  --pallet "$PALLET"
