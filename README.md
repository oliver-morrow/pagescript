# PageScript

PageScript is a Rust compiler and CLI that turns `.page` text files into standalone HTML with styling and interactions. The current release is [v1.1.0-alpha.1](https://github.com/oliver-morrow/pagescript/releases/tag/v1.1.0-alpha.1).

## Try it

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

The [explainer docs](docs/v1-explainer-design.md) describe the input and the evidence each citation carries.

## Token-count example

The checked-in revenue map uses 1,787 tokens as PageScript source and 4,975 tokens as generated HTML with the `o200k_base` tokenizer. The comparison measures just those two files.

```sh
pagescript stats examples/revenue-map-demo.page --page revenue-map
```

[Expected report](conformance/stats/revenue-map.o200k.json)

## Reference and development

- [Language reference](SPEC.md)
- [Compiler tests and fixtures](CONFORMANCE.md)
- [Browser compiler](docs/browser-compiler.md)
- [Documentation index](docs/index.md)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
```
