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

  # reset counters
  declare -a summary_lines=()
  declare -a failures=()
  fail=0
  extr=""

  # parse and validate
  while IFS= read -r line; do
    if [[ $line =~ Extrinsic:\ \"benchmark_([[:alnum:]_]+)\" ]]; then
      extr="${BASH_REMATCH[1]}"
      continue
    fi

    if [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]; then
      [[ -z "$extr" ]] && continue

      meas_us="${BASH_REMATCH[1]}"
      meas_ps=$(awk -v u="$meas_us" 'BEGIN{printf("%.0f", u * 1000000)}')

      # grab reads & writes
      meas_reads="" meas_writes=""
      while IFS= read -r sub; do
        [[ $sub =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]] && meas_reads="${BASH_REMATCH[1]}" && continue
        [[ $sub =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && meas_writes="${BASH_REMATCH[1]}" && break
      done

      # extract code-side values
      code_record=$(
        awk -v extr="$extr" '
          /^\s*#\[pallet::call_index\(/      { next }
          /Weight::from_parts/               {
                                              lw=$0; sub(/.*Weight::from_parts\(\s*/, "", lw);
                                              sub(/[^0-9_].*$/, "", lw); gsub(/_/, "", lw);
                                              w=lw
                                            }
          /reads_writes\(/                   {
                                              lw=$0; sub(/.*reads_writes\(/, "", lw);
                                              sub(/\).*/, "", lw);
                                              split(lw,io,/,/);
                                              gsub(/^[ \t]+|[ \t]+$/, "", io[1]);
                                              gsub(/^[ \t]+|[ \t]+$/, "", io[2]);
                                              r=io[1]; wri=io[2]; next
                                            }
          /\.reads\(/                        {
                                              lw=$0; sub(/.*\.reads\(/, "", lw);
                                              sub(/\).*/, "", lw);
                                              r=lw; next
                                            }
          /\.writes\(/                       {
                                              lw=$0; sub(/.*\.writes\(/, "", lw);
                                              sub(/\).*/, "", lw);
                                              wri=lw; next
                                            }
          $0 ~ ("pub fn[[:space:]]+" extr "\\(") { print w, r, wri; exit }
        ' "$DISPATCH"
      )
      read code_w code_reads code_writes <<<"$code_record"

      # compute drift %
      drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{printf("%.1f", (a-b)/b*100)}')

      summary_lines+=("$(printf "%-30s | reads code=%3s measured=%3s | writes code=%3s measured=%3s | weight code=%12s measured=%12s | drift %6s%%" \
        "$extr" "$code_reads" "$meas_reads" "$code_writes" "$meas_writes" "$code_w" "$meas_ps" "$drift")")

      # validations
      [[ -z "$code_w" ]]      && failures+=("[${extr}] missing code weight")     && fail=1
      [[ -z "$meas_reads" ]]  && failures+=("[${extr}] missing measured reads")  && fail=1
      [[ -z "$meas_writes" ]] && failures+=("[${extr}] missing measured writes") && fail=1
      (( meas_reads   != code_reads  )) && failures+=("[${extr}] reads mismatch code=${code_reads}, measured=${meas_reads}")   && fail=1
      (( meas_writes  != code_writes )) && failures+=("[${extr}] writes mismatch code=${code_writes}, measured=${meas_writes}") && fail=1
      [[ "$code_w" == "0" ]] && failures+=("[${extr}] zero code weight")        && fail=1

      abs_drift=${drift#-}
      drift_int=${abs_drift%%.*}
      if (( drift_int > THRESHOLD )); then
        failures+=("[${extr}] weight code=${code_w}, measured=${meas_ps}, drift=${drift}%")
        fail=1
      fi

      extr=""
    fi
  done < "$TMP"

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
