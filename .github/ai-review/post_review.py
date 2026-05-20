#!/usr/bin/env python3
"""
Post a persona's review to a PR.

Input: a JSON document produced by Codex against `codex-output-schema.json`.
Behaviour:

  1. Post a fresh sticky comment with the new verdict and a findings table.
     Each finding gets a stable ID = sha1(path:line:title)[:8], embedded in
     the row as `<!-- fid:xxxxxxxx -->` so future runs can match.
  2. Open a PR review with each inline finding as a review comment (with
     ```suggestion blocks where applicable, giving the one-click 'Apply'
     button on GitHub).
  3. If a previous sticky comment exists (matched by the persona marker),
     EDIT it in place to:
       - prepend a 'Superseded by <new comment URL>' header
       - render its findings table with strikethrough on every prior finding,
         annotated with status from the new comment's prior_reconciliation:
           addressed         -> ✅ Addressed
           no_longer_applies -> ⏭️ No longer applies
           not_addressed     -> ➡️ Carried forward to <new comment URL>
       - remove the old prior-reconciliation and conclusion sections
     The OLD comment thus becomes a compact historical record; the NEW comment
     is the live status.

Usage:
  GH_TOKEN=... python3 post_review.py \
    --persona skeptic --pr 2668 --repo opentensor/subtensor \
    --commit-sha <sha> --input-file skeptic-output.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from typing import Any


SEVERITY_RANK = {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}


def gh_api(method: str, path: str, body: dict | None = None) -> dict | list:
    """Call gh api; raise on non-zero. Returns parsed JSON, or {} for empty."""
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


def finding_id(path: str, line: int | str, title: str) -> str:
    """Stable 8-char ID derived from a finding's location + title."""
    key = f"{path}:{line}:{title.strip().lower()}"
    return hashlib.sha1(key.encode()).hexdigest()[:8]


def render_inline_comment_body(f: dict) -> str:
    """Build the markdown body of an inline review comment (incl. fid marker + suggestion fence)."""
    severity = f["severity"]
    title = f["title"].strip()
    body = (f.get("body_markdown") or "").strip()
    fid = finding_id(f["path"], f["line"], title)
    parts = [
        f"**[{severity}] {title}**",
        "",
        body,
    ]
    suggestion = f.get("suggestion")
    if suggestion is not None and suggestion != "":
        parts += ["", "```suggestion", suggestion.rstrip("\n"), "```"]
    parts += ["", f"<!-- fid:{fid} -->"]
    return "\n".join(parts).strip() + "\n"


def to_review_comment(f: dict) -> dict:
    """Translate our inline-finding schema to GitHub's review-comment schema."""
    side = (f.get("side") or "RIGHT").upper()
    c: dict[str, Any] = {
        "path": f["path"],
        "line": int(f["line"]),
        "side": side,
        "body": render_inline_comment_body(f),
    }
    if f.get("start_line") is not None:
        c["start_line"] = int(f["start_line"])
        c["start_side"] = side
    return c


def post_review(
    repo: str, pr: int, commit_sha: str, comments: list[dict]
) -> tuple[int, list[dict]]:
    if not comments:
        return (0, [])
    review = gh_api(
        "POST",
        f"repos/{repo}/pulls/{pr}/reviews",
        {
            "commit_id": commit_sha,
            "event": "COMMENT",
            "body": "AI review — see the sticky summary comment for the verdict and the inline comments below for specific findings.",
            "comments": comments,
        },
    )
    review_id = int(review.get("id", 0))
    posted = gh_api("GET", f"repos/{repo}/pulls/{pr}/reviews/{review_id}/comments?per_page=100")
    return (review_id, posted if isinstance(posted, list) else [])


def render_findings_table(
    inline: list[dict], off_diff: list[dict], inline_urls: dict[str, str]
) -> str:
    """Build the live findings table for the new sticky comment."""
    rows: list[tuple[str, str, str, str, str, str]] = []  # (sev_rank, sev, file, title, link, fid)
    for f in inline:
        fid = finding_id(f["path"], f["line"], f["title"])
        sev = f["severity"].upper()
        link = inline_urls.get(fid, "")
        link_md = f"[inline]({link})" if link else "_(post-failed)_"
        rows.append(
            (
                str(SEVERITY_RANK.get(sev, 99)),
                sev,
                f"`{f['path']}:{f['line']}`",
                f["title"].strip().replace("|", "\\|"),
                link_md,
                fid,
            )
        )
    for f in off_diff:
        title = f["title"].strip().replace("|", "\\|")
        sev = f["severity"].upper()
        loc = f.get("approximate_location") or "—"
        fid = finding_id(loc, 0, title)
        rows.append(
            (str(SEVERITY_RANK.get(sev, 99)), sev, f"_{loc}_", title, "_(off-diff)_", fid),
        )
    if not rows:
        return "_No findings._"
    rows.sort()
    lines = ["| Sev | File | Finding |  |", "| --- | --- | --- | --- |"]
    for _, sev, fileloc, title, link, fid in rows:
        lines.append(f"| **{sev}** | {fileloc} | {title} <!-- fid:{fid} --> | {link} |")
    return "\n".join(lines)


def parse_prior_findings(prior_body: str) -> list[dict]:
    """
    Parse rows out of the prior sticky comment's findings table.
    Returns list of {fid, sev, fileloc, title, link_md}.
    """
    rows: list[dict] = []
    pattern = re.compile(
        r"^\|\s*\*\*(?P<sev>[A-Z]+)\*\*\s*\|\s*(?P<fileloc>[^|]+?)\s*\|\s*(?P<title>.+?)<!--\s*fid:(?P<fid>[A-Za-z0-9]+)\s*-->\s*\|\s*(?P<link>[^|]+?)\s*\|",
        re.MULTILINE,
    )
    for m in pattern.finditer(prior_body):
        rows.append(
            {
                "fid": m.group("fid"),
                "sev": m.group("sev"),
                "fileloc": m.group("fileloc").strip(),
                "title": m.group("title").strip().rstrip(),
                "link_md": m.group("link").strip(),
            }
        )
    return rows


def render_superseded_body(
    prior_body: str,
    prior_rows: list[dict],
    reconciliation: list[dict],
    new_comment_url: str,
    persona: str,
) -> str:
    """Build the replacement body for the old sticky comment."""
    status_by_fid: dict[str, dict] = {}
    for r in reconciliation:
        if r.get("prior_finding_id"):
            status_by_fid[r["prior_finding_id"]] = r

    icon = {
        "addressed": "✅ Addressed",
        "no_longer_applies": "⏭️ No longer applies",
        "not_addressed": f"➡️ Carried forward",
    }

    table_lines = ["| ~~Sev~~ | ~~File~~ | ~~Finding~~ | Status |", "| --- | --- | --- | --- |"]
    for r in prior_rows:
        rec = status_by_fid.get(r["fid"])
        if rec is None:
            status_md = "❔ Status unknown in current run"
        else:
            base = icon.get(rec["status"], rec["status"])
            if rec["status"] == "not_addressed":
                base += f" — see [new comment]({new_comment_url})"
            note = rec.get("note_markdown")
            if note:
                base += f"<br/>_{note.strip()}_"
            status_md = base
        table_lines.append(
            f"| ~~**{r['sev']}**~~ | ~~{r['fileloc']}~~ | ~~{r['title']}~~ | {status_md} |"
        )

    # Try to keep the original verdict line for historical legibility.
    first_line = prior_body.splitlines()[0] if prior_body else ""
    original_verdict = (
        f"~~{first_line}~~" if first_line.startswith("VERDICT:") else ""
    )

    parts = [
        f"> ⚠️ **Superseded by [a newer review comment]({new_comment_url}).** This is a historical snapshot.",
        "",
    ]
    if original_verdict:
        parts += [original_verdict, ""]
    parts += [
        "## Findings (status as of supersession)",
        "",
        "\n".join(table_lines),
        "",
        f"<!-- ai-review:{persona} -->",
        "",
        f"<!-- ai-review:{persona}:superseded -->",
    ]
    return "\n".join(parts).strip() + "\n"


def render_new_sticky(
    persona: str,
    verdict: str,
    scrutiny_note: str,
    summary_markdown: str,
    conclusion_markdown: str,
    findings_table: str,
    off_diff: list[dict],
    reconciliation: list[dict],
    prior_url: str | None,
) -> str:
    """Build the body of the new sticky comment."""
    parts = [
        f"VERDICT: {verdict}",
        "",
        scrutiny_note.strip(),
    ]
    if prior_url:
        parts += ["", f"_Supersedes [previous review]({prior_url})._"]
    if summary_markdown.strip():
        parts += ["", summary_markdown.strip()]
    parts += ["", "## Findings", "", findings_table.strip()]
    if off_diff:
        parts += ["", "## Other findings", ""]
        for f in off_diff:
            sev = f["severity"].upper()
            title = f["title"].strip()
            body = f.get("body_markdown", "").strip()
            loc = f.get("approximate_location")
            loc_md = f" _({loc})_" if loc else ""
            parts.append(f"- **[{sev}]** {title}{loc_md} — {body}")
    if reconciliation:
        parts += ["", "## Prior-comment reconciliation", ""]
        for r in reconciliation:
            status = r["status"].replace("_", " ")
            note = r.get("note_markdown")
            line = f"- `{r['prior_finding_id']}`: **{status}**"
            if note:
                line += f" — {note.strip()}"
            parts.append(line)
    parts += ["", "## Conclusion", "", conclusion_markdown.strip(),
              "", f"<!-- ai-review:{persona} -->"]
    return "\n".join(parts).strip() + "\n"


def _post_error_sticky(repo: str, pr: int, persona: str, message: str, raw: str) -> None:
    """
    Post a sticky comment surfacing a Codex-output failure to the PR thread.
    On the next run, this becomes the agent's prior comment, giving it
    direct feedback to self-correct.
    """
    marker = f"<!-- ai-review:{persona} -->"
    # Truncate raw output so the comment isn't enormous.
    raw_trim = raw if len(raw) <= 4000 else raw[:2000] + "\n\n[... truncated ...]\n\n" + raw[-2000:]
    body = (
        f"VERDICT: ERROR\n\n"
        f"⚠️ **Codex output failed validation.** {message}\n\n"
        f"<details><summary>Raw model output ({len(raw)} chars)</summary>\n\n"
        f"```\n{raw_trim}\n```\n\n</details>\n\n"
        f"{marker}\n"
    )
    try:
        # Mark any prior sticky as superseded so the chain remains coherent.
        prior_id, prior_body, _prior_url = find_prior_live_sticky(repo, pr, persona)
        new = post_new_sticky(repo, pr, body)
        if prior_id is not None:
            edit_comment(
                repo, prior_id,
                f"> ⚠️ **Superseded by [an error report]({new.get('html_url','')}).**\n\n"
                f"{prior_body}\n\n<!-- ai-review:{persona}:superseded -->\n",
            )
    except Exception as e:  # last-resort: surface in logs
        print(f"::error::Failed to post error sticky: {e}", file=sys.stderr)
        print(f"::error::Original Codex output ({len(raw)} chars):", file=sys.stderr)
        print(raw_trim, file=sys.stderr)


def find_prior_live_sticky(
    repo: str, pr: int, persona: str
) -> tuple[int | None, str, str]:
    """
    Find the most recent sticky comment for this persona that has NOT yet been
    marked superseded. Returns (comment_id, body, html_url) or (None, '', '').
    """
    marker_live = f"<!-- ai-review:{persona} -->"
    marker_dead = f"<!-- ai-review:{persona}:superseded -->"
    comments = gh_api("GET", f"repos/{repo}/issues/{pr}/comments?per_page=100")
    if not isinstance(comments, list):
        return (None, "", "")
    best: tuple[int | None, str, str] = (None, "", "")
    for c in comments:
        body = c.get("body", "")
        if marker_live in body and marker_dead not in body:
            best = (int(c["id"]), body, c.get("html_url", ""))
    return best


def post_new_sticky(repo: str, pr: int, body: str) -> dict:
    return gh_api("POST", f"repos/{repo}/issues/{pr}/comments", {"body": body})


def edit_comment(repo: str, comment_id: int, body: str) -> None:
    gh_api("PATCH", f"repos/{repo}/issues/comments/{comment_id}", {"body": body})


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--persona", required=True, choices=["skeptic", "auditor"])
    p.add_argument("--pr", required=True, type=int)
    p.add_argument("--repo", required=True)
    p.add_argument("--commit-sha", required=True)
    p.add_argument("--input-file", required=True,
                   help="JSON file produced by Codex against codex-output-schema.json")
    args = p.parse_args()

    if not os.environ.get("GH_TOKEN"):
        print("::error::GH_TOKEN must be set", file=sys.stderr)
        return 1

    with open(args.input_file) as f:
        raw = f.read().strip()
    if not raw:
        # Even an empty Codex output should produce a sticky so the next run's
        # `prior-*-comment.md` makes the failure visible to the agent.
        _post_error_sticky(
            args.repo, args.pr, args.persona,
            "Codex produced no output. Check the workflow logs for the model error.",
            raw="(empty)",
        )
        return 1
    try:
        doc = json.loads(raw)
    except json.JSONDecodeError as e:
        # Pass the error back to the agent via the next run's prior-comment.md.
        _post_error_sticky(
            args.repo, args.pr, args.persona,
            f"Codex emitted output that did not parse as JSON: {e}. "
            "On the next run, you (the agent) will see this comment as your "
            "prior verdict — please re-emit the output strictly per "
            "`codex-output-schema.json` (valid JSON, all required fields).",
            raw=raw,
        )
        return 1

    # Validate required top-level fields. If anything is missing, post an
    # error sticky so the agent sees the schema mismatch on the next run.
    required = {
        "verdict": str,
        "scrutiny_note": str,
        "summary_markdown": str,
        "conclusion_markdown": str,
        "inline_findings": list,
        "off_diff_findings": list,
        "prior_reconciliation": list,
    }
    problems: list[str] = []
    for key, typ in required.items():
        if key not in doc:
            problems.append(f"missing required field `{key}`")
        elif not isinstance(doc[key], typ):
            problems.append(f"`{key}` must be {typ.__name__}, got {type(doc[key]).__name__}")
    if problems:
        _post_error_sticky(
            args.repo, args.pr, args.persona,
            "Codex output parsed as JSON but does not match the schema: "
            + "; ".join(problems)
            + ". Re-emit strictly per `codex-output-schema.json`.",
            raw=raw,
        )
        return 1

    verdict = (doc.get("verdict") or "").strip()
    inline = doc.get("inline_findings") or []
    off_diff = doc.get("off_diff_findings") or []
    reconciliation = doc.get("prior_reconciliation") or []

    # 1. Find the existing live sticky (the one we are about to supersede).
    prior_id, prior_body, prior_url = find_prior_live_sticky(args.repo, args.pr, args.persona)

    # 2. Post the inline review (if any findings have a pinnable line).
    inline_urls: dict[str, str] = {}
    posted: list[dict] = []
    if inline:
        try:
            review_comments = [to_review_comment(f) for f in inline]
            _, posted = post_review(args.repo, args.pr, args.commit_sha, review_comments)
        except RuntimeError as e:
            print(f"::warning::review post failed; rendering without inline links: {e}",
                  file=sys.stderr)
        # Match returned comments back to our findings by fid embedded in the body.
        for c in posted:
            body = c.get("body", "")
            m = re.search(r"<!--\s*fid:([A-Za-z0-9]+)\s*-->", body)
            if m:
                inline_urls[m.group(1)] = c.get("html_url", "")

    # 3. Build and post the NEW sticky comment.
    findings_table = render_findings_table(inline, off_diff, inline_urls)
    new_body = render_new_sticky(
        persona=args.persona,
        verdict=verdict,
        scrutiny_note=doc.get("scrutiny_note", ""),
        summary_markdown=doc.get("summary_markdown", ""),
        conclusion_markdown=doc.get("conclusion_markdown", ""),
        findings_table=findings_table,
        off_diff=off_diff,
        reconciliation=reconciliation,
        prior_url=prior_url or None,
    )
    new_comment = post_new_sticky(args.repo, args.pr, new_body)
    new_url = new_comment.get("html_url", "")
    print(f"Posted new sticky: {new_url}", file=sys.stderr)

    # 4. If a prior live sticky existed, mark it superseded.
    if prior_id is not None:
        prior_rows = parse_prior_findings(prior_body)
        superseded_body = render_superseded_body(
            prior_body=prior_body,
            prior_rows=prior_rows,
            reconciliation=reconciliation,
            new_comment_url=new_url,
            persona=args.persona,
        )
        edit_comment(args.repo, prior_id, superseded_body)
        print(f"Marked prior sticky {prior_id} as superseded.", file=sys.stderr)

    return 0


if __name__ == "__main__":
    sys.exit(main())
