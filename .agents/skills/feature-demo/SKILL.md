---
name: feature-demo
description: >-
  Build a runnable, interactive single-page app that visualizes a newly-developed feature for a
  community audience, starting from a git branch or PR. Produces a technical walkthrough
  (architecture graph + live simulator) or a non-technical walkthrough (narrated slides) — both
  with parameter controls that show how knobs affect each component of the system. Use when the
  user wants to demo, showcase, explain, or visualize a branch / PR / feature for the community,
  or asks to "make a demo of the code".
---

# Feature Demo

Turn a branch or PR into an interactive single-page app the community can open, run, and play
with: change parameters, watch which components of the system react.

## When to run

The user points at a branch or PR — "make a demo for branch X", "explain PR #1234 to the
community". Two output styles, both interactive SPAs:

- **Technical** — architecture walkthrough for engineers: component graph, live simulator, code
  references, failure modes. Spec: [references/technical-mode.md](references/technical-mode.md).
- **Non-technical** — narrated slide walkthrough for the community: one idea per slide, plain
  language, one interactive widget per slide. Spec: [references/nontechnical-mode.md](references/nontechnical-mode.md).

If the user did not say which, ask: technical, non-technical, or both. Both → one app folder
per mode.

## Workflow

1. **Resolve the input.** A branch name or a PR. Get the diff and full context — see
   [references/analyze-feature.md](references/analyze-feature.md): `gh pr view` + `gh pr diff`,
   or `git fetch && git diff <base>...<branch>`. Read the changed files, tests, migrations and
   wiring until you understand the feature end to end.
2. **Write the feature model.** Produce `feature-model.md` — the structured spec both renderers
   consume: components, flows, parameters, the parameter→effect mapping, lifecycle/states,
   failure modes, narrative beats. Schema in [references/analyze-feature.md](references/analyze-feature.md).
   Do not skip this; it is what keeps the demo accurate.
3. **Confirm with the user.** Show the feature-model summary and the chosen mode(s) before
   building. Fix misunderstandings now, not in the SPA.
4. **Scaffold the SPA.** One Vite + React + TS + Tailwind app per mode, **always created
   inside `tmp/`** — exact commands in [references/scaffold.md](references/scaffold.md).
5. **Build the content** from the feature model, following the chosen mode spec. Non-negotiable:
   real parameter controls bound to the feature's actual knobs, and a visualization that reacts.
6. **Verify and hand off.** Run `npm run build` — it must compile clean. Write a `README.md` in
   the output folder with the single run command. Tell the user the folder and the command.

## Core principle

Every demo answers one question visually: **change a parameter → see which components of the
system change, and how.** A demo with no working controls, or controls that do not visibly move
the system, is not done. The feature model's parameter→effect mapping is the contract for this;
keep that logic in pure functions (`model/effects.ts`) shared by both modes.

## Output

Always scaffold into `tmp/feature-demo/<branch-slug>/` (one subfolder per mode) — never anywhere
else in the repo. `tmp/` is git-ignored, so the generated project (and its large `node_modules`)
can never be committed by accident. If `tmp/` does not exist yet, create it. The user runs the
app with one command (`npm run dev`). Never auto-deploy.
