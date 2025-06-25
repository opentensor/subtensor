#!/usr/bin/env bash
###############################################################################
# benchmark_action.sh
#
# 1.  Benchmarks every requested pallet.
# 2.  Validates measured vs. code values.
# 3.  After 3 failed attempts it **updates the code**, commits, and pushes
#     the corrected weight / read / write values automatically.
#
# NOTE: Requires the env var `AUTO_COMMIT_WEIGHTS=1` (set in the workflow).
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

THRESHOLD=15       # max allowed drift %
MAX_RETRIES=3
AUTO_COMMIT="${AUTO_COMMIT_WEIGHTS:-0}"

################################################################################
# Helpers
################################################################################
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

function die() { echo "❌ $1" >&2; exit 1; }

###############################################################################
# Patch helpers – only invoked when auto‑commit is enabled
###############################################################################
# Regex‑safe function name (escapes underscores)
function regex_fn() { echo "$1" | sed 's/_/\\_/g'; }

# Replace a numeric argument inside Weight::from_parts(...)
function patch_weight() {
  local extr="$1" new_weight="$2" file="$3"
  perl -0777 -i -pe "
    s|(pub\\s+fn\\s+$(regex_fn "${extr}")\\s*\\([^{}]*?)Weight::from_parts\\(\\s*\\d+[\\d_]*|\\1Weight::from_parts(${new_weight}|s
  " "$file"
}

# Replace reads_writes(X, Y) *or* .reads(X) / .writes(Y)
function patch_reads_writes() {
  local extr="$1" new_r="$2" new_w="$3" file="$4"
  perl -0777 -i -pe "
    s|(pub\\s+fn\\s+$(regex_fn "${extr}")\\s*\\([^{}]*?)reads_writes\\(\\s*\\d+\\s*,\\s*\\d+\\s*\\)|\\1reads_writes(${new_r}, ${new_w})|s;
    s|(pub\\s+fn\\s+$(regex_fn "${extr}")\\s*\\([^{}]*?)\\.reads\\(\\s*\\d+\\s*\\)|\\1.reads(${new_r})|s;
    s|(pub\\s+fn\\s+$(regex_fn "${extr}")\\s*\\([^{}]*?)\\.writes\\(\\s*\\d+\\s*\\)|\\1.writes(${new_w})|s;
  " "$file"
}

function git_commit_and_push() {
  local msg="$1"
  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add ${PATCHED_FILES[*]}
  if ! git diff --cached --quiet; then
    git commit -m "$msg"
    git push
  fi
}

###############################################################################
# Benchmark logic
###############################################################################
echo "Building runtime‑benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo
echo "─────────────────────────────────────────────"
echo " Will benchmark pallets: ${PALLET_LIST[*]}"
echo "─────────────────────────────────────────────"

PATCHED_FILES=()   # tracks which files we touched (for the final commit)

for pallet in "${PALLET_LIST[@]}"; do
  DISPATCH_REL="${DISPATCH_PATHS[$pallet]:-}"
  [[ -z "$DISPATCH_REL" ]] && die "dispatch path not defined for pallet '$pallet'"
  DISPATCH="$SCRIPT_DIR/$DISPATCH_REL"
  [[ -f "$DISPATCH" ]]   || die "dispatch file not found at $DISPATCH"

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
      --repeat 5 \
      | tee "$TMP"

    # ------------------------------------------------------------------
    # Parse results
    # ------------------------------------------------------------------
    declare -A new_weight=()  new_reads=()  new_writes=()
    summary_lines=(); failures=(); fail=0

    function finalize_extr() {
      [[ -z "$extr" ]] && return
      # Convert μs → ps
      local meas_ps
      meas_ps=$(awk -v x="$meas_us" 'BEGIN{printf("%.0f", x * 1000000)}')

      # Lookup code values
      local code
      code=$(awk -v extr="$extr" '
        /^\s*#\[pallet::call_index\(/ { next }
        /Weight::from_parts/{
          lw=$0; sub(/.*Weight::from_parts\(/,"",lw); sub(/[^0-9_].*/,"",lw); gsub(/_/,"",lw); w=lw
        }
        /reads_writes\(/{
          lw=$0; sub(/.*reads_writes\(/,"",lw); sub(/\).*/,"",lw); split(lw,io,","); gsub(/[ \t]/,"",io[1]); gsub(/[ \t]/,"",io[2]); r=io[1]; wr=io[2]
        }
        /\.reads\(/{
          lw=$0; sub(/.*\.reads\(/,"",lw); sub(/\).*/,"",lw); r=lw
        }
        /\.writes\(/{
          lw=$0; sub(/.*\.writes\(/,"",lw); sub(/\).*/,"",lw); wr=lw
        }
        $0 ~ ("pub fn[[:space:]]+"extr"\\("){ print w,r,wr; exit }
      ' "$DISPATCH")

      local code_w code_r code_wr
      read -r code_w code_r code_wr <<< "$code"
      code_w="${code_w//_/}"; code_r="${code_r//_/}"; code_wr="${code_wr//_/}"
      [[ -z "$code_w" ]] && code_w=0; [[ -z "$code_r" ]] && code_r=0; [[ -z "$code_wr" ]] && code_wr=0

      drift=$(awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{ (b==0)?print 99999:printf("%.1f", (a-b)/b*100) }')
      abs_drift=${drift#-}; drift_int=${abs_drift%%.*}

      summary_lines+=("$(printf "%-30s | reads %3s→%3s | writes %3s→%3s | weight %12s→%12s | drift %6s%%" \
        "$extr" "$code_r" "$meas_reads" "$code_wr" "$meas_writes" "$code_w" "$meas_ps" "$drift")")

      # gather mismatches
      if (( meas_reads != code_r ));   then new_reads[$extr]=$meas_reads;   fail=1; fi
      if (( meas_writes != code_wr )); then new_writes[$extr]=$meas_writes; fail=1; fi
      if (( drift_int > THRESHOLD ));  then new_weight[$extr]=$meas_ps;     fail=1; fi
    }

    # Iterate through benchmark output
    extr=""; meas_us=""; meas_reads=""; meas_writes=""
    while IFS= read -r line; do
      if [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]];      then finalize_extr; extr="${BASH_REMATCH[1]}"; fi
      if [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]];       then meas_us="${BASH_REMATCH[1]}"; fi
      if [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]];  then meas_reads="${BASH_REMATCH[1]}"; fi
      if [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]]; then meas_writes="${BASH_REMATCH[1]}"; fi
    done < "$TMP"
    finalize_extr

    echo; echo "Benchmark summary for '$pallet' (attempt #$attempt):"
    printf '  %s\n' "${summary_lines[@]}"

    if (( fail == 0 )); then
      echo "✅ Pallet '$pallet' is within ±${THRESHOLD}%."
      break
    fi

    # If failed & we still have attempts left → retry
    if (( attempt < MAX_RETRIES )); then
      echo "❌ Issues detected – retrying ($((attempt+1))/${MAX_RETRIES}) …"
      (( attempt++ ))
      continue
    fi

    ###########################################################################
    # All retries exhausted → optionally auto‑patch & commit
    ###########################################################################
    echo "❌ Pallet '$pallet' still failing after ${MAX_RETRIES} attempts."

    if [[ "$AUTO_COMMIT" != "1" ]]; then
      echo "AUTO_COMMIT_WEIGHTS disabled → exiting with error."
      exit 1
    fi

    echo "🛠  Auto‑patching dispatch file…"
    for e in "${!new_weight[@]}"; do
      [[ -n "${new_weight[$e]:-}" ]]   && patch_weight      "$e" "${new_weight[$e]}" "$DISPATCH"
      local r="${new_reads[$e]:-}";   w="${new_writes[$e]:-}"
      [[ -n "$r" || -n "$w" ]]        && patch_reads_writes "$e" "${r:-0}" "${w:-0}" "$DISPATCH"
    done
    PATCHED_FILES+=("$DISPATCH")

    echo "🔄  Re‑running benchmarks once more to verify the patch…"
    attempt=1        # reset attempt counter after patch
  done  # end retry loop

done  # end pallet loop

###############################################################################
# If we patched anything, commit & push
###############################################################################
if [[ "${#PATCHED_FILES[@]}" -gt 0 ]]; then
  echo; echo "📦  Committing updated weight files…"
  git_commit_and_push "chore: auto‑update benchmark weights"
  echo "✅ Auto‑patch committed & pushed."
fi

echo
echo "══════════════════════════════════════"
echo "All pallets validated ✔"
echo "══════════════════════════════════════"
