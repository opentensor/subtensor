#!/usr/bin/env bash
set -euo pipefail

###############################################################################
# CONFIGURATION
###############################################################################
PALLETS=(subtensor admin_utils commitments drand swap)  # add more if needed

declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
  [swap]="../pallets/swap/src/pallet/mod.rs"
)

THRESHOLD=15        # max % drift allowed before we patch weight
MAX_RETRIES=3       # how many times to rerun a pallet if we patched

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

PATCH_MARKER="$SCRIPT_DIR/benchmark_patch_marker"
PATCH_MODE=0        # becomes 1 once we touch the first file

###############################################################################
# PATCH HELPER – tiny targeted Perl one-liners
###############################################################################
patch_field() {
  local file="$1" extr="$2" field="$3" new_val="$4"

  # remember the first time we patch anything
  if (( PATCH_MODE == 0 )); then : >"$PATCH_MARKER"; PATCH_MODE=1; fi
  echo "$file" >>"$PATCH_MARKER"

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

###############################################################################
# MAIN BUILD
###############################################################################
echo "Building runtime-benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo
echo "──────────────────────────────────────────"
echo " Will benchmark pallets: ${PALLETS[*]}"
echo "──────────────────────────────────────────"

###############################################################################
# EXTRINSIC PROCESSOR  (AWK parser  +   auto-patch)
###############################################################################
process_extr() {
  local e="$1" us="$2" rd="$3" wr="$4" dispatch_file="$5"

  [[ -z "$e" || -z "$us" || -z "$rd" || -z "$wr" ]] && return

  local meas_ps
  meas_ps=$(awk -v x="$us" 'BEGIN{printf("%.0f", x * 1000000)}')  # µs → ps

  # ---- parse dispatch file exactly like the old script ----------------------
  local code_record
  code_record=$(awk -v extr="$e" '
    /^\s*#\[pallet::call_index\(/ { next }   # skip helper attrs

    /Weight::from_parts/ {
      tmp = $0
      sub(/.*Weight::from_parts\(\s*/, "", tmp)
      sub(/[^0-9_].*$/, "", tmp)
      gsub(/_/, "", tmp)
      w = tmp
    }

    /reads_writes\(/ {
      tmp = $0
      sub(/.*reads_writes\(/, "", tmp)
      sub(/\).*/, "", tmp)
      split(tmp, io, ",")
      gsub(/^[ \t]+|[ \t]+$/, "", io[1])
      gsub(/^[ \t]+|[ \t]+$/, "", io[2])
      r  = io[1]
      wr = io[2]
      next
    }

    /\.reads\(/ {
      tmp = $0
      sub(/.*\.reads\(/, "", tmp)
      sub(/\).*/, "", tmp)
      r = tmp
      next
    }

    /\.writes\(/ {
      tmp = $0
      sub(/.*\.writes\(/, "", tmp)
      sub(/\).*/, "", tmp)
      wr = tmp
      next
    }

    $0 ~ ("pub fn[[:space:]]+" extr "\\(") { print w, r, wr; exit }
  ' "$dispatch_file")

  local code_w code_r code_wr
  read code_w code_r code_wr <<<"$code_record"

  code_w="${code_w//_/}";   code_w="${code_w%%[^0-9]*}"
  code_r="${code_r//_/}";   code_r="${code_r%%[^0-9]*}"
  code_wr="${code_wr//_/}"; code_wr="${code_wr%%[^0-9]*}"

  [[ -z "$code_w" ]]  && code_w=0
  [[ -z "$code_r" ]]  && code_r=0
  [[ -z "$code_wr" ]] && code_wr=0

  local drift=0
  if (( code_w != 0 )); then
    drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{printf("%.1f",(a-b)/b*100)}')
  else
    drift=99999
  fi
  local abs=${drift#-}; local drift_int=${abs%%.*}

  summary_lines+=("$(printf "%-30s | reads %4s→%4s | writes %4s→%4s | weight %13s→%13s | drift %6s%%" \
    "$e" "$code_r" "$rd" "$code_wr" "$wr" "$code_w" "$meas_ps" "$drift")")

  local fail_this=0
  # ---- patch when mismatching ----------------------------------------------
  if (( rd != code_r )); then
    patch_field "$dispatch_file" "$e" reads "$rd";   fail_this=1
  fi
  if (( wr != code_wr )); then
    patch_field "$dispatch_file" "$e" writes "$wr";  fail_this=1
  fi
  if (( drift_int > THRESHOLD )); then
    patch_field "$dispatch_file" "$e" weight "$meas_ps"; fail_this=1
  fi
  (( fail_this )) && failures+=("$e")
}

###############################################################################
# BENCHMARK LOOP  (retries if anything was patched)
###############################################################################
for pallet in "${PALLETS[@]}"; do
  dispatch="${DISPATCH_PATHS[$pallet]}"
  [[ -n "$dispatch" && -f "$SCRIPT_DIR/$dispatch" ]] \
    || { echo "❌ dispatch file missing for pallet '$pallet'"; exit 1; }
  dispatch="$SCRIPT_DIR/$dispatch"

  for (( attempt=1; attempt<=MAX_RETRIES; attempt++ )); do
    echo
    echo "══════════════════════════════════════"
    echo "Benchmarking pallet: $pallet (attempt #$attempt)"
    echo "Dispatch file: $dispatch"
    echo "══════════════════════════════════════"

    tmp=$(mktemp); trap 'rm -f "$tmp"' EXIT

    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" \
      --genesis-builder=runtime --genesis-builder-preset=benchmark \
      --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" \
      --steps 50 --repeat 5 | tee "$tmp"

    summary_lines=(); failures=()
    extr=""; meas_us=""; meas_r=""; meas_w=""

    finalize() {
      [[ -n $extr ]] && process_extr "$extr" "$meas_us" "$meas_r" "$meas_w" "$dispatch"
      extr=""; meas_us=""; meas_r=""; meas_w=""
    }

    while IFS= read -r line; do
      if [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]]; then
        finalize; extr="${BASH_REMATCH[1]}"; continue
      fi
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]  && meas_us="${BASH_REMATCH[1]}"
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && meas_r="${BASH_REMATCH[1]}"
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && meas_w="${BASH_REMATCH[1]}"
    done <"$tmp"
    finalize

    echo
    echo "Benchmark Summary (attempt #$attempt):"
    printf '  %s\n' "${summary_lines[@]}"

    if (( ${#failures[@]} )); then
      echo -e "\n❌ Issues detected:"
      printf '  • %s\n' "${failures[@]}"
      if (( attempt < MAX_RETRIES )); then
        echo "→ Patches applied, retrying…"
        continue
      else
        echo "❌ Still failing after $MAX_RETRIES attempts."
        exit 1
      fi
    fi
    break  # pallet succeeded
  done
done

echo
echo "══════════════════════════════════════"
echo "All requested pallets benchmarked successfully!"
echo "══════════════════════════════════════"
(( PATCH_MODE )) && echo "💾  Drift fixed in place; touched files listed in $PATCH_MARKER"
exit 0
