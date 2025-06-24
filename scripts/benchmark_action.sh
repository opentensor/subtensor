#!/usr/bin/env bash
set -euo pipefail

# ────────────────────────────────────────────────────────────────────────────────
# Configuration
# ────────────────────────────────────────────────────────────────────────────────
PALLETS=(subtensor admin_utils commitments drand)

declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
  [swap]="../pallets/swap/src/pallet/mod.rs"
)

THRESHOLD=15
MAX_RETRIES=3

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"
PATCH_MARKER="$SCRIPT_DIR/benchmark_patch_marker"
PATCH_MODE=0

# ────────────────────────────────────────────────────────────────────────────────
# Helper: patch a single literal inside an attribute block *for the given extrinsic*
# ────────────────────────────────────────────────────────────────────────────────
patch_field() {
  local file="$1" extr="$2" field="$3" new_val="$4"

  # create marker on first change
  if (( PATCH_MODE == 0 )); then : > "$PATCH_MARKER"; PATCH_MODE=1; fi
  echo "$file" >> "$PATCH_MARKER"

  case "$field" in
    weight)
      perl -0777 -pi -e 's/(#\s*\[pallet::weight\([^]]*?Weight::from_parts\()\s*[0-9_]+(?=[^]]*?\]\s*pub\s+fn\s+'"${extr}"'\b)/\1'"${new_val}"'/s' "$file"
      ;;
    reads)
      perl -0777 -pi -e 's/(#\s*\[pallet::weight\([^]]*?\.reads\()\s*[0-9_]+(?=[^]]*?\]\s*pub\s+fn\s+'"${extr}"'\b)/\1'"${new_val}"'/s' "$file"
      perl -0777 -pi -e 's/(#\s*\[pallet::weight\([^]]*?reads_writes\()\s*[0-9_]+(?=,[^]]*?\]\s*pub\s+fn\s+'"${extr}"'\b)/\1'"${new_val}"'/s' "$file"
      ;;
    writes)
      perl -0777 -pi -e 's/(#\s*\[pallet::weight\([^]]*?\.writes\()\s*[0-9_]+(?=[^]]*?\]\s*pub\s+fn\s+'"${extr}"'\b)/\1'"${new_val}"'/s' "$file"
      perl -0777 -pi -e 's/(#\s*\[pallet::weight\([^]]*?reads_writes\([0-9_]+,\s*)[0-9_]+(?=[^]]*?\]\s*pub\s+fn\s+'"${extr}"'\b)/\1'"${new_val}"'/s' "$file"
      ;;
  esac
}

# ────────────────────────────────────────────────────────────────────────────────
echo "Building runtime‑benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo -e "\n──────────────────────────────────────────"
echo   " Will benchmark pallets: ${PALLETS[*]}"
echo   "──────────────────────────────────────────"

# ────────────────────────────────────────────────────────────────────────────────
# Helper that compares measured vs. code‑side numbers and patches when needed
# ────────────────────────────────────────────────────────────────────────────────
process_extr() {
  local e="$1" us="$2" rd="$3" wr="$4" dispatch_file="$5"
  [[ -z "$e" || -z "$us" || -z "$rd" || -z "$wr" ]] && return

  # Convert micro‑seconds → pico‑seconds (Substrate weights use ps)
  local meas_ps; meas_ps=$(awk -v x="$us" 'BEGIN{printf("%.0f", x * 1000000)}')

  # ── Look up the literal numbers already in the code ─────────────────────────
  local code_record
  code_record=$(awk -v extr="$e" '
    /^\s*#\[pallet::call_index\(/ { next }
    /Weight::from_parts/          { lw=$0; sub(/.*Weight::from_parts\(\s*/, "", lw); sub(/[^0-9_].*$/, "", lw); gsub(/_/, "", lw); w=lw }
    /reads_writes\(/              { lw=$0; sub(/.*reads_writes\(/, "", lw); sub(/\).*/, "", lw); split(lw, io, ","); gsub(/[ \t]/, "", io[1]); gsub(/[ \t]/, "", io[2]); r=io[1]; wri=io[2]; next }
    /\.reads\(/                   { lw=$0; sub(/.*\.reads\(/, "", lw); sub(/\).*/, "", lw); r=lw; next }
    /\.writes\(/                  { lw=$0; sub(/.*\.writes\(/, "", lw); sub(/\).*/, "", lw); wri=lw; next }
    $0 ~ ("pub fn[[:space:]]+" extr "\\(") { print w, r, wri; exit }
  ' "$dispatch_file")

  local code_w code_reads code_writes
  read code_w code_reads code_writes <<<"$code_record"
  code_w="${code_w//_/}";       code_w="${code_w%%[^0-9]*}"
  code_reads="${code_reads//_/}"; code_reads="${code_reads%%[^0-9]*}"
  code_writes="${code_writes//_/}"; code_writes="${code_writes%%[^0-9]*}"
  [[ -z "$code_w" ]]      && code_w="0"
  [[ -z "$code_reads" ]]  && code_reads="0"
  [[ -z "$code_writes" ]] && code_writes="0"

  local drift
  drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{ if(b==0){print 99999;exit}; printf("%.1f",(a-b)/b*100)}')

  summary_lines+=("$(printf "%-30s | reads code=%4s measured=%4s | writes code=%4s measured=%4s | weight code=%12s measured=%12s | drift %6s%%" \
                  "$e" "$code_reads" "$rd" "$code_writes" "$wr" "$code_w" "$meas_ps" "$drift")")

  # ── validations & auto‑patching ─────────────────────────────────────────────
  if (( rd != code_reads )); then
    failures+=("[${e}] reads mismatch code=${code_reads}, measured=${rd}")
    patch_field "$dispatch_file" "$e" "reads" "${rd//[^0-9]/}"
    fail=1
  fi
  if (( wr != code_writes )); then
    failures+=("[${e}] writes mismatch code=${code_writes}, measured=${wr}")
    patch_field "$dispatch_file" "$e" "writes" "${wr//[^0-9]/}"
    fail=1
  fi

  local abs=${drift#-}; local drift_int=${abs%%.*}
  if (( drift_int > THRESHOLD )); then
    failures+=("[${e}] weight code=${code_w}, measured=${meas_ps}, drift=${drift}%")
    local pretty_weight; pretty_weight=$(printf "%'d" "$meas_ps" | tr ',' '_')
    patch_field "$dispatch_file" "$e" "weight" "$pretty_weight"
    fail=1
  fi
}

# ────────────────────────────────────────────────────────────────────────────────
# Main benchmarking loop
# ────────────────────────────────────────────────────────────────────────────────
for pallet in "${PALLETS[@]}"; do
  DISPATCH_FILE="$SCRIPT_DIR/${DISPATCH_PATHS[$pallet]}"
  [[ -f "$DISPATCH_FILE" ]] || { echo "❌ dispatch file not found: $DISPATCH_FILE"; exit 1; }

  attempt=1; pallet_success=0
  while (( attempt <= MAX_RETRIES )); do
    echo -e "\n══════════════════════════════════════"
    echo   "Benchmarking pallet: $pallet (attempt #$attempt)"
    echo   "Dispatch file: $DISPATCH_FILE"
    echo   "══════════════════════════════════════"

    TMP=$(mktemp); trap "rm -f $TMP" EXIT
    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" \
      --genesis-builder=runtime --genesis-builder-preset=benchmark \
      --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" \
      --steps 50 --repeat 5 | tee "$TMP"

    summary_lines=(); failures=(); fail=0
    extr="" meas_us="" meas_reads="" meas_writes=""
    finalise() { process_extr "$extr" "$meas_us" "$meas_reads" "$meas_writes" "$DISPATCH_FILE"; extr=""; meas_us=""; meas_reads=""; meas_writes=""; }

    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([A-Za-z0-9_]+)\" ]] && { finalise; extr="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]  && { meas_us="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && { meas_reads="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && { meas_writes="${BASH_REMATCH[1]}"; continue; }
    done < "$TMP"
    finalise

    echo -e "\nBenchmark Summary for pallet '$pallet' (attempt #$attempt):"
    printf "  %s\n" "${summary_lines[@]}"

    if (( fail )); then
      printf '\n❌ Issues on attempt #%d:\n' "$attempt"
      printf '  • %s\n' "${failures[@]}"

      if (( attempt < MAX_RETRIES )); then
        echo "→ Retrying…"
        (( attempt++ )); continue
      fi

      if (( PATCH_MODE )); then
        echo -e "\n🛠️  Patched dispatch file(s). Continuing."
        pallet_success=1; break
      else
        echo -e "\n❌ Failed after $MAX_RETRIES attempts."; exit 1
      fi
    else
      echo -e "\n✅ Pallet '$pallet' benchmarks within ±${THRESHOLD}%% drift."
      pallet_success=1; break
    fi
  done

  (( pallet_success )) || { echo "❌ Could not benchmark pallet '$pallet'."; exit 1; }
done

echo -e "\n══════════════════════════════════════"
echo   "All requested pallets benchmarked successfully!"
echo   "══════════════════════════════════════"
(( PATCH_MODE )) && echo "💾  Benchmark drift fixed in-place; files recorded in $PATCH_MARKER"
exit 0
