#!/usr/bin/env bash
###############################################################################
# benchmark_action.sh
#
# 1. Benchmark every pallet in PALLET_LIST.
# 2. Validate measured vs. code weights / reads / writes.
# 3. Per‑pallet retry logic:
#      • 3 benchmark attempts max.
#      • If still failing after 3 tries, patch the literals once,
#        commit & push (when AUTO_COMMIT_WEIGHTS=1), then **move on**
#        to the next pallet (no re‑benchmark of the patched pallet).
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

die()                 { echo "❌ $1" >&2; exit 1; }
digits_only()         { echo "${1//[^0-9]/}"; }
dec()                 { local d; d=$(digits_only "$1"); echo "$((10#${d:-0}))"; }

warn_no_subst() { echo "⚠️  patch for $1 :: no substitution made (pattern not found)"; }

patch_weight() {
  local fn="$1" new_w="$2" file="$3"
  local before after
  before=$(sha1sum "$file" | cut -d' ' -f1)
  perl -0777 -i -pe "
    my \$count = s|(pub\\s+fn\\s+\Q${fn}\E\\s*\\([^{}]*?Weight::from_parts\\(\\s*)[0-9A-Za-z_]+|\\1${new_w}|s;
    END{ exit \$count ? 0 : 1 }
  " "$file" || warn_no_subst "$fn (weight)"
  after=$(sha1sum "$file" | cut -d' ' -f1)
  [[ "$before" != "$after" ]]
}

patch_reads_writes() {
  local fn="$1" new_r="$2" new_w="$3" file="$4"
  local before after
  before=$(sha1sum "$file" | cut -d' ' -f1)
  perl -0777 -i -pe "
    my \$hit = 0;
    \$hit += s|(pub\\s+fn\\s+\Q${fn}\E\\s*\\([^{}]*?reads_writes\\(\\s*)([^,]+)(\\s*,\\s*)([^)]+)\\)|\\1${new_r}\\3${new_w}|s;
    \$hit += s|(pub\\s+fn\\s+\Q${fn}\E\\s*\\([^{}]*?\\.reads\\(\\s*)([^)]+)\\)|\\1${new_r}|s;
    \$hit += s|(pub\\s+fn\\s+\Q${fn}\E\\s*\\([^{}]*?\\.writes\\(\\s*)([^)]+)\\)|\\1${new_w}|s;
    END{ exit \$hit ? 0 : 1 }
  " "$file" || warn_no_subst "$fn (reads/writes)"
  after=$(sha1sum "$file" | cut -d' ' -f1)
  [[ "$before" != "$after" ]]
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
    echo "==== git status ===="
    git status --short
    echo "==== end status ===="
    return
  fi

  echo "==== diff preview ===="
  git diff --cached --stat
  git diff --cached --color | head -n 40
  echo "======================"

  git commit -m "$msg"

  if ! git push origin "HEAD:${branch}"; then
    echo "🚨 Push failed. Showing last commit:"
    git --no-pager log -1 --stat
    die "Push to '${branch}' failed."
  fi
}

################################################################################
# Build once
################################################################################
echo "Building runtime‑benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo
echo "─────────────────────────────────────────────"
echo " Will benchmark pallets: ${PALLET_LIST[*]}"
echo "─────────────────────────────────────────────"

PATCHED_FILES=()

################################################################################
# Main loop per pallet
################################################################################
for pallet in "${PALLET_LIST[@]}"; do
  DISPATCH_REL="${DISPATCH_PATHS[$pallet]:-}"
  [[ -z "$DISPATCH_REL" ]] && die "dispatch path undefined for '$pallet'"
  DISPATCH="$SCRIPT_DIR/$DISPATCH_REL"
  [[ -f "$DISPATCH" ]] || die "dispatch file missing: $DISPATCH"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    echo; echo "════ Benchmarking '$pallet' (attempt $attempt/$MAX_RETRIES) ════"

    TMP="$(mktemp)"; trap 'rm -f "$TMP"' EXIT
    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" --genesis-builder=runtime \
      --genesis-builder-preset=benchmark --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" --steps 50 --repeat 5 \
      | tee "$TMP"

    declare -A new_weight=() new_reads=() new_writes=()
    summary_lines=(); failure_lines=(); fail=0
    extr=""; meas_us=""; meas_reads=""; meas_writes=""

    flush_extr() {
      [[ -z "$extr" ]] && return
      local meas_ps; meas_ps=$(awk -v x="$meas_us" 'BEGIN{printf("%.0f", x*1000000)}')

      read -r code_w code_r code_wr < <(awk -v fn="$extr" '
        /^\s*#\[pallet::call_index/ { next }
        /Weight::from_parts/      { lw=$0; sub(/.*Weight::from_parts\(/,"",lw); sub(/[^0-9A-Za-z_].*/,"",lw); w=lw }
        /reads_writes\(/          { lw=$0; sub(/.*reads_writes\(/,"",lw); sub(/\).*/,"",lw);
                                    split(lw,io,","); gsub(/[ \t_]/,"",io[1]); gsub(/[ \t_]/,"",io[2]); r=io[1]; wr=io[2] }
        /\.reads\(/               { lw=$0; sub(/.*\.reads\(/,"",lw); sub(/\).*/,"",lw); gsub(/_/,"",lw); r=lw }
        /\.writes\(/              { lw=$0; sub(/.*\.writes\(/,"",lw); sub(/\).*/,"",lw); gsub(/_/,"",lw); wr=lw }
        $0 ~ ("pub fn[[:space:]]+"fn"\\(") { print w,r,wr; exit }
      ' "$DISPATCH")

      code_w=$(dec "${code_w:-0}")
      code_r=$(dec "${code_r:-0}")
      code_wr=$(dec "${code_wr:-0}")

      local drift
      drift=$([[ "$code_w" -eq 0 ]] && echo 99999 || awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{printf("%.1f", (a-b)/b*100)}')
      local abs_drift=${drift#-}; local drift_int=${abs_drift%%.*}

      summary_lines+=("$(printf "%-35s | reads %4s → %4s | writes %4s → %4s | weight %12s → %12s | drift %6s%%" \
        "$extr" "$code_r" "$meas_reads" "$code_wr" "$meas_writes" "$code_w" "$meas_ps" "$drift")")

      if (( meas_reads != code_r )); then
        failure_lines+=("[$extr] reads mismatch (code=$code_r, measured=$meas_reads)")
        new_reads[$extr]=$meas_reads;   fail=1
      fi
      if (( meas_writes != code_wr )); then
        failure_lines+=("[$extr] writes mismatch (code=$code_wr, measured=$meas_writes)")
        new_writes[$extr]=$meas_writes; fail=1
      fi
      if (( drift_int > THRESHOLD )); then
        failure_lines+=("[$extr] weight drift ${drift}% (code=$code_w, measured=$meas_ps)")
        new_weight[$extr]=$meas_ps;     fail=1
      fi
    }

    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]]      && { flush_extr; extr="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]       && { meas_us="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && { meas_reads="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && { meas_writes="${BASH_REMATCH[1]}"; continue; }
    done < "$TMP"
    flush_extr

    echo; printf '  %s\n' "${summary_lines[@]}"
    (( fail == 0 )) && { echo "✅ '$pallet' within tolerance."; break; }

    printf '  ❌ %s\n' "${failure_lines[@]}"

    if (( attempt < MAX_RETRIES )); then
      echo "→ Retry $((attempt+1))/$MAX_RETRIES …"; (( attempt++ )); continue
    fi

    ###########################################################################
    # Patch after final failure
    ###########################################################################
    echo "❌ '$pallet' still failing; patching …"
    [[ "$AUTO_COMMIT" != "1" ]] && die "AUTO_COMMIT_WEIGHTS disabled – exiting."

    changed=0
    for fn in "${!new_weight[@]}"; do
      patch_weight "$fn" "${new_weight[$fn]}" "$DISPATCH" && changed=1
      r="${new_reads[$fn]:-}"; w="${new_writes[$fn]:-}"
      [[ -n "$r" || -n "$w" ]] && patch_reads_writes "$fn" "${r:-0}" "${w:-0}" "$DISPATCH" && changed=1
    done

    if (( changed )); then
      PATCHED_FILES+=("$DISPATCH")
      echo "✅ Patched '$pallet'; moving on."
    else
      echo "⚠️  No modifications done for '$pallet' (patterns not found)."
    fi
    break
  done
done

################################################################################
# Commit & push
################################################################################
if (( ${#PATCHED_FILES[@]} )); then
  echo; echo "📦  Attempting to commit patched files …"
  git_commit_and_push "chore: auto‑update benchmark weights"
fi

echo
echo "══════════════════════════════════════"
echo "All pallets processed ✔"
echo "══════════════════════════════════════"
