# PageScript

[![Spec](https://img.shields.io/badge/spec-Draft%200.4-blue)](./SPEC.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

PageScript is a draft open standard for LLM-native web composition. It lets AI coding tools and humans write compact `.page` files using semantic layout, design tokens, data, interaction, graph, and effect primitives, then compile them into standalone HTML.

The canonical implementation is Rust:

- `rust/pagescript-rs`: active parser, validator, IR compiler, HTML renderer, adapters, and native CLI.
- `legacy/typescript-reference`: archived TypeScript reference retained for historical/npm-web context.

## Status

- Standard status: Draft 0.4
- Canonical implementation: Rust
- TypeScript implementation: legacy

## Install

```sh
cargo install --path rust/pagescript-rs
```

During local development:

```sh
cargo run -p pagescript-rs -- validate examples/data-lineage-demo.page
cargo run -p pagescript-rs -- render examples/data-lineage-demo.page > lineage-demo.html
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

## Rust API

```rust
use pagescript_rs::{compile_page_ir, parse_page_script, render_to_html, validate_document};

let document = parse_page_script(source);
let diagnostics = validate_document(&document);
let ir = compile_page_ir(&document, None)?;
let html = render_to_html(&document, None)?;
```

## CLI

```sh
pagescript-rs validate examples/data-lineage-demo.page
pagescript-rs ast examples/data-lineage-demo.page
pagescript-rs ir examples/data-lineage-demo.page
pagescript-rs render examples/data-lineage-demo.page > lineage-demo.html
pagescript-rs convert examples/dashboard.page --target shepherd --tour dashboard-onboarding
```

## Standard Documents

- [SPEC.md](./SPEC.md): normative draft standard
- [RATIONALE.md](./RATIONALE.md): why LLM-native page authoring matters
- [CONFORMANCE.md](./CONFORMANCE.md): parser and validator compatibility expectations
- [schemas/](./schemas): machine-readable AST, IR, and diagnostic schemas
- [conformance/](./conformance): compatibility fixtures
- [docs/agent-workflows.md](./docs/agent-workflows.md): using PageScript as a Cursor/Claude Code/Codex org standard

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

The TypeScript implementation is preserved under `legacy/typescript-reference` and is not part of the active release gate.
