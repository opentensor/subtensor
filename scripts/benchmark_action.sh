#!/usr/bin/env bash
###############################################################################
# benchmark_action.sh – super‑verbose, auto‑patching benchmark validator
#
# • Benchmarks every pallet in PALLET_LIST.
# • Compares measured vs. declared weight / read / write values.
# • Retries             : MAX_RETRIES   (default 3).
# • Auto‑patch & commit : when AUTO_COMMIT_WEIGHTS == 1 (set in the workflow).
# • Logs                : extremely chatty – every major step is echoed, and
#                         Bash's `set -x` prints each command with its arguments.
###############################################################################

set -euo pipefail
IFS=$'\n\t'
export LC_ALL=C

###############################################################################
# ─────────────────────────────  CONFIGURATION  ───────────────────────────────
###############################################################################
PALLET_LIST=(subtensor admin_utils commitments drand)

declare -A DISPATCH_PATHS=(
  [subtensor]="../pallets/subtensor/src/macros/dispatches.rs"
  [admin_utils]="../pallets/admin-utils/src/lib.rs"
  [commitments]="../pallets/commitments/src/lib.rs"
  [drand]="../pallets/drand/src/lib.rs"
  [swap]="../pallets/swap/src/pallet/mod.rs"
)

THRESHOLD=15
MAX_RETRIES=3
AUTO_COMMIT="${AUTO_COMMIT_WEIGHTS:-0}"

###############################################################################
# ─────────────────────────────  INITIAL BUILD  ───────────────────────────────
###############################################################################
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_WASM="$SCRIPT_DIR/../target/production/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm"

echo "::group::🔨 Build runtime (once)"
cargo build --profile production -p node-subtensor --features runtime-benchmarks
echo "::endgroup::"

echo
echo "──────────────────────────────────────────────────────────────────────────────"
echo " Will benchmark pallets: ${PALLET_LIST[*]}"
echo "──────────────────────────────────────────────────────────────────────────────"

###############################################################################
# ──────────────────────────────  UTILITIES  ──────────────────────────────────
###############################################################################
set -x   # <‑‑‑  FULL SHELL TRACE FROM HERE ↓

# Log helper – prints and sends to GitHub fold‑marker
log() { echo "::notice::$*"; }

# Strip all non‑digit characters (underscores, u64 suffixes, etc.).
sanitize_num() {
  local v="${1//_/}"
  v="${v%%[^0-9]*}"
  [[ -z "$v" ]] && v=0
  echo "$v"
}

#   ────────  source‑patch helpers (perl)  ────────
patch_weight() {
  local extr="$1" new_weight="$2" file="$3"
  log "patch_weight: extr=$extr new_weight=$new_weight file=$file"
  EXTR="$extr" NEW_WEIGHT="$new_weight" perl -0777 -i -pe '
    my ($ex, $nw) = @ENV{qw/EXTR NEW_WEIGHT/};
    s#(pub\s+fn\s+\Q$ex\E\s*\([^)]*?\).*?Weight::from_parts\(\s*)\d[\d_]*#$1$nw#s;
  ' "$file"
}

patch_reads_writes() {
  local extr="$1" new_r="$2" new_w="$3" file="$4"
  log "patch_reads_writes: extr=$extr R=$new_r W=$new_w file=$file"
  EXTR="$extr" NEW_R="$new_r" NEW_W="$new_w" perl -0777 -i -pe '
    my ($ex,$nr,$nw) = @ENV{qw/EXTR NEW_R NEW_W/};
    s#(pub\s+fn\s+\Q$ex\E\s*\([^)]*?\).*?reads_writes\(\s*)\d+\s*,\s*\d+\s*#${1}${nr}, ${nw}#s;
    s#(pub\s+fn\s+\Q$ex\E\s*\([^)]*?\).*?\.reads\(\s*)\d+\s*#${1}${nr}#s;
    s#(pub\s+fn\s+\Q$ex\E\s*\([^)]*?\).*?\.writes\(\s*)\d+\s*#${1}${nw}#s;
  ' "$file"
}

git_commit_and_push() {
  local msg="$1"
  [[ "${#PATCHED_FILES[@]}" -eq 0 ]] && return
  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add "${PATCHED_FILES[@]}"
  git commit -m "$msg"
  git push
}

###############################################################################
# ────────────────────────────  BENCHMARK LOOP  ───────────────────────────────
###############################################################################
PATCHED_FILES=()

for pallet in "${PALLET_LIST[@]}"; do
  DISPATCH_REL="${DISPATCH_PATHS[$pallet]:-}"
  [[ -n "$DISPATCH_REL" ]] || { log "dispatch path missing for $pallet"; exit 1; }
  DISPATCH="$SCRIPT_DIR/$DISPATCH_REL"
  [[ -f "$DISPATCH" ]] || { log "dispatch file not found: $DISPATCH"; exit 1; }

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    log "▶️  pallet=$pallet attempt=$attempt"
    TMP=$(mktemp)
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

    # ───── parse benchmark output ─────
    declare -A new_weight=() new_reads=() new_writes=()
    summary_lines=(); failures=(); fail=0
    extr=""; meas_us=""; meas_reads=""; meas_writes=""

    finalize_extr() {
      [[ -z "$extr" ]] && return
      local meas_ps
      meas_ps=$(awk -v x="$meas_us" 'BEGIN{printf("%.0f", x*1000000)}')

      # Pull code‑side values
      local code
      code=$(awk -v extr="$extr" '
        /^\s*#\[pallet::call_index\(/ { next }
        /Weight::from_parts/{
          lw=$0; sub(/.*Weight::from_parts\(/,"",lw);
          sub(/[^0-9_].*/,"",lw); w=lw
        }
        /reads_writes\(/{
          lw=$0; sub(/.*reads_writes\(/,"",lw); sub(/\).*/,"",lw);
          split(lw,io,","); gsub(/[ \t]/,"",io[1]); gsub(/[ \t]/,"",io[2]); r=io[1]; wr=io[2]
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
      read -r code_w code_r code_wr <<<"$code"

      # Sanitize everything
      local cw cr cwr mr mw
      cw=$(sanitize_num "$code_w")
      cr=$(sanitize_num "$code_r")
      cwr=$(sanitize_num "$code_wr")
      mr=$(sanitize_num "$meas_reads")
      mw=$(sanitize_num "$meas_writes")

      local drift
      if [[ "$cw" == 0 ]]; then
        drift=99999
      else
        drift=$(awk -v a="$meas_ps" -v b="$cw" 'BEGIN{printf("%.1f", (a-b)/b*100)}')
      fi
      local abs=${drift#-}; local dint=${abs%%.*}

      summary_lines+=("$(printf "%-35s | reads %3s → %3s | writes %3s → %3s | weight %12s → %12s | drift %6s%%" \
        "$extr" "$cr" "$mr" "$cwr" "$mw" "$cw" "$meas_ps" "$drift")")

      if (( mr != cr ));   then new_reads[$extr]=$mr;   fail=1; fi
      if (( mw != cwr ));  then new_writes[$extr]=$mw;  fail=1; fi
      if (( dint > THRESHOLD )); then new_weight[$extr]=$meas_ps; fail=1; fi
    }

    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([[:alnum:]_]+)\" ]] && { finalize_extr; extr="${BASH_REMATCH[1]}"; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]] && { meas_us="${BASH_REMATCH[1]}"; }
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && { meas_reads="${BASH_REMATCH[1]}"; }
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && { meas_writes="${BASH_REMATCH[1]}"; }
    done < "$TMP"
    finalize_extr

    echo "──────────────── summary (pallet=$pallet attempt=$attempt) ────────────────"
    printf '  %s\n' "${summary_lines[@]}"
    echo "────────────────────────────────────────────────────────────────────────────"

    if (( fail == 0 )); then
      log "✅ pallet=$pallet attempt=$attempt status=PASS"
      break
    fi

    if (( attempt < MAX_RETRIES )); then
      log "🔁 pallet=$pallet attempt=$attempt status=FAIL – retrying"
      (( attempt++ ))
      continue
    fi

    # ───── all retries exhausted – maybe auto‑patch ─────
    log "❌ pallet=$pallet exhausted retries; preparing auto‑patch"
    if [[ "$AUTO_COMMIT" != "1" ]]; then
      log "AUTO_COMMIT_WEIGHTS=0 → exiting with error"
      exit 1
    fi

    for e in "${!new_weight[@]}"; do
      [[ -n "${new_weight[$e]:-}" ]] && patch_weight "$e" "${new_weight[$e]}" "$DISPATCH"
      local r="${new_reads[$e]:-}" w="${new_writes[$e]:-}"
      if [[ -n "$r" || -n "$w" ]]; then
        patch_reads_writes "$e" "${r:-0}" "${w:-0}" "$DISPATCH"
      fi
    done
    PATCHED_FILES+=("$DISPATCH")
    log "📄 patched $DISPATCH – will re‑benchmark"
    attempt=1
  done  # end retry loop
done    # pallet loop

###############################################################################
# ─────────────────────────────  COMMIT SECTION  ──────────────────────────────
###############################################################################
if [[ "${#PATCHED_FILES[@]}" -gt 0 ]]; then
  log "📝 committing patched files"
  git_commit_and_push "chore: auto‑update benchmark weights"
  log "🚀 patches pushed successfully"
fi

log "🎉 all pallets validated"
