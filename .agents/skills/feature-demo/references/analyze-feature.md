# Analyzing a feature into a feature model

The feature model is the single source of truth for both demo modes. Spend real effort here —
an inaccurate model produces a confident, wrong demo.

## 1. Gather context

Resolve the input to a base branch and a feature branch:

- **PR:** `gh pr view <n> --json title,body,headRefName,baseRefName,files,commits` then
  `gh pr diff <n>`.
- **Branch:** `git fetch origin <branch> <base>` then `git diff origin/<base>...origin/<branch>`
  (base is usually `main`; for this repo a PR may target `devnet-ready` or `testnet` — use the
  PR's actual `baseRefName`).

Then read — do not skim:

- Every changed source file; new modules first.
- Tests — they show intended behavior and the edge cases the author cared about.
- Migrations — they show storage / on-chain state changes.
- Benchmarks, runtime/config wiring, `Cargo.toml` — they show how the feature is hooked in.
- The PR description and any linked issues for intent and rationale.

If behavior is ambiguous, ask the user. Never invent behavior to fill a gap.

## 2. Write `feature-model.md`

Save it in the output folder. Sections:

### Header
- `feature` — short name.
- `summary` — one sentence, plain language.
- `problem` — what was wrong or missing before this feature.
- `branch` / `pr` — identifiers, base branch.

### Components
The parts of the system the feature involves. These become nodes in the technical graph and
actors in the non-technical narrative. For each: `id`, `name`, `role` (one line),
`kind` (existing | new | modified), `source` (file path, `file:line` where useful).

### Flows
Directed relationships between components — data flow, control flow, handoffs, hooks. For each:
`from`, `to`, `label`, `when` (the trigger or condition).

### Parameters — the heart of the demo
The tunable knobs the feature introduces or changes. For each:
- `id`, `name`.
- `type` — range | enum | boolean.
- `default`, plus `min` / `max` / `step` or `options`, and `unit`.
- `maps_to` — the code symbol it corresponds to (storage item, config constant, extrinsic arg).
- `realistic` — a realistic value or scenario for the default view.

### Parameter → effect mapping — what the visualization shows
For each parameter, the concrete cause→effect chain: which components change, which values
recompute, and the formula or rule. Be concrete enough to implement in JavaScript. Example:

> `tempo` ↑ → epoch period (`tempo + 1`) ↑ → fewer epochs per day → `next_epoch_block`
> shifts later → coinbase emission cadence slows.

Pull the real formulas from the code. This mapping must be **executable** — it becomes
`model/effects.ts`.

### Lifecycle / states
If the feature has a state machine or staged process: the states, the transitions, and what
triggers each. Many features have one (a referendum lifecycle, a request lifecycle, an epoch).

### Failure modes
How the feature breaks if misconfigured or misused — out-of-range parameters, missing guards,
ordering bugs, panics at boot. The technical demo surfaces these explicitly.

### Narrative beats
For the non-technical mode: an ordered list of 6–12 beats, each one idea, building from "why"
through "how" to "what it means". Each beat maps to one slide.

## 3. Sanity-check the model

- Every parameter has an effect mapping with a real formula or rule — not a vague description.
- Every component appears in at least one flow.
- `summary` and `problem` are understandable by a non-engineer.
- Every number (default, bound) comes from the code, not a guess.
- You could hand `feature-model.md` to a stranger and they could rebuild the demo from it.
