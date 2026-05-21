#!/usr/bin/env bash
# Pre-fetch all GitHub context the personas might want, so the Codex step
# itself does not need GH_TOKEN or network access. Outputs JSON / text files
# under $OUTPUT_DIR (default /tmp/ai-review-context). Run with `set -e` so any
# fetch failure aborts the workflow rather than producing a partial picture.

set -euo pipefail

: "${PR_NUMBER:?PR_NUMBER required}"
: "${REPO:?REPO required (e.g. opentensor/subtensor)}"
: "${GH_TOKEN:?GH_TOKEN required (used here only — NOT passed to Codex)}"
OUTPUT_DIR="${OUTPUT_DIR:-/tmp/ai-review-context}"

mkdir -p "$OUTPUT_DIR"
echo "Prefetching context to $OUTPUT_DIR"

# Retry wrapper for `gh` calls. GitHub's GraphQL endpoint in particular hands
# out occasional transient 502s that should not fail the whole review. Retries
# up to 3 times with exponential backoff. Captures stdout to a temp file so a
# partial failed response never ends up redirected into the caller's output.
gh_retry() {
  local max=3
  local delay=2
  local attempt=1
  local tmp
  tmp=$(mktemp)
  while (( attempt <= max )); do
    if "$@" > "$tmp"; then
      cat "$tmp"
      rm -f "$tmp"
      return 0
    fi
    if (( attempt < max )); then
      echo "::warning::gh call failed (attempt $attempt/$max); retrying in ${delay}s: $*" >&2
      sleep "$delay"
      delay=$(( delay * 2 ))
    fi
    attempt=$(( attempt + 1 ))
  done
  echo "::error::gh call failed after $max attempts: $*" >&2
  rm -f "$tmp"
  return 1
}

# Core PR metadata
gh_retry gh pr view "$PR_NUMBER" --repo "$REPO" \
  --json number,title,body,state,baseRefName,headRefName,headRefOid,baseRefOid,additions,deletions,changedFiles,author,createdAt,updatedAt,headRepository,headRepositoryOwner,labels,isDraft,mergeable \
  > "$OUTPUT_DIR/pr.json"

# Body separately for easy reading
jq -r '.body // ""' "$OUTPUT_DIR/pr.json" > "$OUTPUT_DIR/pr-body.md"

# Files changed (paths + per-file additions/deletions; full content lives in the diff)
gh_retry gh pr view "$PR_NUMBER" --repo "$REPO" --json files > "$OUTPUT_DIR/pr-files.json"

# Full unified diff
gh_retry gh pr diff "$PR_NUMBER" --repo "$REPO" > "$OUTPUT_DIR/pr-diff.patch"

# All PR comments (issue-style). `--paginate` alone writes one JSON array per
# page; `--slurp` wraps them as [[page1], [page2], ...]; we then flatten with
# external `jq 'add'` because `gh api` rejects `--slurp` together with `--jq`.
# pipefail (set at top of script) propagates gh failures through the pipe.
gh_retry gh api "repos/$REPO/issues/$PR_NUMBER/comments?per_page=100" \
  --paginate --slurp \
  | jq 'add' \
  > "$OUTPUT_DIR/pr-comments.json"

# Prior persona sticky comments — for rerun reconciliation. Both personas now
# share a single unified comment; each occupies a section delimited by
# <!-- ai-review:<persona>:begin --> / <!-- ai-review:<persona>:end --> markers.
# Extract each persona's section to its own file so the persona prompts can
# remain agnostic about the unified-comment structure.
jq -r '[.[] | select(.body | contains("<!-- ai-review:unified -->"))] | last | .body // ""' \
  "$OUTPUT_DIR/pr-comments.json" > "$OUTPUT_DIR/unified-comment.md"
for p in skeptic auditor; do
  awk -v begin="<!-- ai-review:$p:begin -->" -v end="<!-- ai-review:$p:end -->" '
    $0 ~ begin {flag=1; next}
    $0 ~ end   {flag=0}
    flag       {print}
  ' "$OUTPUT_DIR/unified-comment.md" > "$OUTPUT_DIR/prior-$p-comment.md"
done

# In-PR commits + their authors (committer != PR author is a real signal)
gh_retry gh pr view "$PR_NUMBER" --repo "$REPO" --json commits > "$OUTPUT_DIR/pr-commits.json"

# Author profile
AUTHOR=$(jq -r '.author.login' "$OUTPUT_DIR/pr.json")
echo "PR author: $AUTHOR"
gh_retry gh api "users/$AUTHOR" > "$OUTPUT_DIR/author-profile.json"

# Author contribution graph (rough activity signal). GraphQL endpoint is the
# most flake-prone — retry is especially important here.
gh_retry gh api graphql -f query='
  query($login: String!) {
    user(login: $login) {
      contributionsCollection {
        totalCommitContributions
        totalIssueContributions
        totalPullRequestContributions
        totalPullRequestReviewContributions
        restrictedContributionsCount
      }
    }
  }' -F login="$AUTHOR" > "$OUTPUT_DIR/author-contributions.json"

# Author's history in this repo
gh_retry gh pr list --author "$AUTHOR" --state all --repo "$REPO" --limit 100 \
  --json number,title,state,additions,deletions,createdAt,mergedAt \
  > "$OUTPUT_DIR/author-prs.json"

# Permission level (admin/write => nucleus; everything else => external).
# 404 (non-collaborator) is expected and not an error — bypass retry and
# default to "none" in that case.
if perm=$(gh api "repos/$REPO/collaborators/$AUTHOR/permission" --jq '.permission' 2>/dev/null); then
  echo "$perm" > "$OUTPUT_DIR/author-repo-permission.txt"
else
  echo "none" > "$OUTPUT_DIR/author-repo-permission.txt"
fi

# Other open PRs in the same repo — basis for the auditor's duplicate-work check
gh_retry gh pr list --repo "$REPO" --state open --limit 100 \
  --json number,title,author,baseRefName,headRefName,createdAt \
  > "$OUTPUT_DIR/open-prs.json"

# Cross-reference: which open PRs touch any of the same files as this PR?
THIS_PR_FILES=$(jq -c '.files | map(.path)' "$OUTPUT_DIR/pr-files.json")
echo "[]" > "$OUTPUT_DIR/overlapping-prs.json"
for other in $(jq -r '.[] | .number' "$OUTPUT_DIR/open-prs.json"); do
  if [[ "$other" == "$PR_NUMBER" ]]; then continue; fi
  other_files=$(gh_retry gh pr view "$other" --repo "$REPO" --json files \
    --jq '[.files[].path]' 2>/dev/null || echo "[]")
  overlap=$(jq -n --argjson a "$THIS_PR_FILES" --argjson b "$other_files" \
    '[$a[] | select(. as $f | $b | index($f))] | length')
  if [[ "$overlap" -gt 0 ]]; then
    jq --arg n "$other" --argjson o "$overlap" \
      '. += [{number: ($n | tonumber), overlapping_files: $o}]' \
      "$OUTPUT_DIR/overlapping-prs.json" \
      > "$OUTPUT_DIR/overlapping-prs.json.tmp"
    mv "$OUTPUT_DIR/overlapping-prs.json.tmp" "$OUTPUT_DIR/overlapping-prs.json"
  fi
done

echo "Pre-fetched files:"
ls -la "$OUTPUT_DIR"
