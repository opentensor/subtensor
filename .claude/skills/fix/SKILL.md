---
name: fix
description: Commit current changes, run Rust autofix/lint/format, run pallet-subtensor tests, amend with any fixes.
---

# Fix Skill

Create or reuse one commit, run the Rust fix pipeline in order, run unit tests, and fold all resulting changes into that same commit.

## Steps

1. **Initial commit**: Stage all changes and create a commit with a descriptive message summarizing the changes (use `git add -A && git commit -m "<descriptive message>"`). If there are no changes to commit, create no commit but still proceed with the fix tools below.

2. **Run each fix tool in order**. After EACH tool, check `git status --porcelain` for changes. If there are changes, stage them and amend the commit (`git add -A && git commit --amend --no-edit`).

   The tools to run in order:

   a. `cargo check --workspace`
   b. `cargo clippy --fix --workspace --all-features --all-targets --allow-dirty`
   c. `cargo fix --workspace --all-features --all-targets --allow-dirty`
   d. `cargo fmt --all`

3. **Run unit tests in a Sonnet subagent**: Launch a Task subagent (subagent_type: `general-purpose`, model: `sonnet`) that runs:
   ```
   cargo test -p pallet-subtensor --lib
   ```
   The subagent must:
   - Run the test command and capture full output.
   - If all tests pass, report success and return.
   - If any tests fail, analyze the failures: read the failing test code AND the source code it tests, determine the root cause, apply fixes using Edit tools, and re-run the tests to confirm the fix works.
   - After fixing, if there are further failures, repeat (up to 3 fix-and-retest cycles).
   - Return a summary of: which tests failed, what was fixed, and whether all tests pass now.

4. **Amend commit with test fixes**: After the subagent returns, if any code changes were made (check `git status --porcelain`), stage and amend the commit (`git add -A && git commit --amend --no-edit`). Then re-run the fix tools from step 2 (since code changes from test fixes may need formatting/clippy cleanup), amending after each if there are changes.

5. **Final output**: Show `git log --oneline -1` so the user can see the resulting commit.

## Important

- Use `--allow-dirty` flags on clippy --fix and cargo fix since the working tree may have unstaged changes between steps.
- If a fix tool fails (step 2/4), stop and report the error to the user rather than continuing.
- Do NOT run `scripts/fix_rust.sh` itself — run the individual commands listed above instead.
- Do NOT skip any step. Run all four fix tools even if earlier ones produced no changes.
- The test subagent must fix source code to make tests pass, NOT modify tests to make them pass (unless the test itself is clearly wrong).
- If the test subagent cannot fix all failures after 3 cycles, it must return the remaining failures so the main agent can report them to the user.
