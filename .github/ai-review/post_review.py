#!/usr/bin/env python3
"""
Parse a persona's Codex output (summary markdown + embedded inline-findings
JSON block), post a PR review with each finding as an inline review comment,
inject a markdown table linking to each inline comment into the summary, then
post or update the sticky summary comment.

Usage:
  GH_TOKEN=... python3 post_review.py \
      --persona skeptic \
      --pr 2668 \
      --repo opentensor/subtensor \
      --commit-sha <sha> \
      --output-file skeptic-output.md

Codex output format expected (see skeptic.md / auditor.md):

    <visible markdown summary, including the literal "<!-- inline-findings-table -->"
     placeholder where the findings table should be injected>

    <!-- inline-findings-json
    [
      {
        "path": "...",
        "line": 275,
        "side": "RIGHT",                       # optional, default RIGHT
        "start_line": 270,                     # optional, multi-line
        "severity": "HIGH",
        "title": "Short title",
        "body": "Markdown body (no suggestion fence)",
        "suggestion": "replacement text"       # optional
      }
    ]
    end inline-findings-json -->

    <!-- ai-review:<persona> -->
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from typing import Any


SEVERITY_RANK = {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}


def gh_api(method: str, path: str, body: dict | None = None) -> dict:
    """Thin wrapper around `gh api` so we don't need the requests library."""
    cmd = ["gh", "api", "-X", method, path]
    if body is not None:
        cmd += ["--input", "-"]
    proc = subprocess.run(
        cmd,
        input=json.dumps(body) if body is not None else None,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"gh api {method} {path} failed:\n  stdout={proc.stdout}\n  stderr={proc.stderr}"
        )
    return json.loads(proc.stdout) if proc.stdout.strip() else {}


def split_output(text: str) -> tuple[str, list[dict]]:
    """Split Codex output into visible summary + parsed findings list."""
    pattern = re.compile(
        r"<!--\s*inline-findings-json\s*(.*?)\s*end inline-findings-json\s*-->",
        re.DOTALL,
    )
    match = pattern.search(text)
    findings: list[dict] = []
    if match:
        raw = match.group(1).strip()
        try:
            parsed = json.loads(raw)
            if isinstance(parsed, list):
                findings = parsed
            else:
                print(f"::warning::inline-findings-json was not a list: {type(parsed)}",
                      file=sys.stderr)
        except json.JSONDecodeError as e:
            print(f"::warning::failed to parse inline-findings-json: {e}", file=sys.stderr)
        summary = pattern.sub("", text).strip() + "\n"
    else:
        summary = text
    return summary, findings


def render_comment_body(finding: dict) -> str:
    """Build the comment body posted to GitHub, with optional suggestion fence."""
    severity = finding.get("severity", "INFO").upper()
    title = finding.get("title", "").strip()
    body = finding.get("body", "").strip()
    suggestion = finding.get("suggestion")
    parts = [f"**[{severity}] {title}**".strip(), "", body]
    if suggestion is not None and suggestion != "":
        parts += ["", "```suggestion", suggestion.rstrip("\n"), "```"]
    return "\n".join(parts).strip() + "\n"


def build_review_comments(findings: list[dict]) -> list[dict]:
    """Translate our finding schema to GitHub's review-comment schema."""
    result: list[dict] = []
    for f in findings:
        if not f.get("path") or not f.get("line"):
            print(f"::warning::skipping finding without path+line: {f}", file=sys.stderr)
            continue
        side = (f.get("side") or "RIGHT").upper()
        comment: dict = {
            "path": f["path"],
            "line": int(f["line"]),
            "side": side,
            "body": render_comment_body(f),
        }
        if f.get("start_line") is not None:
            comment["start_line"] = int(f["start_line"])
            comment["start_side"] = (f.get("start_side") or side).upper()
        result.append(comment)
    return result


def post_review(
    repo: str, pr: int, commit_sha: str, comments: list[dict]
) -> tuple[int, list[dict]]:
    """Create a PR review with the given inline comments; return (review_id, posted_comments)."""
    if not comments:
        return (0, [])
    review = gh_api(
        "POST",
        f"repos/{repo}/pulls/{pr}/reviews",
        {
            "commit_id": commit_sha,
            "event": "COMMENT",
            "body": "AI review — see inline comments and the sticky summary.",
            "comments": comments,
        },
    )
    review_id = int(review.get("id", 0))
    posted = gh_api(
        "GET",
        f"repos/{repo}/pulls/{pr}/reviews/{review_id}/comments?per_page=100",
    )
    return (review_id, posted if isinstance(posted, list) else [])


def build_findings_table(findings: list[dict], posted: list[dict]) -> str:
    """Render a markdown table with links to each inline comment."""
    if not findings:
        return "_No inline findings._"
    # GitHub returns review comments roughly in file/line order; pair by path+line.
    url_by_loc: dict[tuple[str, int], str] = {}
    for c in posted:
        key = (c.get("path", ""), int(c.get("line") or c.get("original_line") or 0))
        url_by_loc[key] = c.get("html_url", "")
    rows = ["| Sev | File | Finding | |", "| --- | --- | --- | --- |"]
    ordered = sorted(
        findings,
        key=lambda f: (
            SEVERITY_RANK.get(str(f.get("severity", "")).upper(), 99),
            f.get("path", ""),
            int(f.get("line") or 0),
        ),
    )
    for f in ordered:
        sev = str(f.get("severity", "")).upper() or "—"
        path = f.get("path", "")
        line = f.get("line") or "?"
        title = f.get("title", "").strip().replace("|", "\\|")
        url = url_by_loc.get((path, int(line) if str(line).isdigit() else 0))
        link = f"[inline]({url})" if url else "_(off-diff)_"
        rows.append(f"| **{sev}** | `{path}:{line}` | {title} | {link} |")
    return "\n".join(rows)


def upsert_sticky_comment(repo: str, pr: int, marker: str, body: str) -> None:
    """Edit existing sticky comment matched by marker; else create new."""
    comments = gh_api("GET", f"repos/{repo}/issues/{pr}/comments?per_page=100")
    existing_id = None
    for c in comments:
        if marker in c.get("body", ""):
            existing_id = c.get("id")  # keep walking — `existing_id` ends as the last match
    if existing_id:
        gh_api("PATCH", f"repos/{repo}/issues/comments/{existing_id}", {"body": body})
    else:
        gh_api("POST", f"repos/{repo}/issues/{pr}/comments", {"body": body})


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--persona", required=True, choices=["skeptic", "auditor"])
    p.add_argument("--pr", required=True, type=int)
    p.add_argument("--repo", required=True)
    p.add_argument("--commit-sha", required=True)
    p.add_argument("--output-file", required=True)
    args = p.parse_args()

    if not os.environ.get("GH_TOKEN"):
        print("::error::GH_TOKEN must be set", file=sys.stderr)
        return 1

    with open(args.output_file) as f:
        raw = f.read()
    if not raw.strip():
        print("::error::Codex output file is empty", file=sys.stderr)
        return 1

    summary, findings = split_output(raw)
    marker = f"<!-- ai-review:{args.persona} -->"
    if marker not in summary:
        summary = summary.rstrip() + "\n\n" + marker + "\n"

    inline_comments = build_review_comments(findings)
    posted: list[dict] = []
    if inline_comments:
        try:
            _, posted = post_review(args.repo, args.pr, args.commit_sha, inline_comments)
            print(f"Posted {len(posted)} inline comments.", file=sys.stderr)
        except RuntimeError as e:
            # If the review API rejects (e.g. line outside diff), fall back to
            # listing in the summary without inline links.
            print(f"::warning::review post failed; falling back to summary-only: {e}",
                  file=sys.stderr)

    table = build_findings_table(findings, posted)
    summary = summary.replace("<!-- inline-findings-table -->", table)

    upsert_sticky_comment(args.repo, args.pr, marker, summary)
    print("Updated sticky comment.", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
