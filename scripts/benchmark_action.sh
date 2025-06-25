#!/usr/bin/env bash
# ------------------------------------------------------------------
#  scripts/benchmark_action.sh
#
#  Bench every pallet up to three times.  A dispatch item's recorded
#  weight/reads/writes is *only* updated if it drifts in **every** run
#  (i.e. 3 × in a row).  Patched files are listed in
#    $PATCH_MARKER     so that a later CI step can commit them.
# ------------------------------------------------------------------
set -euo pipefail

PALLETS=(subtensor admin_utils commitments drand)

declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
  [swap]="../pallets/swap/src/pallet/mod.rs"
)

MAX_ATTEMPTS=3
THRESHOLD=15
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"
PATCH_MARKER="$SCRIPT_DIR/benchmark_patch_marker"
: >| "$PATCH_MARKER"

patch_field() {
  local file="$1" kind="$2" new="$3"
  [[ -f "$file" ]] || return 0
  grep -qF "$file" "$PATCH_MARKER" || echo "$file" >> "$PATCH_MARKER"

  case "$kind" in
    weight)
      perl -0777 -i -pe \
        's/(#[^\n]*?\[pallet::weight[^\]]*?Weight::from_parts\(\s*)[0-9_]+/$1'"$new"'/g' "$file"
      ;;
    reads)
      perl -0777 -i -pe \
        's/(#[^\n]*?\[pallet::weight[^\]]*?reads_writes\(\s*)[0-9_]+/$1'"$new"'/g' "$file"
      perl -0777 -i -pe \
        's/(#[^\n]*?\[pallet::weight[^\]]*?\.reads\(\s*)[0-9_]+/$1'"$new"'/g' "$file"
      ;;
    writes)
      perl -0777 -i -pe \
        's/(#[^\n]*?\[pallet::weight[^\]]*?reads_writes\(\s*[0-9_]+\s*,\s*)[0-9_]+/$1'"$new"'/g' "$file"
      perl -0777 -i -pe \
        's/(#[^\n]*?\[pallet::weight[^\]]*?\.writes\(\s*)[0-9_]+/$1'"$new"'/g' "$file"
      ;;
  esac
}

process_extr() {
  local extr="$1" meas_us="$2" meas_r="$3" meas_w="$4" file="$5"

  # convert µs → ps
  local meas_ps; meas_ps=$(awk -v x="$meas_us" 'BEGIN{printf "%.0f",x*1000000}')

  # scrape recorded numbers
  local rec_w rec_r rec_wr
  read rec_w rec_r rec_wr <<<"$(awk -v fn="$extr" '
    BEGIN{w=r=wr=""}
    /Weight::from_parts/{
      gsub(/.*Weight::from_parts\(/,""); gsub(/[[:space:]]*,.*$/,""); w=$0}
    /reads_writes\(/{
      gsub(/.*reads_writes\(/,""); gsub(/\).*/,""); split($0,a,","); r=a[1]; wr=a[2]}
    /\.reads\(/{
      gsub(/.*\.reads\(/,""); gsub(/\).*/,""); r=$0}
    /\.writes\(/{
      gsub(/.*\.writes\(/,""); gsub(/\).*/,""); wr=$0}
    $0~("pub[[:space:]]+fn[[:space:]]+"fn"\\("){print w,r,wr; exit}
  ' "$file")"

  rec_w=${rec_w//_/}; rec_r=${rec_r//_/}; rec_wr=${rec_wr//_/}
  [[ -z $rec_w ]] && rec_w=0; [[ -z $rec_r ]] && rec_r=0; [[ -z $rec_wr ]] && rec_wr=0

  local drift; drift=$(awk -v a="$meas_ps" -v b="$rec_w" \
                         'BEGIN{printf "%.1f",(a-b)*100/(b==0?1:b)}')
  local abs=${drift#-}; local d_int=${abs%%.*}

  # pretty print one‑liner
  printf '%-32s r:%5s→%-5s w:%5s→%-5s ps:%12s→%-12s drift:%6s%%\n' \
         "$extr" "$rec_r" "$meas_r" "$rec_wr" "$meas_w" "$rec_w" "$meas_ps" "$drift"

  # decide mismatches -------------------------------------------------
  local mismatch_weight=0 mismatch_reads=0 mismatch_writes=0
  (( d_int > THRESHOLD )) && mismatch_weight=1
  (( meas_r != rec_r ))   && mismatch_reads=1
  (( meas_w != rec_wr ))  && mismatch_writes=1

  # bump counters if mismatched
  if (( mismatch_weight )); then
    drift_cnt_weight["$extr"]=$(( drift_cnt_weight["$extr"] + 1 ))
    last_meas_weight["$extr"]=$meas_ps
  fi
  if (( mismatch_reads )); then
    drift_cnt_reads["$extr"]=$(( drift_cnt_reads["$extr"] + 1 ))
    last_meas_reads["$extr"]=$meas_r
  fi
  if (( mismatch_writes )); then
    drift_cnt_writes["$extr"]=$(( drift_cnt_writes["$extr"] + 1 ))
    last_meas_writes["$extr"]=$meas_w
  fi

  # flag that *some* mismatch occurred in this attempt (used by caller)
  (( mismatch_weight || mismatch_reads || mismatch_writes )) && SOME_DRIFT=1
}

# ------------------------------------------------------------------
# MAIN
# ------------------------------------------------------------------
echo "➤ Building runtime (production / benchmarks)…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

for pallet in "${PALLETS[@]}"; do
  dispatch_src="${DISPATCH_PATHS[$pallet]}"
  [[ -f "$SCRIPT_DIR/$dispatch_src" ]] || { echo "⚠️  no dispatch file for $pallet"; continue; }
  dispatch="$SCRIPT_DIR/$dispatch_src"

  # reset per‑pallet accumulators
  declare -A drift_cnt_weight drift_cnt_reads drift_cnt_writes
  declare -A last_meas_weight last_meas_reads last_meas_writes
  SOME_DRIFT=0

  echo -e "\n═════════════════════════════════════════════"
  echo   " Benchmarking $pallet   (source → $dispatch)"
  echo   "═════════════════════════════════════════════"

  for ((attempt=1; attempt<=MAX_ATTEMPTS; ++attempt)); do
    echo -e "\n— Attempt $attempt/$MAX_ATTEMPTS —"

    tmp=$(mktemp)
    trap 'rm -f "$tmp"' RETURN

    set +e
    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" \
      --genesis-builder=runtime --genesis-builder-preset=benchmark \
      --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" \
      --steps 50 --repeat 5 2>/dev/null | tee "$tmp" || true
    set -e

    # parse output
    extr=""; us=""; rd=""; wr=""
    flush(){ [[ -n $extr ]] && process_extr "$extr" "$us" "$rd" "$wr" "$dispatch"; extr=""; }
    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]] && { flush; extr=${BASH_REMATCH[1]}; continue; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]      && us=${BASH_REMATCH[1]}
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && rd=${BASH_REMATCH[1]}
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && wr=${BASH_REMATCH[1]}
    done <"$tmp"
    flush

    # Early exit if this run produced zero drift
    if [[ $SOME_DRIFT -eq 0 ]]; then
      echo "✅  Attempt $attempt had no drift — moving on."
      break
    fi
    SOME_DRIFT=0   # reset flag for next attempt
  done

  # ─────────── after all attempts, patch persistent drifts ──────────
  for extr in "${!drift_cnt_weight[@]}"; do
    if (( drift_cnt_weight["$extr"] >= MAX_ATTEMPTS )); then
      patch_field "$dispatch" weight "${last_meas_weight["$extr"]}"
    fi
  done
  for extr in "${!drift_cnt_reads[@]}"; do
    if (( drift_cnt_reads["$extr"] >= MAX_ATTEMPTS )); then
      patch_field "$dispatch" reads  "${last_meas_reads["$extr"]}"
    fi
  done
  for extr in "${!drift_cnt_writes[@]}"; do
    if (( drift_cnt_writes["$extr"] >= MAX_ATTEMPTS )); then
      patch_field "$dispatch" writes "${last_meas_writes["$extr"]}"
    fi
  done

  echo "✅  $pallet finished."
done

echo -e "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [[ -s "$PATCH_MARKER" ]]; then
  echo "💾  Dispatch files updated:"
  sed 's/^/   • /' "$PATCH_MARKER"
else
  echo "No dispatch files needed patching 🎉"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
exit 0
