---
name: ship
description: Ship the current branch: fix, push, create PR, watch CI, fix failures, code review
---

# Ship Skill

Ship the current branch: fix, push, create PR if needed, watch CI, fix failures, and perform code review.

## Phase 1: Fix and Push

1. **Run `/fix`** — invoke the fix skill to commit, lint, and format.
2. **Push the branch** to origin: `git push -u origin HEAD`.
3. **Create a PR if none exists**:
   - Check: `gh pr view --json number 2>/dev/null` — if it fails, no PR exists yet.
   - If no PR exists, create one:
     - Use `git log main..HEAD --oneline` to understand all commits on the branch.
     - Read the changed files with `git diff main...HEAD --stat` to understand scope.
     - Create the PR with `gh pr create --title "<concise title>" --body "<detailed markdown description>" --label "skip-cargo-audit"`.
     - The description must include: a **Summary** section (bullet points of what changed and why), a **Changes** section (key files/modules affected), and a **Test plan** section.
   - If a PR already exists, just note its number/URL.

## Phase 2: Watch CI and Fix Failures

4. **Poll CI status** in a loop:
   - Run: `gh pr checks --json name,state,conclusion,link --watch --fail-fast 2>/dev/null || gh pr checks`
   - If `--watch` is not available, poll manually every 90 seconds using `gh pr checks --json name,state,conclusion,link` until all checks have completed (no checks with state "pending" or conclusion "").
   - **Ignore these known-flaky/irrelevant checks** — treat them as passing even if they fail:
     - `validate-benchmarks` (benchmark CI — not relevant)
     - Any `Contract E2E Tests` check that failed only due to a timeout (look for timeout in the failure link/logs)
     - `cargo-audit` (we already added the skip label)
   - Also ignore any checks related to `check-spec-version` and `e2e` tests — these are environment-dependent and not fixable from code.

5. **If there are real CI failures** (failures NOT in the ignore list above):
   - For EACH distinct failing check, launch a **separate Task subagent** (subagent_type: `general-purpose`, model: `sonnet`) in parallel. Each subagent must:
     - Fetch the failed check's logs: use `gh run view <run-id> --log-failed` or the check link to get failure details.
     - Investigate the root cause by reading relevant source files.
     - Return a **fix plan**: a description of what needs to change and in which files, with specific code snippets showing the fix.
   - **Wait for all subagents** to return their fix plans.

6. **Aggregate and apply fixes**:
   - Review all returned fix plans for conflicts or overlaps.
   - Apply the fixes using Edit/Write tools.
   - Run `/fix` again (invoke the fix skill) to commit, lint, and format the fixes.
   - Push: `git push`.

7. **Re-check CI**: Go back to step 4 and poll again. Repeat the fix cycle up to **3 times**. If CI still fails after 3 rounds, report the remaining failures to the user and stop.

## Phase 3: Code Review

8. **Once CI is green** (or only ignored checks are failing), perform a thorough code review.

9. **Launch a single Opus subagent** (subagent_type: `general-purpose`, model: `opus`) for the review:
   - It must get the full PR diff: `git diff main...HEAD`.
   - It must read every changed file in full.
   - It must produce a numbered list of **issues** found, where each issue has:
     - A unique sequential ID (e.g., `R-1`, `R-2`, ...).
     - **Severity**: critical / major / minor / nit.
     - **File and line(s)** affected.
     - **Description** of the problem.
   - The review must check for: correctness, safety (no panics, no unchecked arithmetic, no indexing), edge cases, naming, documentation gaps, test coverage, and adherence to Substrate/Rust best practices.
   - Return the full list of issues.

10. **For each issue**, launch TWO subagents **in parallel**:
    - **Fix designer** (subagent_type: `general-purpose`, model: `sonnet`): Given the issue description and relevant code context, design a concrete proposed fix with exact code changes (old code -> new code). Return the fix as a structured plan.
    - **Fix reviewer** (subagent_type: `general-purpose`, model: `opus`): Given the issue description, the relevant code context, and the proposed fix (once the fix designer returns — so the reviewer runs AFTER the designer, but reviewers for different issues run in parallel with each other). The reviewer must check:
      - Does the fix actually solve the issue?
      - Does it introduce new problems?
      - Is it the simplest correct fix?
      - Return: approved / rejected with reasoning.

    Implementation note: For each issue, first launch the fix designer. Once the fix designer for that issue returns, launch the fix reviewer for that issue. But all issues should be processed in parallel — i.e., launch all fix designers at once, then as each designer returns, launch its corresponding reviewer. You may batch reviewers if designers finish close together.

11. **Report to user**: Present a formatted summary:
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
- When fetching CI logs, if `gh run view` output is very long, focus on the failed step output only.
- Do NOT apply code review fixes automatically — always present them for user approval first.
- Use HEREDOC syntax for PR body and commit messages to preserve formatting.
