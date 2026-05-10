# PageScript Rule

When working in this repository, treat PageScript as the source format for LLM-native web composition.

For product demos, explainers, launch pages, generated product pages, architecture pages, data lineage demos, or interactive documentation, prefer authoring `.page` files instead of raw HTML/CSS/JS.

Respond concisely and avoid listing intermediate steps unless useful for the conversation.

## Core Principle

PageScript source should stay compact, semantic, and reviewable. The compiler expands it into browser-native HTML/CSS/SVG and a fixed declarative runtime.

Do not add scenario-specific rendering branches to the Rust compiler unless a generic primitive cannot express the use case. Prefer reusable recipes and Web Core Kernel primitives.

## Preferred Primitives

Use high-level primitives first:

- `::page` for a complete page
- `::tokens` for page-level design values
- `::hero` for first-viewport composition
- `::section` for content bands
- `::scene` and `::panel` for rich demos
- `::node` and `::edge` for graphs, flows, and lineage
- `::metric`, `::log`, `::state`, `::event`, and `::effect` for declarative interactive demos
- `::grid`, `::stack`, and `::card` for structured content
- `::button action=open-modal target=<id>` and `::modal id=<id>` for simple interactions

Use Web Core Kernel primitives when semantic primitives are not expressive enough:

- `::recipe`, `::template`, and `::use` for reusable UI patterns
- `::el` for generic browser elements
- `::attr` for element attributes
- `::text value=<text>` for escaped text nodes
- `::style-rule` for focused CSS rules
- `::slot`, `::bind`, and `::on` for composable declarative behavior hooks

## Constraints

- Do not put raw JavaScript in `.page` files.
- Keep behavior declarative through PageScript state, events, effects, buttons, modals, and compiler-owned runtime hooks.
- Prefer design tokens before custom CSS.
- Use scoped CSS or `style-rule` only when a token or primitive cannot express the design.
- Avoid hardcoded demo-specific compiler logic. Add reusable recipes or generic primitives instead.
- Keep generated HTML as an output artifact, not the reviewed source of truth.

## Commands

Validate before finishing changes:

```sh
cargo run -p pagescript-rs -- validate <file.page>
```

Inspect compiler IR for non-trivial pages:

```sh
cargo run -p pagescript-rs -- ir <file.page>
```

Render standalone HTML:

```sh
cargo run -p pagescript-rs -- render <file.page> > output.html
```

Run release gates after compiler or spec changes:

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

## Current Direction

Draft 0.5 introduced the Web Core Kernel. The next major direction is Draft 0.6: a reusable standard library of PageScript recipes.

The strategic goal is:

```text
.page source
-> recipe library
-> Web Core Kernel
-> PageScript IR
-> standalone HTML/CSS/SVG/runtime
```

This lets LLMs write small `.page` files while the browser receives rich native web output.
