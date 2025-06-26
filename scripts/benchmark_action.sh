#!/usr/bin/env bash
###############################################################################
# benchmark_action.sh
###############################################################################
set -euo pipefail

################################################################################
# Configuration
################################################################################
PALLET_LIST=(subtensor admin_utils commitments drand)

declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
)

THRESHOLD=15
MAX_RETRIES=3
AUTO_COMMIT="${AUTO_COMMIT_WEIGHTS:-0}"

################################################################################
# Helpers
################################################################################
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

die()          { echo "❌ $1" >&2; exit 1; }
digits_only()  { echo "${1//[^0-9]/}"; }
dec()          { local d; d=$(digits_only "$1"); echo "$((10#${d:-0}))"; }

warn_no_subst() { echo "⚠️  No substitution made for $1 (pattern not found)"; }

# ---------- PATCH HELPERS ----------------------------------------------------
patch_weight() {
  local fn="$1" new="$2" file="$3"
  perl -0777 -i -pe '
    my $ok = s|
      (\#\s*\[pallet::weight[^\]]*?Weight::from_parts\(\s*)[0-9A-Za-z_]+
      (?=.*?pub\s+fn\s+'"${fn//\_/\\_}"'\b)  # look‑ahead to ensure same extrinsic
    |\1'"$new"'|sx;
    exit($ok ? 0 : 1);
  ' "$file" || warn_no_subst "${fn}::weight"
}

patch_reads_writes() {
  local fn="$1" rd="$2" wr="$3" file="$4"
  perl -0777 -i -pe '
    my $hit = 0;
    $hit += s|
      (\#\s*\[pallet::weight[^\]]*?reads_writes\(\s*)[^,]+(\s*,\s*)[^)\]]+
      (?=.*?pub\s+fn\s+'"${fn//\_/\\_}"'\b)
    |\1'"$rd"'\2'"$wr"'|sx;
    $hit += s|
      (\#\s*\[pallet::weight[^\]]*?\.reads\(\s*)[^)\]]+
      (?=.*?pub\s+fn\s+'"${fn//\_/\\_}"'\b)
    |\1'"$rd"'|sx;
    $hit += s|
      (\#\s*\[pallet::weight[^\]]*?\.writes\(\s*)[^)\]]+
      (?=.*?pub\s+fn\s+'"${fn//\_/\\_}"'\b)
    |\1'"$wr"'|sx;
    exit($hit ? 0 : 1);
  ' "$file" || warn_no_subst "${fn}::reads/writes"
}

git_commit_and_push() {
  local msg="$1"
  local branch
  branch="$(git symbolic-ref --quiet --short HEAD || true)"
  [[ -z "$branch" ]] && die "Not on a branch – cannot push"

  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add "${PATCHED_FILES[@]}" || true

  if git diff --cached --quiet; then
    echo "ℹ️  No staged changes detected."
    git status --short
    return
  fi

  echo "==== diff preview ===="
  git diff --cached --stat
  git diff --cached --color | head -n 40
  echo "======================"

  git commit -m "$msg"
  git push origin "HEAD:${branch}" || die "🚨 Push to '${branch}' failed."
}

################################################################################
# Build once
################################################################################
echo "Building runtime‑benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo -e "\n─────────────────────────────────────────────"
echo " Will benchmark pallets: ${PALLET_LIST[*]}"
echo "─────────────────────────────────────────────"

PATCHED_FILES=()

################################################################################
# Main loop
################################################################################
for pallet in "${PALLET_LIST[@]}"; do
  DISPATCH="${SCRIPT_DIR}/${DISPATCH_PATHS[$pallet]}"
  [[ -f "$DISPATCH" ]] || die "dispatch not found: $DISPATCH"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    echo -e "\n════ Benchmarking '$pallet' (attempt $attempt/$MAX_RETRIES) ════"
    TMP="$(mktemp)"; trap 'rm -f "$TMP"' EXIT
    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" --genesis-builder=runtime \
      --genesis-builder-preset=benchmark --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" --steps 50 --repeat 5 \
      | tee "$TMP"

    declare -A new_weight=() new_reads=() new_writes=()
    summary=(); detail=(); fail=0
    extr=""; meas_us=""; meas_reads=""; meas_writes=""

    flush() {
      [[ -z "$extr" ]] && return
      local meas_ps; meas_ps=$(awk -v x="$meas_us" 'BEGIN{printf("%.0f", x*1000000)}')
      read -r cw cr cwrt < <(awk -v fn="$extr" '
        /^\s*#\[pallet::weight/ { wLine=$0 }
        /Weight::from_parts/    { sub(/.*Weight::from_parts\(/,"",$0); sub(/[^0-9A-Za-z_].*/,"",$0); w=$0 }
        /\.reads\(/             { sub(/.*\.reads\(/,"",$0); sub(/\).*/,"",$0); gsub(/_/,""); r=$0 }
        /\.writes\(/            { sub(/.*\.writes\(/,"",$0); sub(/\).*/,"",$0); gsub(/_/,""); wr=$0 }
        /pub\s+fn\s+'$extr'\b/  { print w,r,wr; exit }
      ' "$DISPATCH")

      cw=$(dec "${cw:-0}"); cr=$(dec "${cr:-0}"); cwrt=$(dec "${cwrt:-0}")
      local drift
      drift=$([[ "$cw" -eq 0 ]] && echo 99999 || awk -v a="$meas_ps" -v b="$cw" 'BEGIN{printf("%.1f", (a-b)/b*100)}')
      local dInt=${drift#-}; dInt=${dInt%%.*}

      summary+=("$(printf "%-30s | reads %3s→%3s | writes %3s→%3s | weight %12s→%12s | drift %6s%%" \
        "$extr" "$cr" "$meas_reads" "$cwrt" "$meas_writes" "$cw" "$meas_ps" "$drift")")

      (( cr != meas_reads ))  && { detail+=("[$extr] reads $cr→$meas_reads");  new_reads[$extr]=$meas_reads;  fail=1; }
      (( cwrt != meas_writes )) && { detail+=("[$extr] writes $cwrt→$meas_writes"); new_writes[$extr]=$meas_writes; fail=1; }
      (( dInt > THRESHOLD ))   && { detail+=("[$extr] drift ${drift}%"); new_weight[$extr]=$meas_ps; fail=1; }
    }

    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]] && { flush; extr="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]  && { meas_us="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && { meas_reads="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && { meas_writes="${BASH_REMATCH[1]}"; continue; }
    done < "$TMP"
    flush

    printf '  %s\n' "${summary[@]}"
    (( fail == 0 )) && { echo "✅ '$pallet' within tolerance."; break; }

    printf '  ❌ %s\n' "${detail[@]}"
    (( attempt < MAX_RETRIES )) && { echo "→ Retrying …"; (( attempt++ )); continue; }

    # ---------- patch & move on ----------
    echo "❌ '$pallet' still failing; patching …"
    [[ "$AUTO_COMMIT" != "1" ]] && die "AUTO_COMMIT_WEIGHTS=0"

    changed=0
    for fn in "${!new_weight[@]}"; do
      patch_weight "$fn" "${new_weight[$fn]}" "$DISPATCH" && changed=1
      rd="${new_reads[$fn]:-}"; wr="${new_writes[$fn]:-}"
      [[ -n "$rd" || -n "$wr" ]] && patch_reads_writes "$fn" "${rd:-0}" "${wr:-0}" "$DISPATCH" && changed=1
    done
    (( changed )) && PATCHED_FILES+=("$DISPATCH")
    break
  done
done

################################################################################
# Commit / push
################################################################################
if (( ${#PATCHED_FILES[@]} )); then
  echo -e "\n📦  Committing patches …"
  git_commit_and_push "chore: auto-update benchmark weights"
fi

echo -e "\n══════════════════════════════════════"
echo   "All pallets processed ✔"
echo   "══════════════════════════════════════"
