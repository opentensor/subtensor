#!/usr/bin/env bash
set -euo pipefail

# ────────────────────────────────────────────────────────────────────────────────
#  CONFIGURATION
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
#  PATCH HELPER
# ────────────────────────────────────────────────────────────────────────────────
patch_field() {
  local file="$1" extr="$2" field="$3" new_val="$4"

  if (( PATCH_MODE == 0 )); then : > "$PATCH_MARKER"; PATCH_MODE=1; fi
  echo "$file" >> "$PATCH_MARKER"

  case "$field" in
    weight)
      perl -0777 -pi -e '
        s{
          (#[[:space:]]*\[pallet::weight\([^]]*?Weight::from_parts\(\s*)
          [0-9_]+
          (?=[^]]*?\]\s*pub\s+fn\s+'"${extr}"'\b)
        }{\1'"${new_val}"'}sx' "$file"
      ;;
    reads)
      perl -0777 -pi -e '
        s{
          (#[[:space:]]*\[pallet::weight\([^]]*?
            (?:\.reads\(\s*|reads_writes\(\s*)\s*)
          )[0-9_]+
          (?=[^]]*?\]\s*pub\s+fn\s+'"${extr}"'\b)
        }{\1'"${new_val}"'}sx' "$file"
      ;;
    writes)
      perl -0777 -pi -e '
        s{
          (#[[:space:]]*\[pallet::weight\([^]]*?
            (?:\.writes\(\s*|reads_writes\(\s*[0-9_]+\s*,\s*)\s*)
          )[0-9_]+
          (?=[^]]*?\]\s*pub\s+fn\s+'"${extr}"'\b)
        }{\1'"${new_val}"'}sx' "$file"
      ;;
  esac
}

# ────────────────────────────────────────────────────────────────────────────────
#  LITERAL EXTRACTION (weight / reads / writes)
# ────────────────────────────────────────────────────────────────────────────────
extract_code_numbers() {
  local file="$1" extr="$2"

  local block
  block=$(perl -0777 -ne '
      if(/#[[:space:]]*\[pallet::weight\(([^]]*?)\]\s*pub\s+fn\s+'"${extr}"'\b/s){
        print $1; exit
      }' "$file" || true)

  local w=0 r=0 wr=0
  [[ $block =~ Weight::from_parts\(\s*([0-9_]+) ]]        && w="${BASH_REMATCH[1]//_/}"
  if [[ $block =~ reads_writes\(\s*([0-9_]+)\s*,\s*([0-9_]+) ]]; then
    r="${BASH_REMATCH[1]//_/}"
    wr="${BASH_REMATCH[2]//_/}"
  else
    [[ $block =~ \.reads\(\s*([0-9_]+) ]]  && r="${BASH_REMATCH[1]//_/}"
    [[ $block =~ \.writes\(\s*([0-9_]+) ]] && wr="${BASH_REMATCH[1]//_/}"
  fi
  echo "$w $r $wr"
}

# ────────────────────────────────────────────────────────────────────────────────
echo "Building runtime‑benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo -e "\n──────────────────────────────────────────"
echo   " Will benchmark pallets: ${PALLETS[*]}"
echo   "──────────────────────────────────────────"

# ────────────────────────────────────────────────────────────────────────────────
#  COMPARISON + OPTIONAL PATCHING
# ────────────────────────────────────────────────────────────────────────────────
process_extr() {
  local e="$1" us="$2" rd="$3" wr="$4" dispatch_file="$5"
  [[ -z "$e" ]] && return

  local meas_ps; meas_ps=$(awk -v x="$us" 'BEGIN{printf("%.0f", x*1000000)}')
  read code_w code_r code_wr < <(extract_code_numbers "$dispatch_file" "$e")

  local drift=0
  [[ $code_w != 0 ]] && drift=$(awk -v a="$meas_ps" -v b="$code_w" \
                                 'BEGIN{printf("%.1f",(a-b)/b*100)}')

  summary_lines+=( "$(printf '%-30s | reads %5s→%5s | writes %5s→%5s | weight %13s→%13s | drift %6s%%' \
                           "$e" "${code_r:-0}" "$rd" "${code_wr:-0}" "$wr" "${code_w:-0}" "$meas_ps" "$drift")" )

  # ── integer drift for arithmetic test ───────────────────────────────────────
  local abs="${drift#-}"
  local drift_int="${abs%%.*}"
  [[ -z "$drift_int" ]] && drift_int=0

  local fail_now=0
  if (( rd != code_r ));     then patch_field "$dispatch_file" "$e" reads  "$rd"; fail_now=1; fi
  if (( wr != code_wr ));    then patch_field "$dispatch_file" "$e" writes "$wr"; fail_now=1; fi
  if (( drift_int > THRESHOLD )); then
    patch_field "$dispatch_file" "$e" weight "$meas_ps"; fail_now=1
  fi
  (( fail_now )) && failures+=( "$e" )
}

# ────────────────────────────────────────────────────────────────────────────────
#  MAIN LOOP
# ────────────────────────────────────────────────────────────────────────────────
for pallet in "${PALLETS[@]}"; do
  DISPATCH_FILE="$SCRIPT_DIR/${DISPATCH_PATHS[$pallet]}"
  [[ -f "$DISPATCH_FILE" ]] || { echo "❌ Dispatch file not found: $DISPATCH_FILE"; exit 1; }

  for (( attempt=1; attempt<=MAX_RETRIES; attempt++ )); do
    echo -e "\n══════════════════════════════════════"
    echo   "Benchmarking pallet: $pallet (attempt #$attempt)"
    echo   "Dispatch file: $DISPATCH_FILE"
    echo   "══════════════════════════════════════"

    TMP=$(mktemp); trap 'rm -f "$TMP"' EXIT
    ./target/production/node-subtensor benchmark pallet \
        --runtime "$RUNTIME_WASM" \
        --genesis-builder=runtime --genesis-builder-preset=benchmark \
        --wasm-execution=compiled \
        --pallet "pallet_${pallet}" --extrinsic "*" \
        --steps 50 --repeat 5 | tee "$TMP"

    summary_lines=(); failures=()
    extr="" meas_us="" meas_r=0 meas_w=0
    finalise() { [[ -n $extr ]] && process_extr "$extr" "$meas_us" "$meas_r" "$meas_w" "$DISPATCH_FILE"; meas_r=0; meas_w=0; extr=""; }

    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([A-Za-z0-9_]+)\" ]] && { finalise; extr="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]  && meas_us="${BASH_REMATCH[1]}"
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && meas_r="${BASH_REMATCH[1]}"
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && meas_w="${BASH_REMATCH[1]}"
    done < "$TMP"
    finalise

    echo -e "\nBenchmark Summary (attempt #$attempt):"
    printf '  %s\n' "${summary_lines[@]}"

    if (( ${#failures[@]} )); then
      echo -e "\n❌ Issues detected:"
      printf '  • %s\n' "${failures[@]}"
      (( attempt < MAX_RETRIES )) && { echo "→ Retrying…"; continue; }
    fi
    break
  done
done

echo -e "\n══════════════════════════════════════"
echo   "All requested pallets benchmarked successfully!"
echo   "══════════════════════════════════════"
(( PATCH_MODE )) && echo "💾  Benchmark drift fixed in‑place; files recorded in $PATCH_MARKER"
exit 0
