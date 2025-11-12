#!/usr/bin/env python3
"""Re-run the latest pull_request workflow runs for a PR."""
from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request
from typing import Dict, List, Optional, Set

DEFAULT_SKIP_WORKFLOWS = {"Validate-Benchmarks", "Label Triggers"}
PER_PAGE = 100
MAX_PAGES = 10


def env(name: str, required: bool = True) -> str:
    value = os.environ.get(name)
    if required and not value:
        print(f"Missing required environment variable: {name}", file=sys.stderr)
        sys.exit(1)
    return value.strip() if isinstance(value, str) else value


def github_request(url: str, token: str, method: str = "GET", payload: Optional[Dict] = None) -> Dict:
    data: Optional[bytes] = None
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(url, data=data, method=method)
    request.add_header("Authorization", f"Bearer {token}")
    request.add_header("Accept", "application/vnd.github+json")
    if data:
        request.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            if response.status == 204:
                return {}
            body = response.read().decode("utf-8")
            return json.loads(body)
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="ignore")
        print(f"GitHub API error ({exc.code}) for {url}:\n{body}", file=sys.stderr)
        raise


def collect_runs(
    *,
    repo: str,
    token: str,
    pr_number: int,
    skip_names: Set[str],
    target_head: Optional[str] = None,
) -> List[Dict]:
    runs: List[Dict] = []
    seen_workflows: Set[int] = set()
    page = 1

    while page <= MAX_PAGES:
        url = (
            f"https://api.github.com/repos/{repo}/actions/runs"
            f"?event=pull_request&per_page={PER_PAGE}&page={page}"
        )
        payload = github_request(url, token)
        batch = payload.get("workflow_runs", [])
        if not batch:
            break

        for run in batch:
            if run.get("event") != "pull_request":
                continue

            prs = run.get("pull_requests") or []
            pr_numbers = {item.get("number") for item in prs if item.get("number") is not None}
            if pr_number not in pr_numbers:
                continue

            if target_head and run.get("head_sha") != target_head:
                continue

            name = run.get("name") or ""
            if name in skip_names:
                continue

            workflow_id = run.get("workflow_id")
            if workflow_id in seen_workflows:
                continue

            seen_workflows.add(workflow_id)
            runs.append(run)

        if len(batch) < PER_PAGE:
            break
        page += 1

    return runs


def main() -> None:
    repo = env("GITHUB_REPOSITORY")
    token = env("GITHUB_TOKEN")
    pr_number_raw = env("PR_NUMBER")
    try:
        pr_number = int(pr_number_raw)
    except ValueError:
        print(f"Invalid PR_NUMBER value: {pr_number_raw}", file=sys.stderr)
        sys.exit(1)

    head_sha = os.environ.get("PR_HEAD_SHA", "").strip()

    extra_skip = {
        value.strip()
        for value in os.environ.get("EXTRA_SKIP_WORKFLOWS", "").split(",")
        if value.strip()
    }
    skip_names = DEFAULT_SKIP_WORKFLOWS | extra_skip

    runs = []
    if head_sha:
        runs = collect_runs(repo=repo, token=token, pr_number=pr_number, skip_names=skip_names, target_head=head_sha)
        if not runs:
            print(
                f"No workflow runs found for PR #{pr_number} with head {head_sha}. "
                "Falling back to the latest runs for this PR.",
                file=sys.stderr,
            )

    if not runs:
        runs = collect_runs(repo=repo, token=token, pr_number=pr_number, skip_names=skip_names)

    if not runs:
        print(f"No pull_request workflow runs found for PR #{pr_number}; nothing to re-run.")
        return

    print(f"Re-running {len(runs)} workflow(s) for PR #{pr_number}.")
    for run in runs:
        run_id = run.get("id")
        name = run.get("name")
        run_number = run.get("run_number")
        if run_id is None:
            continue
        print(f"  • {name} (run #{run_number})")
        rerun_url = f"https://api.github.com/repos/{repo}/actions/runs/{run_id}/rerun"
        github_request(rerun_url, token, method="POST", payload={})


if __name__ == "__main__":
    main()
