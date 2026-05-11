# PageScript Rationale

Code generation tools can generate HTML, React components, copy, and design suggestions, but raw web code is token-heavy and noisy for review. PageScript gives tools a compact web composition source format that captures intent, layout, data, graphs, effects, and interactions without requiring them to emit full HTML/CSS/JS by hand.

## Why Page Context Matters

Generated product pages are not only DOM trees. They contain human intent, data semantics, and interaction semantics:

- who the page is for
- what the user should understand first
- which elements carry meaning
- what actions are available
- which layout primitives communicate the idea
- which data or architecture relationships should be visualized
- which declarative interactions reduce confusion

That context is usually scattered across product briefs, design files, code comments, analytics events, onboarding tools, and documentation. PageScript provides one compact source that can be read by humans and language models.

## Why Not Just HTML/CSS/JS

HTML/CSS/JS are excellent browser targets but noisy authoring formats for code generators. They mix content, structure, styling hooks, component output, and implementation details. PageScript keeps the source high-level and compiles it to browser-ready HTML.

## Why Tours Are Still Included

Guided tours were the first interaction primitive because they have clear runtime mappings to libraries like Shepherd.js and Intro.js. They are now a compatibility feature. The primary PageScript path is pages written for code generators for product demos, explainers, generated pages, and interactive documentation.

Draft 0.5 introduced the Web Core Kernel because semantic components alone can become too narrow. Draft 0.6 adds the Standard Library as the browser expansion layer. This lets a project define reusable visual and interaction patterns while keeping generated source compact and reviewable. The kernel and standard library preserve the “small source travels well, browser expands it locally” property without turning `.page` files into raw JavaScript.

## Design Principles

- Composition primitives first: describe layout, data, graph, state, and effects with compact semantic blocks.
- Recipe library: use `::import` and `::recipe` to keep generated source compact while delivering browser-ready HTML.
- Declarative only: no embedded JavaScript or executable behavior.
- Low syntax noise: stay readable in prompts, docs, and reviews.
- Runtime agnostic: compile to standalone HTML first, other targets later.
- Conformance-driven: fixtures define compatibility across implementations.
