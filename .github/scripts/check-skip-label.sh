#!/usr/bin/env bash
# Check if the "skip-validate-benchmarks" label is present on a PR.
# Usage: check-skip-label.sh <PR_NUMBER>
# Exits 0 normally, or exits 0 after cancelling the workflow if label found.

set -euo pipefail

PR_NUMBER="${1:-}"
[[ -z "$PR_NUMBER" ]] && exit 0

REPO="${GITHUB_REPOSITORY:-}"
[[ -z "$REPO" ]] && exit 0

labels=$(gh pr view "$PR_NUMBER" --repo "$REPO" --json labels --jq '.labels[].name' 2>/dev/null || true)

if echo "$labels" | grep -q "skip-validate-benchmarks"; then
  echo "skip-validate-benchmarks label found — exiting."
  exit 1
fi
