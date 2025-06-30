#!/usr/bin/env bash
###############################################################################
# benchmark_action.sh
#
# Benchmarks selected pallets, validates weight / reads / writes, patches
# mismatches, commits and pushes (when AUTO_COMMIT_WEIGHTS=1).
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

THRESHOLD=15      # % drift tolerated
MAX_RETRIES=3
AUTO_COMMIT="${AUTO_COMMIT_WEIGHTS:-0}"

################################################################################
# Helpers
################################################################################
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

die()         { echo "❌ $1" >&2; exit 1; }
digits_only() { echo "${1//[^0-9]/}"; }
dec()         { local d; d="$(digits_only "$1")"; echo "$((10#${d:-0}))"; }
warn()        { echo "⚠️  $*"; }

###############################################################################
# Patch helpers – work for both attribute‑above and inline weight macros.
###############################################################################
patch_weight() {
  local fn="$1" new_w="$2" file="$3"
  local before; before=$(sha1sum "$file" | cut -d' ' -f1)
  FN="$fn" NEWW="$new_w" perl -0777 -i -pe '
    my $hits = 0;

    # 1) inline attribute on same line as pub fn
    $hits += s|
      (pub\s+fn\s+\Q$ENV{FN}\E\s*[^{]*?Weight::from_parts\(\s*)[0-9A-Za-z_]+
    |$1$ENV{NEWW}|xs;

    # 2) attribute ABOVE (allow comments etc. between ] and pub fn)
    $hits += s|
      (\#\s*\[pallet::weight[\s\S]*?Weight::from_parts\(\s*)[0-9A-Za-z_]+
      (?=[\s\S]{0,800}\]\s*[\s\S]{0,800}pub\s+fn\s+\Q$ENV{FN}\E\b)
    |$1$ENV{NEWW}|xs;

    END{ exit $hits ? 0 : 1 }
  ' "$file" || warn "patch_weight: $fn not substituted"
  [[ "$before" != "$(sha1sum "$file" | cut -d' ' -f1)" ]]
}

patch_reads_writes() {
  local fn="$1" new_r="$2" new_w="$3" file="$4"
  local before; before=$(sha1sum "$file" | cut -d' ' -f1)
  FN="$fn" NEWR="$new_r" NEWW="$new_w" perl -0777 -i -pe '
    my $h=0;

    # inline reads_writes(...)
    $h += s|
      (pub\s+fn\s+\Q$ENV{FN}\E\s*[^{]*?reads_writes\(\s*)([^,]+)(\s*,\s*)([^)]+)
    |$1$ENV{NEWR}$3$ENV{NEWW}|xs;

    # attribute‑above reads_writes(...)
    $h += s|
      (\#\s*\[pallet::weight[\s\S]*?reads_writes\(\s*)([^,]+)(\s*,\s*)([^)]+)
      (?=[\s\S]{0,800}\]\s*[\s\S]{0,800}pub\s+fn\s+\Q$ENV{FN}\E\b)
    |$1$ENV{NEWR}$3$ENV{NEWW}|xs;

    # inline .reads(...)
    $h += s|(pub\s+fn\s+\Q$ENV{FN}\E\s*[^{]*?\.reads\(\s*)([^)]+)|$1$ENV{NEWR}|xs;
    $h += s|(pub\s+fn\s+\Q$ENV{FN}\E\s*[^{]*?\.writes\(\s*)([^)]+)|$1$ENV{NEWW}|xs;

    # attribute‑above .reads(...)
    $h += s|
      (\#\s*\[pallet::weight[\s\S]*?\.reads\(\s*)([^)]+)
      (?=[\s\S]{0,800}\]\s*[\s\S]{0,800}pub\s+fn\s+\Q$ENV{FN}\E\b)
    |$1$ENV{NEWR}|xs;

    $h += s|
      (\#\s*\[pallet::weight[\s\S]*?\.writes\(\s*)([^)]+)
      (?=[\s\S]{0,800}\]\s*[\s\S]{0,800}pub\s+fn\s+\Q$ENV{FN}\E\b)
    |$1$ENV{NEWW}|xs;

    END{ exit $h ? 0 : 1 }
  ' "$file" || warn "patch_reads/writes: $fn not substituted"
  [[ "$before" != "$(sha1sum "$file" | cut -d' ' -f1)" ]]
}

git_commit_and_push() {
  local msg="$1"
  local branch; branch="$(git symbolic-ref --quiet --short HEAD || true)"
  [[ -z "$branch" ]] && die "Not on a branch – cannot push"

  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add "${PATCHED_FILES[@]}" || true

  if git diff --cached --quiet; then
    echo "ℹ️  No staged changes."
    git status --short
    return
  fi

  echo "==== diff preview ===="
  { git diff --cached --stat; git diff --cached | head -n 40; } || true
  echo "======================"

  git commit -m "$msg"
  git push origin "HEAD:${branch}" || die "Push to '${branch}' failed."
}

################################################################################
# Build runtime once
################################################################################
echo "Building runtime‑benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo -e "\n─────────────────────────────────────────────"
echo "Will benchmark: ${PALLET_LIST[*]}"
echo "─────────────────────────────────────────────"

PATCHED_FILES=()

################################################################################
# Benchmark loop
################################################################################
for pallet in "${PALLET_LIST[@]}"; do
  DISPATCH="$SCRIPT_DIR/${DISPATCH_PATHS[$pallet]}"
  [[ -f "$DISPATCH" ]] || die "dispatch missing: $DISPATCH"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    printf "\n════ Benchmark '%s' (try %d/%d) ════\n" "$pallet" "$attempt" "$MAX_RETRIES"
    TMP="$(mktemp)"; trap 'rm -f "$TMP"' EXIT

    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" --genesis-builder=runtime \
      --genesis-builder-preset=benchmark --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" --steps 50 --repeat 5 \
      | tee "$TMP"

    declare -A new_w=() new_r=() new_wr=()
    summary=(); issues=(); fail=0
    extr=""; mus=""; mreads=""; mwrites=""

    flush() {
      [[ -z "$extr" ]] && return
      local mps; mps=$(awk -v x="$mus" 'BEGIN{printf("%.0f", x*1000000)}')

      read -r cw cr cwr < <(awk -v fn="$extr" '
        /^\s*#\[pallet::call_index/ { next }
        /Weight::from_parts/      { t=$0; sub(/.*Weight::from_parts\(/,"",t); sub(/[^0-9A-Za-z_].*/,"",t); w=t }
        /reads_writes\(/          { t=$0; sub(/.*reads_writes\(/,"",t); sub(/\).*/,"",t);
                                    split(t,a,","); gsub(/[ \t_]/,"",a[1]); gsub(/[ \t_]/,"",a[2]); r=a[1]; wr=a[2] }
        /\.reads\(/               { t=$0; sub(/.*\.reads\(/,"",t); sub(/\).*/,"",t); gsub(/_/,"",t); r=t }
        /\.writes\(/              { t=$0; sub(/.*\.writes\(/,"",t); sub(/\).*/,"",t); gsub(/_/,"",t); wr=t }
        $0 ~ ("pub fn[[:space:]]+"fn"\\(") { print w,r,wr; exit }
      ' "$DISPATCH")

      cw=$(dec "${cw:-0}"); cr=$(dec "${cr:-0}"); cwr=$(dec "${cwr:-0}")

      drift=$([[ "$cw" -eq 0 ]] && echo 99999 || awk -v a="$mps" -v b="$cw" 'BEGIN{printf("%.1f",(a-b)/b*100)}')
      d_int=${drift#-}; d_int=${d_int%%.*}

      summary+=("$(printf "%-35s | reads %4s → %4s | writes %4s → %4s | weight %11s → %11s | drift %6s%%" \
                "$extr" "$cr" "$mreads" "$cwr" "$mwrites" "$cw" "$mps" "$drift")")

      (( mreads != cr ))   && { issues+=("[$extr] reads mismatch ($cr → $mreads)"); new_r[$extr]=$mreads; fail=1; }
      (( mwrites != cwr )) && { issues+=("[$extr] writes mismatch ($cwr → $mwrites)"); new_wr[$extr]=$mwrites; fail=1; }
      (( d_int > THRESHOLD )) && { issues+=("[$extr] weight drift ${drift}%"); new_w[$extr]=$mps; fail=1; }
    }

    while IFS= read -r l; do
      [[ $l =~ Extrinsic:\ \"([A-Za-z0-9_]+)\" ]] && { flush; extr="${BASH_REMATCH[1]}"; continue; }
      [[ $l =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]   && mus="${BASH_REMATCH[1]}"
      [[ $l =~ Reads[[:space:]]*=\ ([0-9]+) ]]     && mreads="${BASH_REMATCH[1]}"
      [[ $l =~ Writes[[:space:]]*=\ ([0-9]+) ]]    && mwrites="${BASH_REMATCH[1]}"
    done < "$TMP"; flush

    echo; printf '  %s\n' "${summary[@]}"
    (( fail == 0 )) && { echo "✅ '$pallet' within tolerance."; break; }

    printf '  ❌ %s\n' "${issues[@]}"
    (( attempt < MAX_RETRIES )) && { echo "→ Retrying …"; (( attempt++ )); continue; }

    # == Patch after max retries ==
    echo "❌ '$pallet' still failing → patching …"
    [[ "$AUTO_COMMIT" != "1" ]] && die "AUTO_COMMIT_WEIGHTS disabled."

    changed=0
    for fn in "${!new_w[@]}"; do
      patch_weight        "$fn" "${new_w[$fn]}"  "$DISPATCH" && changed=1
      r="${new_r[$fn]:-}"; w="${new_wr[$fn]:-}"
      [[ -n "$r" || -n "$w" ]] && patch_reads_writes "$fn" "${r:-0}" "${w:-0}" "$DISPATCH" && changed=1
    done
    (( changed )) && { PATCHED_FILES+=("$DISPATCH"); echo "✅ Patched '$pallet'"; } || warn "No substitutions for '$pallet'"
    break
  done
done

################################################################################
# Commit & push patches
################################################################################
if (( ${#PATCHED_FILES[@]} )); then
  echo -e "\n📦  Committing patched files …"
  git_commit_and_push "chore: auto‑update benchmark weights"
fi

echo -e "\n══════════════════════════════════════"
echo "All pallets processed ✔"
echo "══════════════════════════════════════"
