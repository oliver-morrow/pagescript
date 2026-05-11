# PageScript Examples

## `product-intelligence-demo.page`

The Draft 0.6 flagship. Demonstrates the PageScript standard library, named slots, and recursive imports. Composes a high-fidelity landing page with metrics and data flows using mostly imported recipes from `stdlib/`.

Useful commands:

```sh
pagescript validate examples/product-intelligence-demo.page
pagescript ir examples/product-intelligence-demo.page
pagescript render examples/product-intelligence-demo.page --out demo.html
```

## `data-lineage-demo.page`

Shows the Draft 0.5 web composition target: a product demo page with scene, panels, SVG lineage graph, metrics, effects, state, events, design tokens, layout metadata, and scoped CSS.

Useful commands:

```sh
pagescript validate examples/data-lineage-demo.page
pagescript ir examples/data-lineage-demo.page
pagescript render examples/data-lineage-demo.page --out lineage-demo.html
```

## `web-core-kernel.page`

Shows the Draft 0.5 Web Core Kernel: generic `el` and `attr` primitives, focused `style-rule` blocks, escaped text nodes, and compile-time `recipe`/`use` expansion into ordinary HTML.

Useful commands:

```sh
pagescript validate examples/web-core-kernel.page
pagescript ir examples/web-core-kernel.page --page web-core-kernel
pagescript render examples/web-core-kernel.page --out web-core-kernel.html
```

## `autonomous-revenue-command-center.page`

A more ambitious product-demo example for the strategic wedge: codebase-aware generated webpages. It uses multiple scenes, SVG graph composition, metrics, logs, state, events, effects, design tokens, layout metadata, and scoped CSS.

Useful commands:

```sh
pagescript validate examples/autonomous-revenue-command-center.page
pagescript ir examples/autonomous-revenue-command-center.page --page revenue-command-center
pagescript render examples/autonomous-revenue-command-center.page --out command-center.html
```

## `interactive-doc.page`

Shows the new renderable PageScript shape: hero, sections, cards, button actions, and a modal compiled to standalone HTML.

Useful commands:

```sh
pagescript validate examples/interactive-doc.page
pagescript render examples/interactive-doc.page --out interactive-doc.html
```

## `dashboard.page`

Shows the compatibility shape: page-level context with a nested guided tour.

Useful commands:

```sh
pagescript validate examples/dashboard.page
pagescript ast examples/dashboard.page
pagescript convert examples/dashboard.page --target shepherd --tour dashboard-onboarding
```

## `dashboard.tour`

Shows the compatible tour-only format. This remains useful when a project only wants to author guided overlays.

## `minimal.tour`

Smallest valid tour-only example used by CLI tests.

## Snapshot Files

`dashboard.shepherd.json` and `dashboard.intro.json` show representative adapter output for tour runtimes.
