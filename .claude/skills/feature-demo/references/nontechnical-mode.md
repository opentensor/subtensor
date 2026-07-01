# Non-technical mode — narrated walkthrough

Audience: the community — token holders, users, non-engineers. Goal: they understand what
changed, why it matters to them, and feel the mechanic by touching it once.

Reference example: the Subtensor *governance walkthrough* — a hash-routed slide deck, one idea
per slide, plain language, a friendly semantic palette, smooth transitions, and one interactive
widget per mechanic slide.

## Layout

A slide deck. One concept per slide. Hash-routed (`#1`, `#2`, …) so any slide is linkable.
Prev / next buttons, arrow-key navigation, and a progress indicator (slide N of M).

Slides come from the model's narrative beats. Typical arc:

- **Title** — the feature, one line.
- **Why** — the problem, in human terms.
- **The core idea** — one sentence.
- **3–6 mechanic slides** — one piece each, every one carrying an interactive widget.
- **Trade-offs** — what was chosen, and what was given up.
- **Safety / limits** — what stops it from going wrong.
- **Closing** — what it means for the community.

## Visual style

- Sans-serif, generous spacing, large readable type. No code, no jargon — or define a term
  inline, once, the first time it appears.
- Friendly semantic colors: green = good / approve, red = stop / reject, amber = caution,
  blue = neutral / info.
- Smooth slide transitions with `motion` (Framer Motion).

## Required interactive pieces

1. **One widget per mechanic slide** — a slider, toggle, or small simulation tied to a real
   parameter from the model.
2. **A visible consequence.** Moving the widget changes something the viewer can see: a number,
   a chart (`recharts`), an animation, a component lighting up. Use the *same*
   `model/effects.ts` functions as the technical mode, so both demos agree on the numbers.
3. **Plain-language framing.** Each slide states its takeaway in one sentence before the widget;
   the widget then proves that sentence.

## Writing rules

- No undefined jargon. Translate every code term into a human one (e.g. "epoch" → "scoring
  round", "extrinsic" → "on-chain action").
- One idea per slide. If a slide needs two paragraphs of explanation, it is two slides.
- Lead with the consequence for the user, not the implementation detail.

## Done when

A non-engineer can click through every slide, touch each widget, and then explain back what the
feature does and why it matters.
