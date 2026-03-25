---
name: format
description: Commit current changes, run Rust autofix/lint/format, amend with any fixes.
---

# Format Skill

Create or reuse one commit, run the Rust fix pipeline in order and fold all resulting changes into that same commit.

## Steps

1. Stage all changes and create a commit with a descriptive message summarizing the changes
2. Do this:
   a. Run `cargo check --workspace`
   b. Run `cargo clippy --fix --workspace --all-features --all-targets --allow-dirty`
   c. Run `cargo fix --workspace --all-features --all-targets --allow-dirty`
   d. Run `cargo fmt --all`
   e. Amend the commit with any changes
3. Run `git show -s` for user to review

## Important

- If a fix tool fails in step 2, stop and report the error to the user rather than continuing
- Do NOT run `scripts/fix_rust.sh` itself — run the individual commands listed above instead
- Do NOT skip any steps
