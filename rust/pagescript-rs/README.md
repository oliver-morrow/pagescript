# pagescript-rs

Rust reference implementation for the PageScript Draft 0.6 LLM-native web composition standard.

This crate provides:

- parser: `parse_page_script`
- compatibility alias: `parse_tour_script`
- validator: `validate_document`
- IR compiler: `compile_page_ir`
- renderer: `render_to_html`
- adapters: `to_shepherd_config`, `to_intro_config`
- native CLI: `pagescript` with `pagescript-rs` compatibility alias
- embedded Draft 0.6 standard library recipes under `stdlib/`

## CLI

```sh
cargo run --bin pagescript -- guide
cargo run --bin pagescript -- new demo.page
cargo run --bin pagescript -- render demo.page --out index.html
cargo run --bin pagescript -- validate ../../examples/autonomous-revenue-command-center.page --json
cargo run --bin pagescript -- ir ../../examples/autonomous-revenue-command-center.page --page revenue-command-center
cargo run --bin pagescript -- convert ../../examples/dashboard.page --target shepherd --tour dashboard-onboarding
```

## Verification

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The conformance tests compare against the repository-level `conformance/` fixtures.
