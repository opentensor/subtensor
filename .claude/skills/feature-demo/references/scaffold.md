# Scaffolding and running the SPA

One Vite app per mode. Commands assume npm; pnpm or yarn work the same way.

## Where to create it

Always scaffold inside `tmp/` — `tmp/feature-demo/<branch-slug>/` (add a `-technical` /
`-nontechnical` suffix when building both). `tmp/` is git-ignored, so a generated project can
never be accidentally committed. Never scaffold anywhere else in the repo.

## Create the project

```sh
mkdir -p tmp/feature-demo
npm create vite@latest tmp/feature-demo/<branch-slug> -- --template react-ts
cd tmp/feature-demo/<branch-slug>
npm install
```

## Add Tailwind v4

```sh
npm install tailwindcss @tailwindcss/vite
```

In `vite.config.ts`, add the plugin:

```ts
import tailwindcss from '@tailwindcss/vite'
// plugins: [react(), tailwindcss()]
```

Replace the contents of `src/index.css` with a single line:

```css
@import "tailwindcss";
```

## Add mode libraries

- **Technical mode:** `npm install @xyflow/react lucide-react`
- **Non-technical mode:** `npm install recharts motion lucide-react`
  - `motion` is the current package for Framer Motion; import from `motion/react`.

Install latest versions. If a library's current API differs from what a mode spec describes,
check that library's docs and adapt — the spec describes intent, not exact signatures. Do not
pin to old versions.

## Project structure

```
src/
  main.tsx
  App.tsx              # layout shell — sidebar (technical) or slide router (non-technical)
  feature-model.ts     # the feature model as typed, importable data
  model/effects.ts     # pure functions: parameters -> derived system state
  components/          # graph, controls, panels, slides, charts
  index.css
feature-model.md       # the human-readable model — keep this in the folder too
README.md              # how to run
```

Encode the feature model into `feature-model.ts` as typed data (components, flows, parameters,
effect metadata). Put every cause→effect calculation into pure functions in `model/effects.ts`:
`(parameters) => derivedState`. The UI is a thin layer over these functions — both modes import
the *same* `effects.ts`, so a technical and non-technical demo of the same feature never
disagree.

## Verify

`npm run build` must pass with no TypeScript errors before you hand off. Fix every error; do not
hand the user a project that does not compile.

## README.md to write into the output folder

State the single run command and what the demo is:

```md
# <Feature> — <technical|non-technical> demo

Interactive walkthrough of <feature> (branch `<branch>` / PR #<n>).

## Run

    npm install && npm run dev

Then open the printed `localhost` URL.
```

After `npm install` has been run once, the user's single command is `npm run dev`.
