# PageScript

[![Spec](https://img.shields.io/badge/spec-Draft%200.1-blue)](./SPEC.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

PageScript is a draft open standard for LLM-native interactive pages. It lets AI coding tools and humans write compact `.page` files using layout, style, and interaction primitives, then compile them into standalone HTML.

The canonical implementation is Rust:

- `rust/pagescript-rs`: active parser, validator, renderer, adapters, and native CLI.
- `legacy/typescript-reference`: archived TypeScript reference retained for historical/npm-web context.

## Status

- Standard status: Draft 0.1
- Canonical implementation: Rust
- TypeScript implementation: legacy

## Install

```sh
cargo install --path rust/pagescript-rs
```

During local development:

```sh
cargo run -p pagescript-rs -- validate examples/interactive-doc.page
cargo run -p pagescript-rs -- render examples/interactive-doc.page > interactive-doc.html
```

## Example

```text
::page id=agent-docs title="Agent Workflow Docs"
  ::hero tone=dark spacing=xl
    heading="Interactive docs your AI tools can write"
    body="Use compact primitives instead of verbose HTML and CSS."
    ::button label="See workflow" action=open-modal target=workflow
    ::/button
  ::/hero

  ::section spacing=lg
    heading="Use it across your org"
    ::grid columns=3 gap=md
      ::card title="Author" body="Agents and humans write the same source."
      ::/card
      ::card title="Review" body="Pull requests show intent without markup noise."
      ::/card
      ::card title="Publish" body="CI validates and renders HTML."
      ::/card
    ::/grid
  ::/section

  ::modal id=workflow heading="Org workflow"
    body="Set one repo rule and ask agents to update .page files."
  ::/modal
::/page
```

## Rust API

```rust
use pagescript_rs::{parse_page_script, render_to_html, validate_document};

let document = parse_page_script(source);
let diagnostics = validate_document(&document);
let html = render_to_html(&document, None)?;
```

## CLI

```sh
pagescript-rs validate examples/interactive-doc.page
pagescript-rs ast examples/interactive-doc.page
pagescript-rs render examples/interactive-doc.page > interactive-doc.html
pagescript-rs convert examples/dashboard.page --target shepherd --tour dashboard-onboarding
```

## Standard Documents

- [SPEC.md](./SPEC.md): normative draft standard
- [RATIONALE.md](./RATIONALE.md): why LLM-native page authoring matters
- [CONFORMANCE.md](./CONFORMANCE.md): parser and validator compatibility expectations
- [schemas/](./schemas): machine-readable AST and diagnostic schemas
- [conformance/](./conformance): compatibility fixtures
- [docs/agent-workflows.md](./docs/agent-workflows.md): using PageScript as a Cursor/Claude Code/Codex org standard

## Development

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The TypeScript implementation is preserved under `legacy/typescript-reference` and is not part of the active release gate.
