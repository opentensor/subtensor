---
name: fix
description: Commit current changes, run Rust autofix/lint/format, run pallet-subtensor tests, amend with any fixes.
---

# Fix Skill

Create or reuse one commit, run the Rust fix pipeline in order, run unit tests, and fold all resulting changes into that same commit.

## Steps

1. Run /format
2. In a subagent (subagent_type: `general-purpose`, model: `sonnet`) run:
   - `cargo test -p pallet-subtensor --lib` and capture full output
   - If any tests fail, analyze the failures
     - Read the failing test code AND the source code it tests
     - Determine the root cause
     - Apply fixes using Edit tools
     - Re-run the tests to confirm the fix works
     - After fixing, if there are further failures, repeat (up to 3 fix-and-retest cycles)
   - Summarize:
     - Which tests failed, if any
     - What was fixed and how
     - Whether all tests pass now
3. Amend commit with test fixes, if any, then /format
4. Run `git show -s` for user to review

## Important

- Do NOT run `scripts/fix_rust.sh` — let /format take care of it
- Do NOT skip any steps
- The test subagent must fix source code to make tests pass, NOT modify tests to make them pass (unless the test itself is clearly wrong)
- If the test subagent cannot fix all failures after 3 cycles, it must return the remaining failures so the main agent can report them to the user
