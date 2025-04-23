#!/usr/bin/env bash
set -euo pipefail

THRESHOLD=10
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISPATCH="$SCRIPT_DIR/../pallets/subtensor/src/macros/dispatches.rs"
RUNTIME_WASM="./target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

if [[ ! -f "$DISPATCH" ]]; then
  echo "❌ ERROR: dispatches.rs not found at $DISPATCH"
  exit 1
fi

echo "Building runtime-benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo
echo "──────────────────────────────────────────"
echo " Running pallet_subtensor benchmarks…"
echo "──────────────────────────────────────────"

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT
./target/production/node-subtensor benchmark pallet \
  --runtime "$RUNTIME_WASM" \
  --genesis-builder=runtime \
  --genesis-builder-preset=benchmark \
  --wasm-execution=compiled \
  --pallet pallet_subtensor \
  --extrinsic "*" \
  --steps 50 \
  --repeat 5 \
| tee "$TMP"

declare -a summary_lines=()
declare -a failures=()
fail=0
extr=""

while IFS= read -r line; do
  # Detect new extrinsic
  if [[ $line =~ Extrinsic:\ \"benchmark_([[:alnum:]_]+)\" ]]; then
    extr="${BASH_REMATCH[1]}"
    continue
  fi

  # Capture first timing line
  if [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]; then
    [[ -z "$extr" ]] && continue

    meas_us="${BASH_REMATCH[1]}"
    meas_ps=$(awk -v u="$meas_us" 'BEGIN{printf("%.0f", u * 1000000)}')

    # Grab Reads/Writes
    meas_reads="" meas_writes=""
    while IFS= read -r sub; do
      [[ $sub =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]] && meas_reads="${BASH_REMATCH[1]}" && continue
      [[ $sub =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && meas_writes="${BASH_REMATCH[1]}" && break
    done

    # Extract code values from dispatches.rs
    code_record=$(
      awk -v extr="$extr" '
        /^\s*#\[pallet::call_index\(/ { next }
        /Weight::from_parts/ { lw=$0; sub(/.*Weight::from_parts\(\s*/, "", lw); sub(/[^0-9_].*$/, "", lw); gsub(/_/, "", lw); w=lw; next }
        /reads_writes\(/ { lw=$0; sub(/.*reads_writes\(/, "", lw); sub(/\).*/, "", lw); split(lw,io,/,/); gsub(/^[ \t]+|[ \t]+$/, "", io[1]); gsub(/^[ \t]+|[ \t]+$/, "", io[2]); r=io[1]; wri=io[2]; next }
        /\.reads\(/   { lw=$0; sub(/.*\.reads\(/, "", lw); sub(/\).*/, "", lw); r=lw; next }
        /\.writes\(/  { lw=$0; sub(/.*\.writes\(/, "", lw); sub(/\).*/, "", lw); wri=lw; next }
        $0 ~ ("pub fn[[:space:]]+" extr "[[:space:]]*\\(") { print w, r, wri; exit }
      ' "$DISPATCH"
    )
    read code_w code_reads code_writes <<<"$code_record"

    # Calculate drift
    drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{printf("%.1f", (a-b)/b*100)}')

    # Store formatted summary
    summary_lines+=("$(printf "%-30s | reads code=%3s meas=%3s | writes code=%3s meas=%3s | weight code=%12s meas=%12s | drift %6s%%" \
      "$extr" "$code_reads" "$meas_reads" "$code_writes" "$meas_writes" "$code_w" "$meas_ps" "$drift")")

    # Validation checks
    [[ -z "$code_w" ]]        && failures+=("[${extr}] missing code weight")       && fail=1
    [[ -z "$meas_reads" ]]    && failures+=("[${extr}] missing measured reads")    && fail=1
    [[ -z "$meas_writes" ]]   && failures+=("[${extr}] missing measured writes")   && fail=1
    (( meas_reads   != code_reads ))  && failures+=("[${extr}] reads mismatch code=${code_reads}, meas=${meas_reads}")   && fail=1
    (( meas_writes  != code_writes )) && failures+=("[${extr}] writes mismatch code=${code_writes}, meas=${meas_writes}") && fail=1
    [[ "$code_w" == "0" ]]    && failures+=("[${extr}] zero code weight")           && fail=1
    abs_drift=$(awk -v d="$drift" 'BEGIN{if(d<0)d=-d;printf("%.1f",d)}')
    (( $(awk -v d="$abs_drift" -v t="$THRESHOLD" 'BEGIN{print d>t}') )) && failures+=("[${extr}] drift ${drift}%") && fail=1

    extr=""
  fi
done < "$TMP"

echo
echo "Benchmark Summary:"
for l in "${summary_lines[@]}"; do
  echo "$l"
done

if (( fail )); then
  echo
  echo "❌ Detected issues:"
  for e in "${failures[@]}"; do
    echo "  • $e"
  done
  exit 1
else
  echo
  echo "✅ All benchmarks within ±${THRESHOLD}% drift."
  exit 0
fi
