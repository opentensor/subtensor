# Technical mode — architecture walkthrough

Audience: engineers who will read the code. Goal: they understand the architecture, can reason
about every parameter, and see how the feature fails when misconfigured.

Reference example: the Subtensor *governance architecture walkthrough* — a sidebar-navigated
single page with an interactive component graph, a live referendum simulator, an adjustment-curve
chart, and explicit failure-mode callouts, all written in code-precise language.

## Layout

Single page. Left sidebar navigation, collapsible. One scrollable section per major area of the
feature; clicking a sidebar entry jumps to its section.

Typical sections — adapt to the feature:

- **Architecture** — the component graph.
- **Structure / composability** — how the parts connect; traits, interfaces, ownership.
- **Configuration** — every parameter, what it does, its bounds and code symbol.
- **Lifecycle** — the state machine, if the feature has one.
- **Simulator** — the live, interactive core.
- **Failure modes** — how misconfiguration breaks it.

## Visual style

- Dense, precise, monospace-forward. Mono font for code and identifiers
  (`ui-monospace, "JetBrains Mono", "Fira Code", Consolas`); a sans-serif for prose.
- Light background, restrained palette — greys plus one accent. No decorative gradients.
- Show the real code symbols: type names, trait names, extrinsic names, `pallet/file.rs`
  references. Engineers trust a demo that names the same things the code does.

## Required interactive pieces

1. **Component graph.** Use `@xyflow/react`. Nodes = components from the model, edges = flows.
   Lay it out, make it pannable / zoomable / draggable. Clicking a node opens its details —
   role, source file, the parameters that touch it.
2. **Parameter panel.** Every parameter from the model as a real control: slider for `range`,
   select for `enum`, switch for `boolean`. Label each with its `maps_to` code symbol and bounds.
3. **Live effect.** Changing a parameter immediately recomputes derived state through the pure
   functions in `model/effects.ts` and updates, in the same frame: highlighted nodes/edges in
   the graph, a results panel listing the recomputed values, and any chart.
4. **Simulator** — if the feature has a lifecycle. Let the user step or play through the state
   machine, drive its inputs, and watch the current state, transitions, and side effects.
5. **Failure-mode callouts.** For each failure mode, a preset or control that reproduces it,
   plus a plain explanation of exactly what breaks and why.

## Done when

An engineer can open it, change every parameter, watch the exact components and values that
depend on it react, and trace each effect back to a named code symbol.
