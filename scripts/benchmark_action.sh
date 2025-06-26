#!/usr/bin/env bash
set -euo pipefail
###############################################################################
# benchmark_action.sh – CI helper
###############################################################################
PALLET_LIST=(subtensor admin_utils commitments drand)

declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
)

THRESHOLD=15            # allowed weight drift %
MAX_RETRIES=3
AUTO_COMMIT="${AUTO_COMMIT_WEIGHTS:-0}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

die()          { echo "❌ $1" >&2; exit 1; }
digits_only()  { echo "${1//[^0-9]/}"; }
dec()          { local n; n=$(digits_only "$1"); echo "$((10#${n:-0}))"; }

# ---------------------------------------------------------------------------
# Patch helpers – operate strictly inside the attribute that belongs to the
# target `pub fn`, so ordering of attribute / function doesn't matter.
# ---------------------------------------------------------------------------
do_subst() { perl -0777 -i -pe "$1" "$2" || return 1; }
patch_weight() {       # args: fn new file
  do_subst '
    my $fn=q{'$1'};
    s{
      (\#\s*\[pallet::weight[^\]]*?Weight::from_parts\()\s*[0-9A-Za-z_]+
      (?=[^\]]*\]\s*pub\s+fn\s+$fn\b)
    }{$1'$2'}sx' "$3"
}
patch_reads() {        # args: fn new file
  do_subst '
    my $fn=q{'$1'};
    s{
      (\.reads\(\s*)[0-9A-Za-z_]+
      (?=[^\]]*\]\s*pub\s+fn\s+$fn\b)
    }{$1'$2'}sx' "$3"
}
patch_writes() {       # args: fn new file
  do_subst '
    my $fn=q{'$1'};
    s{
      (\.writes\(\s*)[0-9A-Za-z_]+
      (?=[^\]]*\]\s*pub\s+fn\s+$fn\b)
    }{$1'$2'}sx' "$3"
}

git_commit_and_push() {
  local msg="$1"
  local branch; branch=$(git symbolic-ref --quiet --short HEAD || true)
  [[ -z "$branch" ]] && die "detached HEAD – cannot push"
  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add "${PATCHED_FILES[@]}" || true
  if git diff --cached --quiet; then
    echo "ℹ️  Nothing changed — skipping commit."
    return
  fi
  echo "🔍 Staged diff:"; git diff --cached --stat
  git commit -m "$msg"
  git push origin "HEAD:${branch}" || die "push failed"
}

# ---------------------------------------------------------------------------
echo "🔨 Building runtime with benchmarks …"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

PATCHED_FILES=()
for pallet in "${PALLET_LIST[@]}"; do
  DISPATCH="$SCRIPT_DIR/${DISPATCH_PATHS[$pallet]}"
  [[ -f $DISPATCH ]] || die "dispatch path missing for $pallet"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    echo -e "\n════ Benchmark $pallet (attempt $attempt/$MAX_RETRIES) ════"
    TMP=$(mktemp); trap 'rm -f "$TMP"' EXIT
    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" --genesis-builder=runtime \
      --genesis-builder-preset=benchmark --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" --steps 50 --repeat 5 \
      | tee "$TMP"

    declare -A updW updR updWr
    summary=() failLines=(); fail=0
    extr=""; us=""; rd=""; wr=""

    finish_extr() {
      [[ -z $extr ]] && return
      ps=$(awk -v x="$us" 'BEGIN{printf("%.0f",x*1000000)}')
      read -r cw cr cwr < <(awk -v fn="$extr" '
        /^\s*\#\s*\[pallet::weight/ { attr=$0 }
        /Weight::from_parts/        { sub(/.*Weight::from_parts\(/,"",$0); sub(/[^0-9A-Za-z_].*/,"",$0); w=$0 }
        /\.reads\(/                 { sub(/.*\.reads\(/,"",$0); sub(/\).*/,"",$0); gsub(/_/,""); r=$0 }
        /\.writes\(/                { sub(/.*\.writes\(/,"",$0); sub(/\).*/,"",$0); gsub(/_/,""); wr=$0 }
        /pub\s+fn\s+'$extr'\b/      { print w,r,wr; exit }
      ' "$DISPATCH")
      cw=$(dec "${cw:-0}"); cr=$(dec "${cr:-0}"); cwr=$(dec "${cwr:-0}")
      drift=$([[ $cw -eq 0 ]] && echo 99999 || awk -v a=$ps -v b=$cw 'BEGIN{printf("%.1f",(a-b)/b*100)}')
      summary+=("$(printf "%-28s | r %3s→%3s | w %3s→%3s | wt %11s→%11s | Δ% 6s%%" \
        "$extr" "$cr" "$rd" "$cwr" "$wr" "$cw" "$ps" "$drift")")
      (( rd != cr ))   && { failLines+=("$extr · reads $cr→$rd");   updR[$extr]=$rd;   fail=1; }
      (( wr != cwr ))  && { failLines+=("$extr · writes $cwr→$wr"); updWr[$extr]=$wr; fail=1; }
      driftInt=${drift#-}; driftInt=${driftInt%%.*}
      (( driftInt > THRESHOLD )) && { failLines+=("$extr · drift ${drift}%"); updW[$extr]=$ps; fail=1; }
    }

    while IFS= read -r l; do
      [[ $l =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]] && { finish_extr; extr=${BASH_REMATCH[1]}; continue; }
      [[ $l =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]  && { us=${BASH_REMATCH[1]}; continue; }
      [[ $l =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && { rd=${BASH_REMATCH[1]}; continue; }
      [[ $l =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && { wr=${BASH_REMATCH[1]}; continue; }
    done <"$TMP"; finish_extr

    printf '  %s\n' "${summary[@]}"
    (( fail == 0 )) && { echo "✅ $pallet OK"; break; }

    printf '  ❌ %s\n' "${failLines[@]}"
    (( attempt < MAX_RETRIES )) && { (( attempt++ )); continue; }

    echo "🔧 Patching $pallet …"
    [[ $AUTO_COMMIT == 1 ]] || die "AUTO_COMMIT_WEIGHTS disabled"

    changed=0
    for fn in "${!updW[@]}"; do
      patch_weight  "$fn" "${updW[$fn]}"  "$DISPATCH" && changed=1
      [[ ${updR[$fn]+x} ]] && patch_reads  "$fn" "${updR[$fn]}" "$DISPATCH" && changed=1
      [[ ${updWr[$fn]+x} ]]&& patch_writes "$fn" "${updWr[$fn]}" "$DISPATCH" && changed=1
    done
    (( changed )) && PATCHED_FILES+=("$DISPATCH")
    break
  done
done

# ---------------------------------------------------------------------------
[[ ${#PATCHED_FILES[@]} -gt 0 ]] && git_commit_and_push "chore: auto‑update benchmark weights"
echo -e "\n🎉 All pallets processed."
