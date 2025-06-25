#!/usr/bin/env bash
###############################################################################
# scripts/benchmark_action.sh
#
# • Benchmarks each pallet in PALLET_LIST.
# • Compares measured vs. code Weight / Reads / Writes.
# • Retries up to MAX_RETRIES, then (optionally) rewrites the code values,
#   commits, and pushes when AUTO_COMMIT_WEIGHTS=1 is set.
###############################################################################
set -euo pipefail

################################################################################
# Configuration
################################################################################
PALLET_LIST=(subtensor admin_utils commitments drand)   # add swap if desired

declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
  [swap]="../pallets/swap/src/pallet/mod.rs"
)

THRESHOLD=15           # allowed drift %
MAX_RETRIES=3
AUTO_COMMIT="${AUTO_COMMIT_WEIGHTS:-0}"

################################################################################
# Small helpers
################################################################################
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"
die() { echo "❌ $1" >&2; exit 1; }

# strip underscores & any trailing non‑digits, e.g. 12_345u64 → 12345
num() { local v="${1:-}"; v="${v//_/}"; v="${v%%[^0-9]*}"; echo "${v:-0}"; }

###############################################################################
# Patch helpers (invoked only when AUTO_COMMIT_WEIGHTS=1)
###############################################################################
regex_fn() { printf '%s' "$1" | sed 's/[][().*?+^$|{}]/\\&/g; s/_/\\_/g'; }

patch_weight() {
  local extr="$1" new="$2" file="$3"
  perl -0777 -i -pe '
    my $fn = qr/\b'"$(regex_fn "$extr")"'\b/;
    s{
      (pub\s+fn\s+$fn\s* \([^{}]*? Weight::from_parts\(\s*)
      \d[\d_]*(?:\s*u64)?
    }{$1'"$new"'}sx;
  ' "$file"
}

patch_reads_writes() {
  local extr="$1" new_r="$2" new_w="$3" file="$4"
  perl -0777 -i -pe '
    my $fn = qr/\b'"$(regex_fn "$extr")"'\b/;
    my ($r,$w) = ('"$new_r"','"$new_w"');
    s{
      (pub\s+fn\s+$fn\s* \([^{}]*? reads_writes\()\s*
      \d[\d_]*(?:\s*u64)?\s*,\s*\d[\d_]*(?:\s*u64)?\s*\)
    }{$1$r, $w)}sx;
    s{
      (pub\s+fn\s+$fn\s* \([^{}]*? \.reads\()\s*\d[\d_]*(?:\s*u64)?\s*\)
    }{$1$r)}sx;
    s{
      (pub\s+fn\s+$fn\s* \([^{}]*? \.writes\()\s*\d[\d_]*(?:\s*u64)?\s*\)
    }{$1$w)}sx;
  ' "$file"
}

git_commit_and_push() {
  local msg="$1"
  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add "${PATCHED_FILES[@]}"
  if ! git diff --cached --quiet; then
    git commit -m "$msg"
    git push
  fi
}

################################################################################
# Build once for all pallets
################################################################################
echo "Building runtime‑benchmarks …"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo
echo "─────────────────────────────────────────────"
echo " Will benchmark pallets: ${PALLET_LIST[*]}"
echo "─────────────────────────────────────────────"

PATCHED_FILES=()

################################################################################
# Main loop
################################################################################
for pallet in "${PALLET_LIST[@]}"; do
  DISPATCH_REL="${DISPATCH_PATHS[$pallet]:-}" || die "dispatch path missing for $pallet"
  DISPATCH="$SCRIPT_DIR/$DISPATCH_REL"
  [[ -f "$DISPATCH" ]] || die "dispatch file not found: $DISPATCH"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    echo
    echo "══════════════════════════════════════════════"
    echo "Benchmarking pallet: $pallet (attempt #$attempt)"
    echo "══════════════════════════════════════════════"

    TMP="$(mktemp)"
    trap 'rm -f "$TMP"' EXIT

    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" \
      --genesis-builder=runtime --genesis-builder-preset=benchmark \
      --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" \
      --steps 50 --repeat 5 | tee "$TMP"

    ##########################################################################
    # Parse benchmark output
    ##########################################################################
    declare -A new_wt=() new_r=() new_w=()
    summary=(); fail=0
    extr="" us="" rd="" wr=""

    finalize_extr() {
      [[ -z "$extr" ]] && return

      local meas_ps
      meas_ps=$(awk -v x="$us" 'BEGIN{printf("%.0f", x*1000000)}')

      # --- code‑side lookup ----------------------------------------------------
      local record
      record=$(awk -v extr="$extr" '
        /^\s*#\[pallet::call_index\(/ { next }
        /Weight::from_parts/{
          sub(/.*Weight::from_parts\(/,"",$0); sub(/[^0-9_].*/,"",$0); w=$0
        }
        /reads_writes\(/{
          sub(/.*reads_writes\(/,"",$0); sub(/\).*/,"",$0);
          split($0,io,","); r=io[1]; wr=io[2]
        }
        /\.reads\(/{
          sub(/.*\.reads\(/,"",$0); sub(/\).*/,"",$0); r=$0
        }
        /\.writes\(/{
          sub(/.*\.writes\(/,"",$0); sub(/\).*/,"",$0); wr=$0
        }
        $0 ~ ("pub fn[[:space:]]+"extr"\\("){ print w,r,wr; exit }
      ' "$DISPATCH") || true

      # Default values
      local cw=0 cr=0 cwrt=0
      if [[ -n "$record" ]]; then
        # shellcheck disable=SC2086
        set -- $record       # split into $1 $2 $3 (some may be empty)
        cw=$(num "$1"); cr=$(num "${2:-}"); cwrt=$(num "${3:-}")
      fi

      local mrd=$(num "$rd") mwr=$(num "$wr")

      # drift
      local drift; [[ "$cw" == 0 ]] && drift="99999" \
        || drift=$(awk -v a="$meas_ps" -v b="$cw" 'BEGIN{printf("%.1f",(a-b)/b*100)}')
      local drift_abs=${drift#-}; drift_abs=${drift_abs%%.*}

      summary+=("$(printf "%-28s | reads %3s → %3s | writes %3s → %3s | weight %11s → %11s | drift %6s%%" \
        "$extr" "$cr" "$mrd" "$cwrt" "$mwr" "$cw" "$meas_ps" "$drift")")

      # mismatches
      [[ $mrd -ne $cr      ]] && { new_r[$extr]=$mrd;  fail=1; }
      [[ $mwr -ne $cwrt    ]] && { new_w[$extr]=$mwr;  fail=1; }
      (( drift_abs > THRESHOLD )) && { new_wt[$extr]=$meas_ps; fail=1; }
    }

    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([A-Za-z0-9_]+)\" ]] && { finalize_extr; extr="${BASH_REMATCH[1]}"; us=""; rd=""; wr=""; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]               && us="${BASH_REMATCH[1]}"
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9_]+[0-9u]*) ]]  && rd="${BASH_REMATCH[1]}"
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9_]+[0-9u]*) ]] && wr="${BASH_REMATCH[1]}"
    done < "$TMP"
    finalize_extr

    echo; printf '  %s\n' "${summary[@]}"

    ##########################################################################
    # Decide pass / retry / patch
    ##########################################################################
    if (( fail == 0 )); then
      echo "✅  '$pallet' within ±${THRESHOLD}%."; break
    fi

    if (( attempt < MAX_RETRIES )); then
      echo "❌  Issues detected – retrying ($((attempt+1))/${MAX_RETRIES}) …"
      (( attempt++ )); continue
    fi

    echo "❌  '$pallet' still failing after $MAX_RETRIES attempts."

    if [[ "$AUTO_COMMIT" != "1" ]]; then
      die "AUTO_COMMIT_WEIGHTS disabled – aborting."
    fi

    echo "🛠   Auto‑patching $DISPATCH …"
    for e in "${!new_wt[@]}"; do
      patch_weight "$e" "${new_wt[$e]}" "$DISPATCH"
      patch_reads_writes "$e" "${new_r[$e]:-0}" "${new_w[$e]:-0}" "$DISPATCH"
    done
    PATCHED_FILES+=("$DISPATCH")

    echo "🔄   Re‑running benchmarks after patch …"
    attempt=1   # reset attempts
  done
done

################################################################################
# Commit & push if we changed anything
################################################################################
if (( ${#PATCHED_FILES[@]} )); then
  git_commit_and_push "chore: auto‑update benchmark weights"
  echo "✅  Auto‑patch committed & pushed."
fi

echo
echo "══════════════════════════════════════"
echo "All pallets validated ✔"
echo "══════════════════════════════════════"
