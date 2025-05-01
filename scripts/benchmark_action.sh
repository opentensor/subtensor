#!/usr/bin/env bash
set -euo pipefail

# Max allowed drift (%)
THRESHOLD=10

# Resolve script paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISPATCH="$SCRIPT_DIR/../pallets/subtensor/src/macros/dispatches.rs"
RUNTIME_WASM="./target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

# Sanity check
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

MAX_RETRIES=3
attempt=1

################################################################################
# Helper function to "finalize" an extrinsic. We look up the code-side
# reads/writes/weight in dispatches.rs, then compare to measured values.
################################################################################
summary_lines=()
failures=()
fail=0

function process_extr() {
  local e="$1"
  local us="$2"
  local rd="$3"
  local wr="$4"

  # If any piece is empty, skip
  if [[ -z "$e" || -z "$us" || -z "$rd" || -z "$wr" ]]; then
    return
  fi

  # Convert microseconds to picoseconds
  local meas_ps
  meas_ps=$(awk -v x="$us" 'BEGIN{printf("%.0f", x * 1000000)}')

  # ---------------------------------------------------------------------------
  # Code-side lookup from $DISPATCH
  # ---------------------------------------------------------------------------
  # We find the matching "pub fn <extr>(" line in dispatches.rs,
  # then parse the preceding Weight::from_parts, .reads, .writes lines.

  local code_record
  code_record=$(awk -v extr="$e" '
    /^\s*#\[pallet::call_index\(/ { next }

    /Weight::from_parts/ {
      lw = $0
      sub(/.*Weight::from_parts\(\s*/, "", lw)
      sub(/[^0-9_].*$/, "", lw)
      gsub(/_/, "", lw)
      w = lw
    }

    /reads_writes\(/ {
      lw = $0
      sub(/.*reads_writes\(/, "", lw)
      sub(/\).*/, "", lw)
      split(lw, io, ",")
      gsub(/^[ \t]+|[ \t]+$/, "", io[1])
      gsub(/^[ \t]+|[ \t]+$/, "", io[2])
      r = io[1]
      wri = io[2]
      next
    }

    /\.reads\(/ {
      lw = $0
      sub(/.*\.reads\(/, "", lw)
      sub(/\).*/, "", lw)
      r = lw
      next
    }

    /\.writes\(/ {
      lw = $0
      sub(/.*\.writes\(/, "", lw)
      sub(/\).*/, "", lw)
      wri = lw
      next
    }

    # main condition: function name must match "pub fn <extr>("
    $0 ~ ("pub fn[[:space:]]+" extr "\\(") {
      print w, r, wri
      exit
    }
  ' "$DISPATCH")

  # separate into variables
  local code_w code_reads code_writes
  read code_w code_reads code_writes <<<"$code_record"

  # strip underscores or non-digits
  code_w="${code_w//_/}"
  code_w="${code_w%%[^0-9]*}"
  code_reads="${code_reads//_/}"
  code_reads="${code_reads%%[^0-9]*}"
  code_writes="${code_writes//_/}"
  code_writes="${code_writes%%[^0-9]*}"

  # default them if empty
  [[ -z "$code_w" ]] && code_w="0"
  [[ -z "$code_reads" ]] && code_reads="0"
  [[ -z "$code_writes" ]] && code_writes="0"

  # compute drift
  local drift
  drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN {
    if (b == "" || b == 0) {
      print 99999
      exit
    }
    printf("%.1f", (a - b) / b * 100)
  }')

  # produce summary line
  summary_lines+=("$(printf "%-30s | reads code=%3s measured=%3s | writes code=%3s measured=%3s | weight code=%12s measured=%12s | drift %6s%%" \
    "$e" \
    "$code_reads" \
    "$rd" \
    "$code_writes" \
    "$wr" \
    "$code_w" \
    "$meas_ps" \
    "$drift")")

  # validations
  if (( rd != code_reads )); then
    failures+=("[${e}] reads mismatch code=${code_reads}, measured=${rd}")
    fail=1
  fi
  if (( wr != code_writes )); then
    failures+=("[${e}] writes mismatch code=${code_writes}, measured=${wr}")
    fail=1
  fi
  if [[ "$code_w" == "0" ]]; then
    failures+=("[${e}] zero code weight")
    fail=1
  fi

  local abs_drift=${drift#-}
  local drift_int=${abs_drift%%.*}
  if (( drift_int > THRESHOLD )); then
    failures+=("[${e}] weight code=${code_w}, measured=${meas_ps}, drift=${drift}%")
    fail=1
  fi
}

################################################################################

while (( attempt <= MAX_RETRIES )); do
  echo
  echo "Attempt #$attempt"
  echo "──────────────────────────────────────────"

  # run benchmarks and capture output
  TMP="$(mktemp)"
  trap "rm -f \"$TMP\"" EXIT

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

  # reset arrays
  summary_lines=()
  failures=()
  fail=0

  # Current extrinsic data
  extr=""
  meas_us=""
  meas_reads=""
  meas_writes=""

  # We'll finalize an extrinsic each time we see a new "Extrinsic: <x>"
  # or at the end of parsing.
  function finalize_extr() {
    process_extr "$extr" "$meas_us" "$meas_reads" "$meas_writes"
    # reset
    extr=""
    meas_us=""
    meas_reads=""
    meas_writes=""
  }

  # parse the file line-by-line
  while IFS= read -r line; do
    # match new extrinsic name
    if [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]]; then
      # finalize the old extrinsic if any
      finalize_extr
      extr="${BASH_REMATCH[1]}"
      continue
    fi

    # match "Time ~= ..."
    if [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]; then
      meas_us="${BASH_REMATCH[1]}"
      continue
    fi

    # match "Reads = n"
    if [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]; then
      meas_reads="${BASH_REMATCH[1]}"
      continue
    fi

    # match "Writes = n"
    if [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]]; then
      meas_writes="${BASH_REMATCH[1]}"
      continue
    fi
  done < "$TMP"

  # finalize the last extrinsic if we have one
  finalize_extr

  # summary output
  echo
  echo "Benchmark Summary for attempt #$attempt:"
  for l in "${summary_lines[@]}"; do
    echo "  $l"
  done

  if (( fail )); then
    echo
    echo "❌ Issues detected on attempt #$attempt:"
    for e in "${failures[@]}"; do
      echo "  • $e"
    done

    if (( attempt < MAX_RETRIES )); then
      echo "→ Retrying…"
      (( attempt++ ))
      continue
    else
      echo
      echo "❌ Benchmarks failed after $MAX_RETRIES attempts."
      exit 1
    fi
  else
    echo
    echo "✅ All benchmarks within ±${THRESHOLD}% drift."
    exit 0
  fi
done
