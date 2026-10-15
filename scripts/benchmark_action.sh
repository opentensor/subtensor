#!/usr/bin/env bash
set -euo pipefail

# CI benchmark run on the reference machine.
#
# Full mode (BENCHMARK_TARGETS unset): measure every benchmark, compare with the
# committed weights, and patch the ones that drifted beyond THRESHOLD.
#
# Targeted mode (BENCHMARK_TARGETS set, one `pallet:bench,bench` per line, as
# printed by scripts/benchmark_targets.py): measure only those benchmarks and
# patch them with the new measurement unconditionally, because their committed
# weight was written for a different benchmark or by hand.
#
# Exit: 0 = weights unchanged, 1 = error, 2 = patch in .bench_patch/

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

NODE_BIN="${NODE_BIN:-$ROOT_DIR/target/production/node-subtensor}"
RUNTIME_WASM="${RUNTIME_WASM:-$ROOT_DIR/target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm}"
TEMPLATE="$ROOT_DIR/.maintain/frame-weight-template.hbs"
WEIGHT_CMP="${WEIGHT_CMP:-$ROOT_DIR/target/production/weight-compare}"

PATCH_DIR="$ROOT_DIR/.bench_patch"
THRESHOLD="${THRESHOLD:-75}"
STEPS="${STEPS:-50}"
REPEAT="${REPEAT:-20}"
# Utility batches are low-amplitude microbenchmarks; median-slopes avoids noisy intercept drift.
UTILITY_OUTPUT_ANALYSIS="${UTILITY_OUTPUT_ANALYSIS:-median-slopes}"

die() { echo "ERROR: $1" >&2; exit 1; }

selective_patch_weights_file() {
  python3 "$SCRIPT_DIR/benchmark_action.py" selective-patch "$@"
}

# ── Auto-discover pallets ────────────────────────────────────────────────────
declare -A OUTPUTS
while read -r name path; do
  OUTPUTS[$name]="$path"
done < <("$SCRIPT_DIR/discover_pallets.sh")

(( ${#OUTPUTS[@]} > 0 )) || die "no benchmarked pallets found"

# Pallet -> comma-separated benchmarks to measure ("*" = all).
declare -A SELECTED
if [[ -n "${BENCHMARK_TARGETS:-}" ]]; then
  while IFS=: read -r pallet benchmarks; do
    [[ -n "$pallet" ]] || continue
    [[ -n "${OUTPUTS[$pallet]:-}" ]] || die "unknown benchmarked pallet in BENCHMARK_TARGETS: $pallet"
    [[ "$benchmarks" =~ ^[A-Za-z0-9_]+(,[A-Za-z0-9_]+)*$ ]] || die "invalid benchmark list for $pallet: $benchmarks"
    SELECTED[$pallet]="$benchmarks"
  done <<< "$BENCHMARK_TARGETS"
  (( ${#SELECTED[@]} > 0 )) || die "BENCHMARK_TARGETS has no entries"
  echo "Targeted run:"; printf '  %s\n' $BENCHMARK_TARGETS
else
  for pallet in "${!OUTPUTS[@]}"; do SELECTED[$pallet]="*"; done
fi

mkdir -p "$PATCH_DIR"

[[ -x "$NODE_BIN" ]] || die "node binary not found at $NODE_BIN"
[[ -f "$RUNTIME_WASM" ]] || die "runtime WASM not found at $RUNTIME_WASM"
[[ -x "$WEIGHT_CMP" ]] || die "weight-compare not found at $WEIGHT_CMP"

PATCHED=()
SUMMARY=()
FAILED=0

for pallet in "${!SELECTED[@]}"; do
  output="${OUTPUTS[$pallet]}"
  extrinsics="${SELECTED[$pallet]}"
  committed="$ROOT_DIR/$output"
  tmp=$(mktemp)

  echo ""
  echo "════ $pallet ($extrinsics) ════"

  analysis_args=()
  if [[ -n "${OUTPUT_ANALYSIS:-}" ]]; then
    analysis_args+=(--output-analysis="$OUTPUT_ANALYSIS")
  elif [[ "$pallet" == "pallet_subtensor_utility" ]]; then
    analysis_args+=(--output-analysis="$UTILITY_OUTPUT_ANALYSIS")
  fi

  if ! "$NODE_BIN" benchmark pallet \
    --runtime="$RUNTIME_WASM" \
    --genesis-builder=runtime \
    --genesis-builder-preset=benchmark \
    --wasm-execution=compiled \
    --pallet="$pallet" \
    --extrinsic="$extrinsics" \
    --steps="$STEPS" \
    --repeat="$REPEAT" \
    --no-storage-info \
    --no-min-squares \
    --no-median-slopes \
    "${analysis_args[@]}" \
    --output="$tmp" \
    --template="$TEMPLATE" 2>&1; then
    SUMMARY+=("$pallet: FAILED"); FAILED=1; rm -f "$tmp"; continue
  fi

  if [[ ! -f "$committed" ]]; then
    cp "$tmp" "$committed"; PATCHED+=("$output"); SUMMARY+=("$pallet: NEW")
    rm -f "$tmp"; continue
  fi

  if [[ "$extrinsics" != "*" ]]; then
    # The comparison is informational here: the selected weights are stale by
    # construction, so the measurement replaces them whatever the drift.
    IFS=, read -r -a measured <<< "$extrinsics"
    missing=()
    for name in "${measured[@]}"; do
      grep -Eq "^[[:space:]]*fn ${name}\(" "$tmp" || missing+=("$name")
    done
    if (( ${#missing[@]} )); then
      SUMMARY+=("$pallet: FAILED (no measurement for ${missing[*]})"); FAILED=1
      rm -f "$tmp"; continue
    fi
    merged=$(mktemp)
    cp "$committed" "$merged"
    # Exit 1 here only means every measured function is byte-identical.
    selective_patch_weights_file "$merged" "$tmp" "${measured[@]}" || true
    "$WEIGHT_CMP" --old "$committed" --new "$merged" --threshold "$THRESHOLD" 2>&1 || true
    if cmp -s "$merged" "$committed"; then
      SUMMARY+=("$pallet: OK (measurement matches committed weights)")
    else
      cp "$merged" "$committed"
      PATCHED+=("$output")
      SUMMARY+=("$pallet: UPDATED ${measured[*]}")
    fi
    rm -f "$merged" "$tmp"
    continue
  fi

  compare_log=$(mktemp)
  if "$WEIGHT_CMP" --old "$committed" --new "$tmp" --threshold "$THRESHOLD" 2>&1 | tee "$compare_log"; then
    rc=0
  else
    rc=${PIPESTATUS[0]}
  fi

  if (( rc == 2 )); then
    drifted_benchmarks=()
    while IFS= read -r benchmark_name; do
      drifted_benchmarks+=("$benchmark_name")
    done < <(python3 "$SCRIPT_DIR/benchmark_action.py" drifted-benchmarks "$compare_log")

    if (( ${#drifted_benchmarks[@]} == 0 )); then
      SUMMARY+=("$pallet: COMPARE FAILED"); FAILED=1
    else
      selective_patch_weights_file "$committed" "$tmp" "${drifted_benchmarks[@]}"
      PATCHED+=("$output")
      SUMMARY+=("$pallet: UPDATED ${#drifted_benchmarks[@]} benchmark(s): ${drifted_benchmarks[*]}")
    fi
  elif (( rc == 0 )); then
    SUMMARY+=("$pallet: OK")
  else
    SUMMARY+=("$pallet: COMPARE FAILED"); FAILED=1
  fi
  rm -f "$compare_log" "$tmp"
done

echo ""; printf '%s\n' "${SUMMARY[@]}"

# Preserve successful reference measurements even if another pallet fails.
# The run remains red; a partial patch is evidence, not a passing gate.
if (( ${#PATCHED[@]} == 0 )); then
  printf '%s\n' "${SUMMARY[@]}" > "$PATCH_DIR/summary.txt"
  (( FAILED )) && exit 1
  echo "Weights unchanged."
  exit 0
fi

# Prepare patch
cd "$ROOT_DIR"
git add "${PATCHED[@]}"
{ echo "Head SHA: $(git rev-parse HEAD)"; echo ""; printf '%s\n' "${SUMMARY[@]}"; echo ""; git diff --cached --stat; } > "$PATCH_DIR/summary.txt"
git diff --cached --binary > "$PATCH_DIR/benchmark_patch.diff"
git reset HEAD -- "${PATCHED[@]}" >/dev/null 2>&1 || true
if (( FAILED )); then
  echo "Partial reference patch saved at $PATCH_DIR/benchmark_patch.diff; failed pallets remain unmeasured."
  exit 1
fi
echo "Patch ready at $PATCH_DIR/benchmark_patch.diff"
exit 2
