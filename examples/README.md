# PageScript Examples

## `data-lineage-demo.page`

Shows the Draft 0.4 web composition target: a product demo page with scene, panels, SVG lineage graph, metrics, effects, state, events, design tokens, layout metadata, and scoped CSS.

Useful commands:

```sh
pagescript-rs validate examples/data-lineage-demo.page
pagescript-rs ir examples/data-lineage-demo.page
pagescript-rs render examples/data-lineage-demo.page > lineage-demo.html
```

## `autonomous-revenue-command-center.page`

A more ambitious product-demo example for the strategic wedge: codebase-aware generated webpages. It uses multiple scenes, SVG graph composition, metrics, logs, state, events, effects, design tokens, layout metadata, and scoped CSS.

Useful commands:

```sh
pagescript-rs validate examples/autonomous-revenue-command-center.page
pagescript-rs ir examples/autonomous-revenue-command-center.page --page revenue-command-center
pagescript-rs render examples/autonomous-revenue-command-center.page > command-center.html
```

## `interactive-doc.page`

Shows the new renderable PageScript shape: hero, sections, cards, button actions, and a modal compiled to standalone HTML.

Useful commands:

```sh
pagescript-rs validate examples/interactive-doc.page
pagescript-rs render examples/interactive-doc.page > interactive-doc.html
```

## `dashboard.page`

Shows the compatibility shape: page-level context with a nested guided tour.

Useful commands:

```sh
pagescript-rs validate examples/dashboard.page
pagescript-rs ast examples/dashboard.page
pagescript-rs convert examples/dashboard.page --target shepherd --tour dashboard-onboarding
```

## `dashboard.tour`

Shows the compatible tour-only format. This remains useful when a project only wants to author guided overlays.

## `minimal.tour`

Smallest valid tour-only example used by CLI tests.

## Snapshot Files

`dashboard.shepherd.json` and `dashboard.intro.json` show representative adapter output for tour runtimes.
