# pagescript-rs

Rust reference implementation for PageScript Draft 0.7: compact `.page` compilation plus source-cited explainer rendering.

This crate provides:

- parser: `parse_page_script`
- compatibility alias: `parse_tour_script`
- validator: `validate_document`
- IR compiler: `compile_page_ir`
- renderer: `render_to_html`
- reproducible authored-source token report: `measure_token_savings` (`o200k_base`)
- evidence bundle and explainer-spec validation with SHA-256 binding
- source-cited standalone explainer renderer: `render_explainer_to_html`
- adapters: `to_shepherd_config`, `to_intro_config`
- native CLI: `pagescript` with `pagescript-rs` compatibility alias
- embedded PageScript standard-library recipes under `stdlib/`

## CLI

```sh
cargo run --bin pagescript -- guide
cargo run --bin pagescript -- new demo.page
cargo run --bin pagescript -- render demo.page --out index.html
cargo run --bin pagescript -- validate ../../examples/revenue-map-demo.page --json
cargo run --bin pagescript -- ir ../../examples/revenue-map-demo.page --page revenue-map
cargo run --bin pagescript -- stats ../../examples/revenue-map-demo.page --page revenue-map
cargo run --bin pagescript -- convert ../../examples/dashboard.page --target shepherd --tour dashboard-onboarding
cargo run --bin pagescript -- evidence validate ../../conformance/evidence/valid/minimal.evidence.json --json
cargo run --bin pagescript -- explain ../../conformance/evidence/valid/minimal.evidence.json --spec ../../conformance/explainer/valid/minimal.explainer.json --out fixture-explainer.html
```

The token report compares authored `.page` source to this compiler's generated standalone HTML. It does not estimate prompt, tool-call, repair-turn, or previous-context costs.

## Verification

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The conformance tests compare against the repository-level `conformance/` fixtures.
