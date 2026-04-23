---
name: ship
description: Ship current branch end-to-end: run /fix, push, open/update PR, triage CI failures, then deliver review findings for approval.
---

# Ship Skill

Ship the branch through CI and review without force-pushes, and never apply review fixes without explicit user approval.

Run the following skill in a subagent to prevent context pollution. Make the subagent return a short summary to the main agent.

1. Run `/fix`
2. Push the branch to origin
3. Create a PR with a comprehensive description if none exists yet
   - Update the description if PR exists already
   - Add label `skip-cargo-audit` to the PR
4. Poll CI status in a loop:
   - Run: `gh pr checks --json name,state,conclusion,link --watch --fail-fast 2>/dev/null || gh pr checks`
   - If `--watch` is not available, poll manually every 90 seconds using `gh pr checks --json name,state,conclusion,link` until all checks have completed (no checks with state "pending" or conclusion "").
   - **Ignore these known-flaky/irrelevant checks** — treat them as passing even if they fail:
     - `validate-benchmarks` (benchmark CI — not relevant)
     - Any `Contract E2E Tests` check that failed only due to a timeout (look for timeout in the failure link/logs)
     - `cargo-audit`
5. **If there are real CI failures** (failures NOT in the ignore list above):
   - For EACH distinct failing check, launch a **separate Task subagent** (subagent_type: `general-purpose`, model: `sonnet`) in parallel. Each subagent must:
     - Fetch the failed check's logs: use `gh run view <run-id> --log-failed` or the check link to get failure details.
     - Investigate the root cause by reading relevant source files.
     - Return a **fix plan**: a description of what needs to change and in which files, with specific code snippets showing the fix.
   - **Wait for all subagents** to return their fix plans.
6. **Aggregate and apply fixes**:
   - Review all returned fix plans for conflicts or overlaps.
   - Apply the fixes using Edit/Write tools.
   - Invoke the /fix skill
   - `git push`
7. **Re-check CI**: Go back to step 4 and poll again. Repeat the fix cycle up to **3 times**. If CI still fails after 3 rounds, report the remaining failures to the user and stop.
8. **Once CI is green** (or only ignored checks are failing), perform a thorough code review.
   - **Launch a single Opus subagent** (subagent_type: `general-purpose`, model: `opus`) for the review:
     - It must get the full PR diff: `git diff main...HEAD`.
     - It must read every changed file in full.
     - It must produce a numbered list of **issues** found, where each issue has:
       - A unique sequential ID (e.g., `R-1`, `R-2`, ...).
       - **Severity**: critical / major / minor / nit.
       - **File and line(s)** affected.
       - **Description** of the problem.
     - The review must check for: correctness, safety (no panics, no unchecked arithmetic, no indexing), edge cases, naming, documentation gaps, test coverage, and adherence to Substrate/Rust best practices.
     - Return the full list of issues.
9. **For each issue**, run fix designer then fix reviewer in sequence; run all issues concurrently with each other:
    - **Fix designer** (subagent_type: `general-purpose`, model: `sonnet`): Given the issue description and relevant code context, design a concrete proposed fix with exact code changes (old code -> new code). Return the fix as a structured plan.
    - **Fix reviewer** (subagent_type: `general-purpose`, model: `opus`): Given the issue description, the relevant code context, and the proposed fix (once the fix designer returns — so the reviewer runs AFTER the designer, but reviewers for different issues run in parallel with each other). The reviewer must check:
      - Does the fix actually solve the issue?
      - Does it introduce new problems?
      - Is it the simplest correct fix?
      - Return: approved / rejected with reasoning.

    Implementation note: For each issue, first launch the fix designer. Once the fix designer for that issue returns, launch the fix reviewer for that issue. But all issues should be processed in parallel — i.e., launch all fix designers at once, then as each designer returns, launch its corresponding reviewer. You may batch reviewers if designers finish close together.

10. **Report to user**: Present a formatted summary:
    ```
    ## Code Review Results

    ### R-1: <title> [severity]
    **File**: path/to/file.rs:42
    **Issue**: <description>
    **Proposed fix**: <summary of fix>
    **Review**: Approved / Rejected — <reasoning>

    ### R-2: ...
    ```
    Ask the user which fixes to apply (all approved ones, specific ones by ID, or none).

## Important Rules

- Never force-push. Always use regular `git push`.
- All CI polling must have a maximum total wall-clock timeout of 45 minutes. If CI hasn't finished by then, report current status and stop waiting.
- When fetching CI logs, use a subagent to isolate the relevant part. If `gh run view` output is very long, focus on the failed step output only.
- Do NOT apply code review fixes automatically — always present them for user approval first.
- Use HEREDOC syntax for PR body and commit messages to preserve formatting.
