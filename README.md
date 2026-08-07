# PageScript

[![Spec](https://img.shields.io/badge/spec-Draft%200.7-blue)](./SPEC.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

PageScript is a Rust toolkit for two closely related jobs:

- compile compact, safe `.page` files to standalone HTML; and
- turn a source-cited Evidence Bundle into a reviewable architecture or lineage explainer.

The explainer path is the v1.1 focus. It keeps structural claims separate from presentation, pins an Explainer Spec to an evidence digest, and renders an offline HTML artifact whose claims retain their source locations.

The main implementation is Rust:

- `rust/pagescript-rs`: active parser, validator, IR compiler, HTML renderer, adapters, and native CLI.
- `legacy/typescript-reference`: archived TypeScript reference retained for historical/npm-web context.

## Status

- Standard status: Draft 0.7 (`v1.1.0-alpha.1`)
- Main implementation: Rust
- TypeScript implementation: legacy
- Evidence input: typed JSON bundle and explainer-spec schemas
- Repository and dbt extraction adapters: planned; not yet a supported public workflow

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

## Reproducible token-savings demo

The flagship benchmark measures the authored source needed to create a real standalone page. `examples/revenue-map-demo.page` is **1,787 `o200k_base` tokens**; its generated standalone HTML is **4,975 tokens**—a **64.08% reduction** in authored artifact tokens.

```sh
cargo run -p pagescript-rs -- stats examples/revenue-map-demo.page --page revenue-map
```

The checked-in [report](./conformance/stats/revenue-map.o200k.json) and [schema](./schemas/token-savings.schema.json) make the number reproducible. The benchmark canonicalizes artifact line endings to LF, and deliberately excludes prompt templates, tool calls, repair turns, and prior context; those require workflow-specific measurement.

## Source-cited explainer

The current evidence workflow accepts a reviewed bundle rather than guessing from a repository. This makes the boundary explicit while the repository and dbt adapters are being built.

```sh
cargo run -p pagescript-rs -- evidence validate conformance/evidence/valid/minimal.evidence.json --json
cargo run -p pagescript-rs -- explain conformance/evidence/valid/minimal.evidence.json \
  --spec conformance/explainer/valid/minimal.explainer.json \
  --out fixture-explainer.html
```

Open `fixture-explainer.html` locally. It is standalone, makes no external requests, and exposes each entity and relationship citation as a local `path:line` reference.

The public contracts are [Evidence Bundle schema](./schemas/evidence-bundle.schema.json), [Explainer Spec schema](./schemas/explainer-spec.schema.json), and the [v1.1 product design](./docs/v1-explainer-design.md).

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

::use recipe=product-hero title="PageScript" subtitle="pages written for code generators"
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
pagescript stats examples/revenue-map-demo.page --page revenue-map
pagescript render examples/web-core-kernel.page > web-core-kernel.html
pagescript convert examples/dashboard.page --target shepherd --tour dashboard-onboarding
pagescript evidence validate conformance/evidence/valid/minimal.evidence.json --json
pagescript explain conformance/evidence/valid/minimal.evidence.json --spec conformance/explainer/valid/minimal.explainer.json --out explainer.html
```

## Standard Documents

- [SPEC.md](./SPEC.md): normative draft standard
- [RATIONALE.md](./RATIONALE.md): why page authoring for code generators matters
- [CONFORMANCE.md](./CONFORMANCE.md): parser and validator compatibility expectations
- [schemas/](./schemas): machine-readable AST, IR, and diagnostic schemas
- [conformance/](./conformance): compatibility fixtures
- [docs/tool-workflows.md](./docs/tool-workflows.md): using PageScript from Cursor, Claude Code, or Codex
- [docs/generation-guide.md](./docs/generation-guide.md): compact syntax guide, small examples, and repair-loop workflow for generation
- [docs/extension-model.md](./docs/extension-model.md): deterministic core, recipe iteration, and escape-hatch policy
- [docs/v1-explainer-design.md](./docs/v1-explainer-design.md): product boundary and evidence lineage model
- [docs/v1-completion-plan.md](./docs/v1-completion-plan.md): staged delivery plan and release gates

## Development

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## CI/CD

GitHub Actions workflows cover Rust verification, conformance smoke tests, crate packaging, docs deployment, tagged releases, and dependency auditing under `.github/workflows/`. The evidence workflow is covered by schema, digest-binding, rendering, and CLI tests.
The GitHub Pages homepage is authored in [docs/index.page](./docs/index.page) and rendered by the docs workflow.

## Releases

Draft 0.7 is not a v1.1 release. The repository currently has build-oriented release workflows, but crates.io publication and GitHub release automation must be rehearsed before a public tag. See the completion plan for the required release gates.

The TypeScript implementation is preserved under `legacy/typescript-reference` and is not part of the active release gate.
