# PageScript

PageScript is a text format for coding agents that write web pages. The Rust compiler turns `.page` files into standalone HTML with styling and interactions. The current release is [v1.1.0-alpha.2](https://github.com/oliver-morrow/pagescript/releases/tag/v1.1.0-alpha.2).

## Try it

Teach your coding agent to write and render PageScript with the [agent skill](skills/pagescript/SKILL.md).

From this checkout:

```sh
cargo install --path rust/pagescript-rs --locked
pagescript new demo.page
pagescript render demo.page --out index.html
```

Open `index.html` in your browser. A small input looks like this:

```text
::page id=hello title="Hello"
  ::hero spacing=xl
    heading="Hello from PageScript"
    body="This page was rendered from a text file."
  ::/hero
::/page
```

See the [examples](examples/README.md) for reusable recipes, diagrams, and interactions. Use `pagescript validate demo.page` to check a file and `pagescript guide` for syntax examples.

## Source-cited explainers

The alpha can render a reviewed JSON evidence bundle as an HTML architecture or lineage diagram with file and line citations.

```sh
pagescript explain conformance/evidence/valid/minimal.evidence.json \
  --spec conformance/explainer/valid/minimal.explainer.json --out explainer.html
```

The [evidence bundle](schemas/evidence-bundle.schema.json) and [explainer spec](schemas/explainer-spec.schema.json) schemas describe the input. The [language reference](SPEC.md) covers how it is rendered.

## Token benchmarks

Five examples compare authored PageScript with hand-written HTML using `o200k_base`. The results range from 29.87% fewer tokens for the incident timeline to 22.42% more for paper sizes. [Counts, inputs, and reproduction](benchmarks/README.md)

The original revenue map has 1,787 source tokens and 5,901 tokens in the compiler's standalone HTML. The output bundles shared CSS and JavaScript. [Revenue map report](conformance/stats/revenue-map.o200k.json)

```sh
pagescript stats examples/revenue-map-demo.page --page revenue-map
```

## Reference and development

- [Language reference](SPEC.md)
- [Compiler fixtures](conformance/)
- [Generation guide](docs/generation-guide.md)
- [Browser compiler](docs/browser-compiler.md)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
```

When compiler behavior changes, update the language reference, schemas, and fixtures to match. Run `bash scripts/build-site.sh` after changing the playground.
