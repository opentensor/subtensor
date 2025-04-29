#!/usr/bin/env bash
set -euo pipefail

# ---------------------------
# Configurable parameters
# ---------------------------
THRESHOLD=10    # Max allowed drift in %
MAX_RETRIES=3   # Number of retry attempts

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="./target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

echo "[DEBUG] Building runtime-benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

# Pallets to benchmark
PALLETS=(subtensor admin_utils commitments drand crowdloan)

# Dispatch-file paths
declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
  [crowdloan]="../pallets/crowdloan/src/lib.rs"
)

# -------------------------------------
# Main loop over each pallet
# -------------------------------------
for pallet in "${PALLETS[@]}"; do
  echo
  echo "──────────────────────────────────────────"
  echo " Benchmarking pallet: $pallet"
  echo "──────────────────────────────────────────"

  DISPATCH="$SCRIPT_DIR/${DISPATCH_PATHS[$pallet]}"
  if [[ ! -f "$DISPATCH" ]]; then
    echo "❌ ERROR: dispatch file not found at $DISPATCH"
    exit 1
  fi

  # Convert e.g. "admin_utils" → "pallet_admin_utils"
  PALLET_NAME="pallet_${pallet}"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    echo
    echo "Attempt #$attempt for $PALLET_NAME"
    echo "──────────────────────────────────────────"

    TMP="$(mktemp)"
    trap 'rm -f "$TMP"' EXIT

    echo "[DEBUG] Running benchmarks for $PALLET_NAME…"
    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" \
      --genesis-builder=runtime \
      --genesis-builder-preset=benchmark \
      --wasm-execution=compiled \
      --pallet "$PALLET_NAME" \
      --extrinsic "*" \
      --steps 50 \
      --repeat 5 \
    | tee "$TMP"

    # ---------------------
    # PHASE 1: Collect logs
    # ---------------------
    extrinsics_list=()
    measure_list=()

    echo "[DEBUG] Parsing output line by line…"

    while IFS= read -r line; do
      echo "[DEBUG] Line => $line"

      # (A) Match lines in the form:
      #     Pallet: "pallet_foo", Extrinsic: "bar", ...
      # Example:
      #     Pallet: "pallet_subtensor", Extrinsic: "benchmark_register", Lowest values: []
      #
      # So we capture the extrinsic name out of the quotes after 'Extrinsic:'
      if [[ $line =~ ^Pallet:\ \"${PALLET_NAME}\",[[:space:]]Extrinsic:\ \"([[:alnum:]_]+)\" ]]; then
        ex="${BASH_REMATCH[1]}"
        echo "[DEBUG]   --> Matched extrinsic name: $ex"
        extrinsics_list+=("$ex")

      # (B) Match lines in the form:
      #     Time ~=    123.45
      # Possibly with multiple spaces, so we use a more relaxed pattern:
      elif [[ $line =~ ^Time[[:space:]]*~=[[:space:]]*([0-9]+(\.[0-9]+)?) ]]; then
        meas_us="${BASH_REMATCH[1]}"
        echo "[DEBUG]   --> Matched time microseconds: $meas_us"

        # Convert microseconds → picoseconds
        meas_ps=$(awk -v u="$meas_us" 'BEGIN{printf("%.0f", u * 1000000)}')
        echo "[DEBUG]       => Converted to picoseconds: $meas_ps"

        # Next lines: "Reads = X" and "Writes = Y"
        meas_reads=""
        meas_writes=""
        while IFS= read -r sub; do
          echo "[DEBUG]       sub => $sub"
          if [[ $sub =~ Reads[[:space:]]*=[[:space:]]*([0-9]+).* ]]; then
            meas_reads="${BASH_REMATCH[1]}"
            echo "[DEBUG]         --> Matched reads: $meas_reads"
          elif [[ $sub =~ Writes[[:space:]]*=[[:space:]]*([0-9]+).* ]]; then
            meas_writes="${BASH_REMATCH[1]}"
            echo "[DEBUG]         --> Matched writes: $meas_writes"
          fi
          if [[ -n "$meas_reads" && -n "$meas_writes" ]]; then
            break
          fi
        done

        measure_list+=("${meas_ps},${meas_reads},${meas_writes}")
        echo "[DEBUG]   --> Pushed measurement: ${meas_ps},${meas_reads},${meas_writes}"
      fi
    done < "$TMP"

    echo "[DEBUG] Finished reading logs."
    echo "[DEBUG] extrinsics_list => ${extrinsics_list[@]}"
    echo "[DEBUG] measure_list    => ${measure_list[@]}"

    # -------------------------------
    # PHASE 2: Pair up extrinsics & measurements
    # -------------------------------
    summary_lines=()
    failures=()
    fail=0

    len_extr=${#extrinsics_list[@]}
    len_meas=${#measure_list[@]}
    pair_count=$(( len_extr < len_meas ? len_extr : len_meas ))

    echo "[DEBUG] extrinsics count: $len_extr"
    echo "[DEBUG] measurements count: $len_meas"
    echo "[DEBUG] pairing up to: $pair_count"

    for (( i=0; i< pair_count; i++ )); do
      extr="${extrinsics_list[$i]}"
      measurement="${measure_list[$i]}"
      echo "[DEBUG] Pairing extrinsic #$i '$extr' with measurement '$measurement'"

      IFS=',' read -r meas_ps meas_reads meas_writes <<< "$measurement"

      # Look up code-side values from the dispatch file
      code_record=$(
        awk -v extr="$extr" '
          /^\s*#\[pallet::call_index\(/     { next }
          /Weight::from_parts/              {
                                              lw=$0; sub(/.*Weight::from_parts\(\s*/, "", lw);
                                              sub(/[^0-9_].*$/, "", lw);
                                              gsub(/_/, "", lw);
                                              w=lw
                                            }
          /reads_writes\(/                  {
                                              lw=$0; sub(/.*reads_writes\(/, "", lw);
                                              sub(/\).*/, "", lw);
                                              split(lw,io,/,/);
                                              gsub(/^[ \t]+|[ \t]+$/, "", io[1]);
                                              gsub(/^[ \t]+|[ \t]+$/, "", io[2]);
                                              r=io[1]; wri=io[2]; next
                                            }
          /\.reads\(/                       {
                                              lw=$0; sub(/.*\.reads\(/, "", lw);
                                              sub(/\).*/, "", lw);
                                              r=lw; next
                                            }
          /\.writes\(/                      {
                                              lw=$0; sub(/.*\.writes\(/, "", lw);
                                              sub(/\).*/, "", lw);
                                              wri=lw; next
                                            }
          $0 ~ ("pub fn[[:space:]]+" extr "\\(") {
             print w, r, wri
             exit
          }
        ' "$DISPATCH"
      )

      read code_w code_reads code_writes <<< "$code_record"
      code_w=${code_w//_/};        code_w=${code_w%%[^0-9]*}
      code_reads=${code_reads//_/}; code_reads=${code_reads%%[^0-9]*}
      code_writes=${code_writes//_/}; code_writes=${code_writes%%[^0-9]*}

      drift="0"
      if [[ -n "$code_w" && "$code_w" != "0" ]]; then
        drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{printf("%.1f", (a-b)/b*100)}')
      fi

      summary_line="$(printf "%-30s | reads code=%3s measured=%3s | writes code=%3s measured=%3s | weight code=%12s measured=%12s | drift %6s%%" \
        "$extr" \
        "${code_reads:-0}" "${meas_reads:-0}" \
        "${code_writes:-0}" "${meas_writes:-0}" \
        "${code_w:-0}" "$meas_ps" "$drift" )"
      summary_lines+=( "$summary_line" )

      echo "[DEBUG] Built summary line => $summary_line"

      # Validations
      if [[ -z "$code_w" ]]; then
        failures+=("[${extr}] missing code weight")
        fail=1
      fi
      if [[ -z "$meas_reads" ]]; then
        failures+=("[${extr}] missing measured reads")
        fail=1
      fi
      if [[ -z "$meas_writes" ]]; then
        failures+=("[${extr}] missing measured writes")
        fail=1
      fi
      if [[ -n "$code_reads" && -n "$meas_reads" ]]; then
        (( meas_reads != code_reads )) && {
          failures+=("[${extr}] reads mismatch code=${code_reads}, measured=${meas_reads}")
          fail=1
        }
      fi
      if [[ -n "$code_writes" && -n "$meas_writes" ]]; then
        (( meas_writes != code_writes )) && {
          failures+=("[${extr}] writes mismatch code=${code_writes}, measured=${meas_writes}")
          fail=1
        }
      fi
      if [[ "$code_w" == "0" ]]; then
        failures+=("[${extr}] zero code weight")
        fail=1
      fi
      abs_drift=${drift#-}
      drift_int=${abs_drift%%.*}
      if (( drift_int > THRESHOLD )); then
        failures+=("[${extr}] weight code=${code_w}, measured=${meas_ps}, drift=${drift}%")
        fail=1
      fi
    done

    # If there are more extrinsics than measurements
    if (( len_extr > pair_count )); then
      echo "⚠️  Found ${len_extr} extrinsics but only ${len_meas} measurements."
      extra=$(( len_extr - pair_count ))
      echo "   → ${extra} extrinsics had no timing data!"
      fail=1
    fi

    # If there are more measurements than extrinsics
    if (( len_meas > pair_count )); then
      echo "⚠️  Found ${len_meas} measurements but only ${len_extr} extrinsics."
      extra=$(( len_meas - pair_count ))
      echo "   → ${extra} measurements were unused (no matching extrinsic)!"
      fail=1
    fi

    # ---------------
    # Print summary
    # ---------------
    echo
    echo "Benchmark Summary for $PALLET_NAME attempt #$attempt:"
    if [[ ${#summary_lines[@]} -eq 0 ]]; then
      echo "  (No extrinsics matched or no measurement data was paired.)"
    else
      for line in "${summary_lines[@]}"; do
        echo "  $line"
      done
    fi

    # Check for failures
    if (( fail )); then
      echo
      echo "❌ Issues detected for $PALLET_NAME on attempt #$attempt:"
      for e in "${failures[@]}"; do
        echo "  • $e"
      done

      if (( attempt < MAX_RETRIES )); then
        echo "→ Retrying…"
        (( attempt++ ))
        continue
      else
        echo
        echo "❌ Benchmarks failed for $PALLET_NAME after $MAX_RETRIES attempts."
        exit 1
      fi
    else
      echo
      echo "✅ All benchmarks within ±${THRESHOLD}% drift for $PALLET_NAME."
      break
    fi

  done
done

echo
echo "✅ All pallets benchmarked successfully."
