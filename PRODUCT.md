# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Engineering recruiters, engineers, and developers arriving from a portfolio or GitHub profile. They need to understand what PageScript does, try a real example quickly, and find the implementation when they want to inspect it. Developers can ask a coding agent to write PageScript for them.

## Product Purpose

PageScript gives coding agents a small text format for writing standalone web pages. A visitor can copy a prompt for their own agent, see how example source becomes HTML, and download the result.

## Positioning

The browser uses the same Rust compiler as the CLI through WebAssembly. The example is actual PageScript source, and the preview is generated from that source. An agent skill describes how to author, validate, and render a page using the CLI.

## Operating Context

Visitors can try the editor without installing the CLI or creating an account. They use the skill in their own coding agent. Developers can also read the language reference, explore the GitHub repository, or use the native CLI for local files.

## Capabilities and Constraints

- The browser compiler renders `.page` source with bundled recipes and returns errors with line and column locations.
- The native CLI also resolves imports from disk and renders source-cited explainers.
- The project is an alpha. Public descriptions should show what the current implementation does.
- The deployment is static. The custom domain launches after the design is approved.

## Brand Commitments

Use the PageScript name and plain language. Let the example and the actual compiler demonstrate the work.

## Evidence on Hand

The source and fixtures live in `examples/` and `conformance/`. The Rust implementation lives in `rust/pagescript-rs/`, and the browser bindings live in `rust/pagescript-web/`.
