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
digits_only()         { echo "${1//[^0-9]/}"; }       # strip _ and suffixes
dec()                 { local d; d=$(digits_only "$1"); echo "$((10#${d:-0}))"; }

# Patch helpers (used only when AUTO_COMMIT_WEIGHTS=1)
patch_weight() {
  local fn="$1" new_w="$2" file="$3"
  perl -0777 -i -pe "
    s|(pub\\s+fn\\s+\Q${fn}\E\\s*\\([^{}]*?Weight::from_parts\\(\\s*)[0-9A-Za-z_]+|\\1${new_w}|s
  " "$file"
}

patch_reads_writes() {
  local fn="$1" new_r="$2" new_w="$3" file="$4"
  perl -0777 -i -pe "
    s|(pub\\s+fn\\s+\Q${fn}\E\\s*\\([^{}]*?reads_writes\\(\\s*)([^,]+)(\\s*,\\s*)([^)]+)\\)|\\1${new_r}\\3${new_w}|s;
    s|(pub\\s+fn\\s+\Q${fn}\E\\s*\\([^{}]*?\\.reads\\(\\s*)([^)]+)\\)|\\1${new_r}|s;
    s|(pub\\s+fn\\s+\Q${fn}\E\\s*\\([^{}]*?\\.writes\\(\\s*)([^)]+)\\)|\\1${new_w}|s;
  " "$file"
}

git_commit_and_push() {
  local msg="$1"
  local branch
  branch="$(git symbolic-ref --quiet --short HEAD || true)"
  [[ -z "$branch" ]] && die "Not on a branch – cannot push"

  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add "${PATCHED_FILES[@]}"

  if git diff --cached --quiet; then
    echo "ℹ️  Nothing to commit; patches produced no diff."
    return
  fi

  git commit -m "$msg"
  # explicit remote/branch to avoid 'simple' push failures
  if ! git push origin "HEAD:${branch}"; then
    die "Failed to push patches to remote branch '${branch}'."
  fi
}

################################################################################
# Build once (faster than per‑pallet)
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
  [[ -z "$DISPATCH_REL" ]] && die "dispatch path not defined for pallet '$pallet'"
  DISPATCH="$SCRIPT_DIR/$DISPATCH_REL"
  [[ -f "$DISPATCH" ]] || die "dispatch file not found at $DISPATCH"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    echo
    echo "══════════════════════════════════════════════"
    echo "Benchmarking pallet: $pallet (attempt #$attempt)"
    echo "Dispatch file: $DISPATCH"
    echo "══════════════════════════════════════════════"

    TMP="$(mktemp)"
    trap 'rm -f "$TMP"' EXIT

    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" \
      --genesis-builder=runtime \
      --genesis-builder-preset=benchmark \
      --wasm-execution=compiled \
      --pallet "pallet_${pallet}" \
      --extrinsic "*" \
      --steps 50 \
      --repeat 5 | tee "$TMP"

    # ───── Parse benchmark output ─────
    declare -A new_weight=() new_reads=() new_writes=()
    summary_lines=(); failure_lines=(); fail=0

    extr=""; meas_us=""; meas_reads=""; meas_writes=""

    flush_extr() {
      [[ -z "$extr" ]] && return

      local meas_ps
      meas_ps=$(awk -v x="$meas_us" 'BEGIN{printf("%.0f", x*1000000)}')

      read -r code_w code_r code_wr < <(awk -v fn="$extr" '
        /^\s*#\[pallet::call_index/ { next }
        /Weight::from_parts/{
          lw=$0; sub(/.*Weight::from_parts\(/,"",lw); sub(/[^0-9A-Za-z_].*/,"",lw); w=lw
        }
        /reads_writes\(/{
          lw=$0; sub(/.*reads_writes\(/,"",lw); sub(/\).*/,"",lw);
          split(lw,io,","); gsub(/[ \t_]/,"",io[1]); gsub(/[ \t_]/,"",io[2]); r=io[1]; wr=io[2]
        }
        /\.reads\(/{
          lw=$0; sub(/.*\.reads\(/,"",lw); sub(/\).*/,"",lw); gsub(/_/,"",lw); r=lw
        }
        /\.writes\(/{
          lw=$0; sub(/.*\.writes\(/,"",lw); sub(/\).*/,"",lw); gsub(/_/,"",lw); wr=lw
        }
        $0 ~ ("pub fn[[:space:]]+"fn"\\("){ print w,r,wr; exit }
      ' "$DISPATCH")

      code_w="$(dec "${code_w:-0}")"
      code_r="$(dec "${code_r:-0}")"
      code_wr="$(dec "${code_wr:-0}")"

      local drift
      if [[ "$code_w" -eq 0 ]]; then
        drift=99999
      else
        drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{printf("%.1f", (a-b)/b*100)}')
      fi
      local abs_drift=${drift#-}; local drift_int=${abs_drift%%.*}

      summary_lines+=("$(printf "%-35s | reads %4s → %4s | writes %4s → %4s | weight %12s → %12s | drift %6s%%" \
        "$extr" "$code_r" "$meas_reads" "$code_wr" "$meas_writes" "$code_w" "$meas_ps" "$drift")")

      if (( meas_reads != code_r )); then
        failure_lines+=("[$extr] reads mismatch (code=$code_r, measured=$meas_reads)")
        new_reads["$extr"]="$meas_reads"
        fail=1
      fi
      if (( meas_writes != code_wr )); then
        failure_lines+=("[$extr] writes mismatch (code=$code_wr, measured=$meas_writes)")
        new_writes["$extr"]="$meas_writes"
        fail=1
      fi
      if (( drift_int > THRESHOLD )); then
        failure_lines+=("[$extr] weight drift ${drift}% (code=$code_w, measured=$meas_ps)")
        new_weight["$extr"]="$meas_ps"
        fail=1
      fi
    }

    while IFS= read -r line; do
      if   [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]];          then flush_extr; extr="${BASH_REMATCH[1]}"
      elif [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]];           then meas_us="${BASH_REMATCH[1]}"
      elif [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]];   then meas_reads="${BASH_REMATCH[1]}"
      elif [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]];  then meas_writes="${BASH_REMATCH[1]}"
      fi
    done < "$TMP"
    flush_extr

    echo; printf '  %s\n' "${summary_lines[@]}"

    if (( fail == 0 )); then
      echo "✅ Pallet '$pallet' benchmarks within ±${THRESHOLD}%."
      break
    fi

    printf '  ❌ %s\n' "${failure_lines[@]}"

    if (( attempt < MAX_RETRIES )); then
      echo "→ Retrying …"
      (( attempt++ ))
      continue
    fi

    ###########################################################################
    # After MAX_RETRIES — patch once, commit, and **continue to next pallet**
    ###########################################################################
    echo "❌ Pallet '$pallet' still failing after ${MAX_RETRIES} attempts."

    if [[ "$AUTO_COMMIT" != "1" ]]; then
      echo "AUTO_COMMIT_WEIGHTS disabled → exiting with error."
      exit 1
    fi

    echo "🛠  Auto‑patching $DISPATCH …"
    for fn in "${!new_weight[@]}"; do
      [[ -n "${new_weight[$fn]}" ]] && patch_weight "$fn" "${new_weight[$fn]}" "$DISPATCH"
      r="${new_reads[$fn]:-}"; w="${new_writes[$fn]:-}"
      [[ -n "$r" || -n "$w" ]] && patch_reads_writes "$fn" "${r:-0}" "${w:-0}" "$DISPATCH"
    done
    PATCHED_FILES+=("$DISPATCH")

    echo "✅ Patched $pallet; moving on to next pallet."
    break   # move to next pallet
  done      # retry loop
done        # pallet loop

################################################################################
# Commit & push any patches
################################################################################
if [[ "${#PATCHED_FILES[@]}" -gt 0 ]]; then
  echo; echo "📦  Committing updated weight files …"
  git_commit_and_push "chore: auto‑update benchmark weights"
  echo "✅ Auto‑patch committed & pushed."
fi

echo
echo "══════════════════════════════════════"
echo "All pallets processed ✔"
echo "══════════════════════════════════════"
