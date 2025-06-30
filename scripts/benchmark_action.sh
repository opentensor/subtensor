#!/usr/bin/env bash
###############################################################################
# benchmark_action.sh
#
# 1. Benchmarks every pallet in PALLET_LIST.
# 2. Compares measured vs. declared weight / reads / writes.
# 3. Each pallet → max 3 attempts. After 3 failures:
#      • Patch literals once
#      • Commit & push when AUTO_COMMIT_WEIGHTS=1
#      • Move on to the next pallet (no re‑benchmark)
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
log_warn()     { echo "⚠️  $*"; }

###############################################################################
# Patch helpers (support attribute‑above *or* inline style)
###############################################################################
patch_weight() {      # $1 fn  $2 new_weight  $3 file
  local before after; before=$(sha1sum "$3" | cut -d' ' -f1)
  FN="$1" NEWV="$2" perl -0777 -i -pe '
    my $n = $ENV{NEWV};                                 # raw digits
    my $hit=0;
    # inline:   pub fn … Weight::from_parts(
    $hit += s|(pub\s+fn\s+\Q$ENV{FN}\E\s*[^{}]*?Weight::from_parts\(\s*)[0-9A-Za-z_]+|$1$n|s;
    # attribute‑above: #[pallet::weight(Weight::from_parts( … )]  ⟶ same number
    $hit += s|(\#\s*\[pallet::weight[^\]]*?Weight::from_parts\(\s*)[0-9A-Za-z_]+(?=[^\]]*?\]\s*pub\s+fn\s+\Q$ENV{FN}\E\b)|$1$n|s;
    END{ exit $hit ? 0 : 1 }
  ' "$3" || log_warn "patch_weight: no substitution for $1"
  after=$(sha1sum "$3" | cut -d' ' -f1); [[ "$before" != "$after" ]]
}

patch_reads_writes() {  # $1 fn  $2 new_r  $3 new_w  $4 file
  local before after; before=$(sha1sum "$4" | cut -d' ' -f1)
  FN="$1" NEWR="$2" NEWW="$3" perl -0777 -i -pe '
    my ($r,$w)=("$ENV{NEWR}_u64","$ENV{NEWW}_u64");
    my $h=0;
    # inline reads_writes(...)
    $h += s|(pub\s+fn\s+\Q$ENV{FN}\E\s*[^{}]*?reads_writes\(\s*)([^,]+)(\s*,\s*)([^)]+)|$1$r$3$w|s;
    # attribute reads_writes(...)
    $h += s|(\#\s*\[pallet::weight[^\]]*?reads_writes\(\s*)([^,]+)(\s*,\s*)([^)]+)(?=[^\]]*?\]\s*pub\s+fn\s+\Q$ENV{FN}\E\b)|$1$r$3$w|s;
    # inline .reads() / .writes()
    $h += s|(pub\s+fn\s+\Q$ENV{FN}\E\s*[^{}]*?\.reads\(\s*)([^)]+)|$1$r|s;
    $h += s|(pub\s+fn\s+\Q$ENV{FN}\E\s*[^{}]*?\.writes\(\s*)([^)]+)|$1$w|s;
    # attribute .reads() / .writes()
    $h += s|(\#\s*\[pallet::weight[^\]]*?\.reads\(\s*)([^)]+)(?=[^\]]*?\]\s*pub\s+fn\s+\Q$ENV{FN}\E\b)|$1$r|s;
    $h += s|(\#\s*\[pallet::weight[^\]]*?\.writes\(\s*)([^)]+)(?=[^\]]*?\]\s*pub\s+fn\s+\Q$ENV{FN}\E\b)|$1$w|s;
    END{ exit $h ? 0 : 1 }
  ' "$4" || log_warn "patch_reads_writes: no substitution for $1"
  after=$(sha1sum "$4" | cut -d' ' -f1); [[ "$before" != "$after" ]]
}

git_commit_and_push() {
  local msg="$1"
  local branch; branch=$(git symbolic-ref --quiet --short HEAD || true)
  [[ -z "$branch" ]] && die "Not on a branch – cannot push"

  git config user.name  "github-actions[bot]"
  git config user.email "github-actions[bot]@users.noreply.github.com"
  git add "${PATCHED_FILES[@]}" || true

  if git diff --cached --quiet; then
    echo "ℹ️  No staged changes after patching."; git status --short; return
  fi

  echo "==== diff preview ===="; git diff --cached --stat
  # head can SIGPIPE diff → ignore failure
  git diff --cached | head -n 40 || true
  echo "======================"

  git commit -m "$msg"
  git push origin "HEAD:$branch" || die "Push to '${branch}' failed."
}

################################################################################
# Build runtime once
################################################################################
echo "Building runtime‑benchmarks…"
cargo build --profile production -p node-subtensor --features runtime-benchmarks

echo -e "\n─────────────────────────────────────────────"
echo " Will benchmark pallets: ${PALLET_LIST[*]}"
echo "─────────────────────────────────────────────"

PATCHED_FILES=()

################################################################################
# Benchmark loop
################################################################################
for pallet in "${PALLET_LIST[@]}"; do
  DISPATCH="$SCRIPT_DIR/${DISPATCH_PATHS[$pallet]}"
  [[ -f "$DISPATCH" ]] || die "dispatch file missing: $DISPATCH"

  attempt=1
  while (( attempt <= MAX_RETRIES )); do
    printf "\n════ Benchmarking '%s' (attempt %d/%d) ════\n" "$pallet" "$attempt" "$MAX_RETRIES"

    TMP=$(mktemp); trap 'rm -f "$TMP"' EXIT
    ./target/production/node-subtensor benchmark pallet \
      --runtime "$RUNTIME_WASM" --genesis-builder=runtime \
      --genesis-builder-preset=benchmark --wasm-execution=compiled \
      --pallet "pallet_${pallet}" --extrinsic "*" --steps 50 --repeat 5 \
      | tee "$TMP"

    # ───── parse output ─────
    declare -A new_weight new_reads new_writes
    summary=(); failures=(); fail=0
    extr=""; meas_us=0; meas_r=0; meas_w=0

    flush() {
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

      code_w=$(dec "${code_w:-0}"); code_r=$(dec "${code_r:-0}"); code_wr=$(dec "${code_wr:-0}")

      local drift; drift=$([[ "$code_w" -eq 0 ]] && echo 99999 || awk -v a="$meas_ps" -v b="$code_w" 'BEGIN{printf("%.1f", (a-b)/b*100)}')
      local abs=${drift#-}; local dint=${abs%%.*}

      summary+=("$(printf "%-35s | reads %4s → %4s | writes %4s → %4s | weight %12s → %12s | drift %6s%%" \
                 "$extr" "$code_r" "$meas_r" "$code_wr" "$meas_w" "$code_w" "$meas_ps" "$drift")")

      if (( meas_r != code_r ));     then failures+=("[$extr] reads mismatch");   new_reads[$extr]=$meas_r;  fail=1; fi
      if (( meas_w != code_wr ));    then failures+=("[$extr] writes mismatch");  new_writes[$extr]=$meas_w; fail=1; fi
      if (( dint > THRESHOLD ));     then failures+=("[$extr] weight drift ${drift}%"); new_weight[$extr]=$meas_ps; fail=1; fi
    }

    while IFS= read -r line; do
      [[ $line =~ Extrinsic:\ \"([_[:alnum:]]+)\" ]] && { flush; extr="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Time\ ~=\ *([0-9]+(\.[0-9]+)?) ]]  && { meas_us="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Reads[[:space:]]*=[[:space:]]*([0-9]+) ]]  && { meas_r="${BASH_REMATCH[1]}"; continue; }
      [[ $line =~ Writes[[:space:]]*=[[:space:]]*([0-9]+) ]] && { meas_w="${BASH_REMATCH[1]}"; continue; }
    done < "$TMP"; flush

    echo; printf '  %s\n' "${summary[@]}"
    (( fail == 0 )) && { echo "✅ '$pallet' within tolerance."; break; }

    printf '  ❌ %s\n' "${failures[@]}"
    (( attempt < MAX_RETRIES )) && { echo "→ Retrying …"; (( attempt++ )); continue; }

    # --- Patch after final failure ---
    echo "❌ '$pallet' still failing; patching …"
    [[ "$AUTO_COMMIT" != "1" ]] && die "AUTO_COMMIT_WEIGHTS disabled."

    changed=0
    for fn in "${!new_weight[@]}"; do
      patch_weight "$fn" "${new_weight[$fn]}" "$DISPATCH" && changed=1
      r="${new_reads[$fn]:-}"; w="${new_writes[$fn]:-}"
      [[ -n "$r" || -n "$w" ]] && patch_reads_writes "$fn" "${r:-0}" "${w:-0}" "$DISPATCH" && changed=1
    done

    if (( changed )); then PATCHED_FILES+=("$DISPATCH"); echo "✅ Patched '$pallet' file.";
    else                   echo "⚠️  No modifications applied for '$pallet'."; fi
    break
  done
done

################################################################################
# Commit & push patches (if any)
################################################################################
if (( ${#PATCHED_FILES[@]} )); then
  echo -e "\n📦  Committing patched files …"; git_commit_and_push "chore: auto‑update benchmark weights"
fi

echo -e "\n══════════════════════════════════════"
echo "All pallets processed ✔"
echo "══════════════════════════════════════"
