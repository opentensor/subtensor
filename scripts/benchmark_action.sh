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
# Helper: patch a literal number inside the attribute block of <extrinsic>
# ────────────────────────────────────────────────────────────────────────────────
patch_field() {
  local file="$1" extr="$2" field="$3" new_val="$4"

  if (( PATCH_MODE == 0 )); then : > "$PATCH_MARKER"; PATCH_MODE=1; fi
  echo "$file" >> "$PATCH_MARKER"

  case "$field" in
    weight)
      awk -i inplace -v fn="$extr" -v nv="$new_val" '
        NR>25 { window[NR%26]=$0 }          # ring buffer of last 25 lines
        { line[NR]=$0 }                     # store all lines for later output
        $0 ~ ("pub[[:space:]]+fn[[:space:]]+"fn"\\(") {
          for(i=NR-1;i>=NR-25&&i>0;i--){
            if(match(line[i],/Weight::from_parts\([0-9_]+/,m)){
              sub(/[0-9_]+/,nv,line[i]); line_modified[i]=1; break;
            }
          }
        }
        END{
          for(i=1;i<=NR;i++){
            if(line_modified[i]) print line[i]; else print line[i];
          }
        }' "$file"
      ;;
    reads)
      awk -i inplace -v fn="$extr" -v nv="$new_val" '
        { line[NR]=$0 }
        $0 ~ ("pub[[:space:]]+fn[[:space:]]+"fn"\\(") {
          for(i=NR-1;i>=NR-25&&i>0;i--){
            if(match(line[i],/reads_writes\([0-9_]+,[[:space:]]*[0-9_]+/,m)){
              sub(/[0-9_]+/,nv,line[i]); line_modified[i]=1; break
            }
            if(match(line[i],/\.reads\([0-9_]+/,m)){
              sub(/[0-9_]+/,nv,line[i]); line_modified[i]=1; break
            }
          }
        }
        END{ for(i=1;i<=NR;i++) print line[i] }' "$file"
      ;;
    writes)
      awk -i inplace -v fn="$extr" -v nv="$new_val" '
        { line[NR]=$0 }
        $0 ~ ("pub[[:space:]]+fn[[:space:]]+"fn"\\(") {
          for(i=NR-1;i>=NR-25&&i>0;i--){
            if(match(line[i],/reads_writes\([0-9_]+,[[:space:]]*[0-9_]+/,m)){
              sub(/,[[:space:]]*[0-9_]+/,", "nv,line[i]); line_modified[i]=1; break
            }
            if(match(line[i],/\.writes\([0-9_]+/,m)){
              sub(/[0-9_]+/,nv,line[i]); line_modified[i]=1; break
            }
          }
        }
        END{ for(i=1;i<=NR;i++) print line[i] }' "$file"
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
# Helper that extracts code‑side numbers for one extrinsic
# ────────────────────────────────────────────────────────────────────────────────
lookup_code_numbers() {
  local extr="$1" file="$2"
  awk -v fn="$extr" '
    { buf[NR]=$0 }
    $0 ~ ("pub[[:space:]]+fn[[:space:]]+"fn"\\(") {
      w=r=wri="0"
      for(i=NR-1;i>=NR-25&&i>0;i--){
        if(match(buf[i],/Weight::from_parts\(([0-9_]+)/,m)){ w=m[1] }
        if(match(buf[i],/reads_writes\(([0-9_]+),[[:space:]]*([0-9_]+)/,m)){ r=m[1]; wri=m[2] }
        if(match(buf[i],/\.reads\(([0-9_]+)/,m)){ r=m[1] }
        if(match(buf[i],/\.writes\(([0-9_]+)/,m)){ wri=m[1] }
      }
      gsub(/_/,"",w); gsub(/_/,"",r); gsub(/_/,"",wri)
      print w, r, wri; exit
    }' "$file"
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
    finalise() {
      [[ -z "$extr" ]] && return
      read code_w code_reads code_writes <<<"$(lookup_code_numbers "$extr" "$DISPATCH_FILE")"

      # Convert µs → ps
      local meas_ps; meas_ps=$(awk -v x="$meas_us" 'BEGIN{printf("%.0f", x * 1000000)}')

      local drift
      drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{ if(b==0){print 99999;exit}; printf("%.1f",(a-b)/b*100)}')

      summary_lines+=("$(printf "%-30s | reads code=%4s measured=%4s | writes code=%4s measured=%4s | weight code=%12s measured=%12s | drift %6s%%" \
                       "$extr" "$code_reads" "$meas_reads" "$code_writes" "$meas_writes" "$code_w" "$meas_ps" "$drift")")

      local abs=${drift#-}; local drift_int=${abs%%.*}
      if (( meas_reads != code_reads )); then
        failures+=("[${extr}] reads mismatch code=${code_reads}, measured=${meas_reads}")
        patch_field "$DISPATCH_FILE" "$extr" "reads" "$meas_reads"
        fail=1
      fi
      if (( meas_writes != code_writes )); then
        failures+=("[${extr}] writes mismatch code=${code_writes}, measured=${meas_writes}")
        patch_field "$DISPATCH_FILE" "$extr" "writes" "$meas_writes"
        fail=1
      fi
      if (( drift_int > THRESHOLD )); then
        failures+=("[${extr}] weight code=${code_w}, measured=${meas_ps}, drift=${drift}%")
        local pretty_weight; pretty_weight=$(printf "%'d" "$meas_ps" | tr ',' '_')
        patch_field "$DISPATCH_FILE" "$extr" "weight" "$pretty_weight"
        fail=1
      fi
      extr="" meas_us="" meas_reads="" meas_writes=""
    }

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
