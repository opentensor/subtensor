#!/usr/bin/env bash
###############################################################################
# benchmark_action.sh
# Benchmarks pallets, patches weight literals when mismatched and pushes the
# fix (if AUTO_COMMIT_WEIGHTS=1).
###############################################################################
set -u  # we *do not* use `set -e` globally; we handle errors manually
set -o pipefail

###############################  CONFIG  ######################################
PALLET_LIST=(subtensor admin_utils commitments drand)

declare -A DISPATCH=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
)

THRESHOLD=15          # % drift tolerated
MAX_RETRIES=3
AUTO_COMMIT="${AUTO_COMMIT_WEIGHTS:-0}"

###############################  HELPERS  #####################################
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

blue()  { printf "\033[34m%s\033[0m\n" "$*"; }
warn()  { printf "\033[33m⚠️  %s\033[0m\n" "$*"; }
die()   { printf "\033[31m❌ %s\033[0m\n" "$*"; exit 1; }
digits(){ echo "${1//[^0-9]/}"; }
dec()   { local d; d=$(digits "$1"); echo "$((10#${d:-0}))"; }

hash_of() { sha1sum "$1" | cut -d' ' -f1; }

# ---- patch routines ---------------------------------------------------------
patch_weight() {
  local fn="$1" new="$2" file="$3" before after; before=$(hash_of "$file")
  FN="$fn" NEW="$new" perl -0777 -i -pe '
     my $hit=0;
     $hit += s|(pub\s+fn\s+\Q$ENV{FN}\E[^{]*?Weight::from_parts\(\s*)[0-9A-Za-z_]+|$1$ENV{NEW}|s;
     $hit += s|(\#\s*\[pallet::weight[\s\S]*?Weight::from_parts\(\s*)[0-9A-Za-z_]+
              (?=[\s\S]{0,800}\]\s*[\s\S]{0,800}pub\s+fn\s+\Q$ENV{FN}\E\b)|$1$ENV{NEW}|xs;
     END{ exit $hit ? 0 : 1 }
  ' "$file" || warn "no weight match for $fn"
  after=$(hash_of "$file"); [[ $before != $after ]]
}

patch_rw() {
  local fn="$1" nr="$2" nw="$3" file="$4" before after; before=$(hash_of "$file")
  FN="$fn" NR="$nr" NW="$nw" perl -0777 -i -pe '
     my $h=0;
     $h += s|(pub\s+fn\s+\Q$ENV{FN}\E[^{]*?reads_writes\(\s*)([^,]+)(\s*,\s*)([^)]+)|$1$ENV{NR}$3$ENV{NW}|s;
     $h += s|(\#\s*\[pallet::weight[\s\S]*?reads_writes\(\s*)([^,]+)(\s*,\s*)([^)]+)
            (?=[\s\S]{0,800}\]\s*[\s\S]{0,800}pub\s+fn\s+\Q$ENV{FN}\E\b)|$1$ENV{NR}$3$ENV{NW}|xs;
     $h += s|(pub\s+fn\s+\Q$ENV{FN}\E[^{]*?\.reads\(\s*)([^)]+)|$1$ENV{NR}|s;
     $h += s|(pub\s+fn\s+\Q$ENV{FN}\E[^{]*?\.writes\(\s*)([^)]+)|$1$ENV{NW}|s;
     $h += s|(\#\s*\[pallet::weight[\s\S]*?\.reads\(\s*)([^)]+)
            (?=[\s\S]{0,800}\]\s*[\s\S]{0,800}pub\s+fn\s+\Q$ENV{FN}\E\b)|$1$ENV{NR}|xs;
     $h += s|(\#\s*\[pallet::weight[\s\S]*?\.writes\(\s*)([^)]+)
            (?=[\s\S]{0,800}\]\s*[\s\S]{0,800}pub\s+fn\s+\Q$ENV{FN}\E\b)|$1$ENV{NW}|xs;
     END{ exit $h ? 0 : 1 }
  ' "$file" || warn "no R/W match for $fn"
  after=$(hash_of "$file"); [[ $before != $after ]]
}

commit_and_push() {
  local msg="$1"
  local branch; branch=$(git symbolic-ref --quiet --short HEAD) || die "detached HEAD"
  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add "${PATCHED[@]}" || true

  if git diff --cached --quiet; then
    warn "nothing to commit"; return
  fi

  { git diff --cached --stat && git diff --cached | head -n 30; } || true
  git commit -m "$msg"
  git push origin "HEAD:$branch" || die "push failed"
}

####################   BUILD RUNTIME ONCE  ####################################
blue "Building runtime‑benchmarks …"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

PATCHED=()

###########################   MAIN LOOP   #####################################
for pallet in "${PALLET_LIST[@]}"; do
  FILE="$SCRIPT_DIR/${DISPATCH[$pallet]}"
  [[ -f "$FILE" ]] || die "dispatch missing: $FILE"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    blue "▶  $pallet (attempt $attempt/$MAX_RETRIES)"
    TMP=$(mktemp)

    # --- run benchmark (non‑fatal) ------------------------------------------
    set +e
    ./target/production/node-subtensor benchmark pallet \
       --runtime "$RUNTIME_WASM" --genesis-builder=runtime \
       --genesis-builder-preset=benchmark --wasm-execution=compiled \
       --pallet "pallet_${pallet}" --extrinsic "*" --steps 50 --repeat 5 \
       | tee "$TMP"
    bench_rc=$?
    set -e
    (( bench_rc != 0 )) && warn "benchmark exited $bench_rc (continuing)"

    # --- parse --------------------------------------------------------------
    declare -A NEW_W=() NEW_R=() NEW_WR=()
    summary=() fails=() fail=0
    extr=""; mus=0; mr=0; mw=0

    finalize() {
      [[ -z $extr ]] && return
      local mps; mps=$(awk -v x="$mus" 'BEGIN{printf "%.0f", x*1000000}')
      read -r cw cr cwr < <(awk -v fn="$extr" '
        /^\s*#\[pallet::call_index/ { next }
        /Weight::from_parts/      { t=$0; sub(/.*Weight::from_parts\(/,"",t); sub(/[^0-9A-Za-z_].*/,"",t); w=t }
        /reads_writes\(/          { t=$0; sub(/.*reads_writes\(/,"",t); sub(/\).*/,"",t);
                                    split(t,a,","); gsub(/[ \t_]/,"",a[1]); gsub(/[ \t_]/,"",a[2]); r=a[1]; wr=a[2] }
        /\.reads\(/               { t=$0; sub(/.*\.reads\(/,"",t); sub(/\).*/,"",t); gsub(/_/,"",t); r=t }
        /\.writes\(/              { t=$0; sub(/.*\.writes\(/,"",t); sub(/\).*/,"",t); gsub(/_/,"",t); wr=t }
        $0 ~ ("pub fn[[:space:]]+"fn"\\(") { print w,r,wr; exit }
      ' "$FILE")
      cw=$(dec "$cw"); cr=$(dec "$cr"); cwr=$(dec "$cwr")
      drift=$([[ $cw -eq 0 ]] && echo 99999 || awk -v a="$mps" -v b="$cw" 'BEGIN{printf "%.1f",(a-b)/b*100}')
      abs=${drift#-}; abs=${abs%%.*}

      summary+=("$(printf "%-30s | r %3s→%3s | w %3s→%3s | wt %11s→%11s | %6s%%" \
        "$extr" "$cr" "$mr" "$cwr" "$mw" "$cw" "$mps" "$drift")")

      (( mr != cr ))   && { fails+=("$extr R $cr→$mr"); NEW_R[$extr]=$mr; fail=1; }
      (( mw != cwr ))  && { fails+=("$extr W $cwr→$mw"); NEW_WR[$extr]=$mw; fail=1; }
      (( abs > THRESHOLD )) && { fails+=("$extr wt drift"); NEW_W[$extr]=$mps; fail=1; }
    }

    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([A-Za-z0-9_]+)\" ]] && { finalize; extr="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]   && mus="${BASH_REMATCH[1]}"
      [[ $line =~ Reads[[:space:]]*=\ ([0-9]+) ]]     && mr="${BASH_REMATCH[1]}"
      [[ $line =~ Writes[[:space:]]*=\ ([0-9]+) ]]    && mw="${BASH_REMATCH[1]}"
    done < "$TMP"; finalize; rm "$TMP"

    printf '  %s\n' "${summary[@]}"
    (( fail == 0 )) && { blue "✓ $pallet OK"; break; }

    printf '  ❌ %s\n' "${fails[@]}"
    (( attempt < MAX_RETRIES )) && { attempt=$((attempt+1)); continue; }

    # --- patch ----------------------------------------------------------------
    blue "patching $pallet …"
    [[ "$AUTO_COMMIT" != 1 ]] && die "AUTO_COMMIT_WEIGHTS disabled"

    changed=0
    for fn in "${!NEW_W[@]}"; do
      patch_weight "$fn" "${NEW_W[$fn]}" "$FILE" && changed=1
      r="${NEW_R[$fn]:-}" w="${NEW_WR[$fn]:-}"
      [[ -n $r || -n $w ]] && patch_rw "$fn" "${r:-0}" "${w:-0}" "$FILE" && changed=1
    done
    (( changed )) && PATCHED+=("$FILE") || warn "no substitutions in $FILE"
    break
  done
done

[[ ${#PATCHED[@]} -gt 0 ]] && commit_and_push "chore: auto‑update benchmark weights"

blue "──────────── All pallets processed ✔ ────────────"
