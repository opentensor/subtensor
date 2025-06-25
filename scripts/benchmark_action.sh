#!/usr/bin/env bash
# scripts/benchmark_action.sh
# ------------------------------------------------------------
# Bench every pallet once, patch any weight/read/write drift,
# leave a marker with changed files, and exit 0.
# The commit/push happens in the workflow’s final “Commit any
# updated weights” step.
# ------------------------------------------------------------
set -euo pipefail

PALLETS=(subtensor admin_utils commitments drand)

declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
  [swap]="../pallets/swap/src/pallet/mod.rs"
)

THRESHOLD=15
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"
PATCH_MARKER="$SCRIPT_DIR/benchmark_patch_marker"
: >| "$PATCH_MARKER"

###############################################################################
# PATCH ENGINE (robust against constants / generics / multiline attrs)
###############################################################################
patch_field() {
  local file="$1" _extr="$2" kind="$3" new="$4"

  [[ -f "$file" ]] || return 0
  grep -qF "$file" "$PATCH_MARKER" || echo "$file" >> "$PATCH_MARKER"

  local perl_expr
  case "$kind" in
    weight)
      perl_expr='s{
        (#[^\n]*?\[pallet::weight[^\]]*?Weight::from_parts[^\(]*\(\s*)
        [0-9A-Za-z_]+
      }{$1'"$new"'}xg'
      ;;
    reads)
      perl_expr='s{
        (#[^\n]*?\[pallet::weight[^\]]*?reads_writes\(\s*)
        [0-9A-Za-z_]+
      }{$1'"$new"'}xg;
      s{
        (#[^\n]*?\[pallet::weight[^\]]*?\.reads\(\s*)
        [0-9A-Za-z_]+
      }{$1'"$new"'}xg'
      ;;
    writes)
      perl_expr='s{
        (#[^\n]*?\[pallet::weight[^\]]*?reads_writes\(\s*[0-9A-Za-z_]+\s*,\s*)
        [0-9A-Za-z_]+
      }{$1'"$new"'}xg;
      s{
        (#[^\n]*?\[pallet::weight[^\]]*?\.writes\(\s*)
        [0-9A-Za-z_]+
      }{$1'"$new"'}xg'
      ;;
  esac

  perl -0777 -i -pe "$perl_expr" "$file"
}

###############################################################################
# EXTRINSIC PARSER
###############################################################################
process_extr() {
  local extr="$1" meas_us="$2" meas_r="$3" meas_w="$4" file="$5"

  # µs → ps
  local meas_ps; meas_ps=$(awk -v x="$meas_us" 'BEGIN{printf "%.0f",x*1000000}')

  # scrape recorded values in dispatch file
  local rec_w rec_r rec_wr
  read rec_w rec_r rec_wr <<<"$(awk -v fn="$extr" '
    BEGIN{w="";r="";wr=""}
    /Weight::from_parts/{
      gsub(/.*Weight::from_parts\(/,""); gsub(/[[:space:]]*,.*$/,""); w=$0
    }
    /reads_writes\(/{
      gsub(/.*reads_writes\(/,""); gsub(/\).*/,""); split($0,a,","); r=a[1]; wr=a[2]
    }
    /\.reads\(/{
      gsub(/.*\.reads\(/,""); gsub(/\).*/,""); r=$0
    }
    /\.writes\(/{
      gsub(/.*\.writes\(/,""); gsub(/\).*/,""); wr=$0
    }
    $0~("pub[[:space:]]+fn[[:space:]]+"fn"\\("){print w,r,wr; exit}
  ' "$file")"

  rec_w=${rec_w//_/}; rec_r=${rec_r//_/}; rec_wr=${rec_wr//_/}
  [[ -z "$rec_w"  ]] && rec_w=0
  [[ -z "$rec_r"  ]] && rec_r=0
  [[ -z "$rec_wr" ]] && rec_wr=0

  local drift; drift=$(awk -v a="$meas_ps" -v b="$rec_w" 'BEGIN{printf "%.1f",(a-b)*100.0/(b==0?1:b)}')
  local abs=${drift#-}; local d_int=${abs%%.*}

  # patch when needed
  (( meas_r != rec_r ))   && patch_field "$file" "$extr" reads  "$meas_r"
  (( meas_w != rec_wr ))  && patch_field "$file" "$extr" writes "$meas_w"
  (( d_int  > THRESHOLD ))&& patch_field "$file" "$extr" weight "$meas_ps"

  printf '%-32s r:%5s→%-5s w:%5s→%-5s ps:%12s→%-12s drift:%6s%%\n' \
         "$extr" "$rec_r" "$meas_r" "$rec_wr" "$meas_w" "$rec_w" "$meas_ps" "$drift"
}

###############################################################################
# MAIN
###############################################################################
echo "➤ Building runtime (production / benchmarks)…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

for pallet in "${PALLETS[@]}"; do
  dispatch_src="${DISPATCH_PATHS[$pallet]}"
  [[ -f "$SCRIPT_DIR/$dispatch_src" ]] || { echo "⚠️  no dispatch file for $pallet"; continue; }
  dispatch="$SCRIPT_DIR/$dispatch_src"

  echo -e "\n═════════════════════════════════════════════"
  echo   " Benchmarking $pallet – dispatch file: $dispatch"
  echo   "═════════════════════════════════════════════"

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

  extr=""; us=""; rd=""; wr=""
  flush(){ [[ -n $extr ]] && process_extr "$extr" "$us" "$rd" "$wr" "$dispatch"; extr=""; }
  while IFS= read -r line; do
    [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]] && { flush; extr=${BASH_REMATCH[1]}; continue; }
    [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]  && us=${BASH_REMATCH[1]}
    [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && rd=${BASH_REMATCH[1]}
    [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && wr=${BASH_REMATCH[1]}
  done <"$tmp"
  flush

  echo "✅  $pallet complete."
done

echo -e "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [[ -s "$PATCH_MARKER" ]]; then
  echo "💾  Dispatch files updated:"
  sed 's/^/   • /' "$PATCH_MARKER"
else
  echo "No dispatch files needed patching 🎉"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
exit 0
