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

TMP="$(mktemp)"; trap 'rm -f "$TMP"' EXIT

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

fail=0
declare -a failures
extr=""

while IFS= read -r line; do
  # detect a new benchmark block
  if [[ $line =~ Extrinsic:\ \"benchmark_([[:alnum:]_]+)\" ]]; then
    extr="${BASH_REMATCH[1]}"
    continue
  fi

  # only process the first "Time ~=" line per extrinsic
  if [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]; then
    [[ -z "$extr" ]] && continue

    meas_us="${BASH_REMATCH[1]}"

    # convert microseconds → picoseconds
    meas_ps=$(awk -v u="$meas_us" 'BEGIN { printf("%.0f", u * 1000000) }')

    # extract the hard-coded ps literal for this extrinsic
    code_w=$(
      awk -v extr="$extr" '
        /^\s*#\[pallet::call_index\([0-9]+\)\]/ { w=""; next }
        /Weight::from_parts/ {
          lw=$0
          sub(/.*Weight::from_parts\(\s*/, "", lw)
          sub(/[^0-9_].*$/, "", lw)
          gsub(/_/, "", lw)
          w=lw
          next
        }
        $0 ~ ("pub fn[[:space:]]+" extr "[[:space:]]*\\(") {
          print w
          exit
        }
      ' "$DISPATCH"
    )

    if [[ -z "$code_w" ]]; then
      echo "::error ::[${extr}] ❌ could not extract code-weight"
      failures+=("[${extr}] missing code-weight")
      fail=1
      extr=""
      continue
    fi

    # compute drift percentage
    drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN { printf("%.1f", (a-b)/b*100) }')
    abs_drift=$(awk -v d="$drift" 'BEGIN { if (d<0) d=-d; printf("%.1f", d) }')

    # flag if outside threshold
    too_big=$(awk -v d="$abs_drift" -v t="$THRESHOLD" 'BEGIN { print (d>t) ? 1 : 0 }')

    if [[ "$too_big" -eq 1 ]]; then
      echo "::error ::[${extr}] code=${code_w}, meas_ps=${meas_ps}, drift=${drift}% (>±${THRESHOLD}%)"
      failures+=("[${extr}] code=${code_w}, meas_ps=${meas_ps}, drift=${drift}%")
      fail=1
    else
      echo "[ok] ${extr}: drift ${drift}% (code=${code_w}, meas_ps=${meas_ps})"
    fi

    # clear extr so we only handle one Time~= per block
    extr=""
  fi
done < "$TMP"

echo
if (( fail )); then
  echo "❌ Benchmark regressions detected:"
  for e in "${failures[@]}"; do
    echo "  • $e"
  done
  exit 1
else
  echo "✅ All benchmarks within ±${THRESHOLD}%."
  exit 0
fi
