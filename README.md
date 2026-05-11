# PageScript

[![Spec](https://img.shields.io/badge/spec-Draft%200.6-blue)](./SPEC.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

PageScript is a draft open standard and CLI for LLM-native web composition. It lets AI coding tools and humans write compact `.page` files using semantic layout, design tokens, data, interaction, graph, effect, and Web Core Kernel primitives, then compile them into standalone HTML.

The canonical implementation is Rust:

- `rust/pagescript-rs`: active parser, validator, IR compiler, HTML renderer, adapters, and native CLI.
- `legacy/typescript-reference`: archived TypeScript reference retained for historical/npm-web context.

## Status

- Standard status: Draft 0.6
- Canonical implementation: Rust
- TypeScript implementation: legacy

## Quick Start

```sh
cargo install --path rust/pagescript-rs
pagescript new demo.page
pagescript render demo.page --out index.html
# Open index.html in your browser.
```

During local development:

```sh
cargo run -p pagescript-rs -- new demo.page --force
cargo run -p pagescript-rs -- render demo.page --out index.html
```

## Example

```text
::page id=lineage-demo title="Data Lineage Demo"
  ::state id=selectedNode default=warehouse
  ::/state

  ::event on=node.click set=selectedNode value="$node.id"
  ::/event

  ::effect id=flow type=flow
  ::/effect

  ::tokens
    color.accent="#4dd6a0"
    radius.panel=14
  ::/tokens

  ::scene id=lineage layout=split title="Live Data Lineage"
    ::panel id=pipeline title="Pipeline graph"
      ::node id=source label="Stripe Events" status=active x=90 y=90
      ::/node
      ::node id=warehouse label="Snowflake" status=syncing x=290 y=165
      ::/node
      ::edge from=source to=warehouse effect=flow
      ::/edge
    ::/panel
  ::/scene
::/page
```

Draft 0.5 introduced the generic Web Core Kernel, and Draft 0.6 adds the Standard Library for browser-native expansion:

```text
::import from="stdlib/product.page"
::/import

::use recipe=product-hero title="PageScript" subtitle="LLM-native web composition"
  ::slot name=actions
    ::button label="Get Started" action=toggle target=signup
    ::/button
  ::/slot
::/use
```

## Standard Library

PageScript includes a built-in standard library of reusable recipes in `stdlib/`:

- `product.page`: Marketing and landing page components
- `data.page`: Data visualization and dashboard components
- `docs.page`: Documentation and API reference components
- `layout.page`: Layout and grid systems

## Rust API

```rust
use pagescript_rs::{Resolver, compile_page_ir, parse_page_script, render_to_html, validate_document};

let document = parse_page_script(source);
let resolver = Resolver::new(Some(base_path.into()));
let diagnostics = validate_document(&document, &resolver);
let ir = compile_page_ir(&document, None, &resolver)?;
let html = render_to_html(&document, None, &resolver)?;
```

## CLI

```sh
pagescript guide
pagescript new demo.page --template product
pagescript validate demo.page
pagescript validate demo.page --json
pagescript ast examples/data-lineage-demo.page
pagescript ir examples/data-lineage-demo.page
pagescript render demo.page --out index.html
pagescript render examples/web-core-kernel.page > web-core-kernel.html
pagescript convert examples/dashboard.page --target shepherd --tour dashboard-onboarding
```

## Standard Documents

- [SPEC.md](./SPEC.md): normative draft standard
- [RATIONALE.md](./RATIONALE.md): why LLM-native page authoring matters
- [CONFORMANCE.md](./CONFORMANCE.md): parser and validator compatibility expectations
- [schemas/](./schemas): machine-readable AST, IR, and diagnostic schemas
- [conformance/](./conformance): compatibility fixtures
- [docs/agent-workflows.md](./docs/agent-workflows.md): using PageScript as a Cursor/Claude Code/Codex org standard
- [docs/llm-generation.md](./docs/llm-generation.md): compact syntax guide, canonical examples, and repair-loop workflow for LLM generation
- [docs/extension-model.md](./docs/extension-model.md): deterministic core, recipe velocity, and escape-hatch policy

## Development

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## CI/CD

GitHub Actions workflows cover Rust verification, conformance smoke tests, crate packaging, docs deployment, tagged releases, and dependency auditing under `.github/workflows/`.
The GitHub Pages homepage is authored in [docs/index.page](./docs/index.page) and rendered by the docs workflow.

## Releases

Release automation is split across two workflows:

- `Release-plz` runs on pushes to `main`, opens or updates a release PR with the next crate version and changelog, then publishes the crate and creates the GitHub release when that release PR is merged.
- `Release` builds and uploads Linux, macOS, and Windows binary archives to the GitHub release.

Repository setup required:

- Enable GitHub Actions workflow permissions that allow Actions to create pull requests.
- Configure `CARGO_REGISTRY_TOKEN` with crates.io publish permissions before expecting automated crate publishing.

The TypeScript implementation is preserved under `legacy/typescript-reference` and is not part of the active release gate.
