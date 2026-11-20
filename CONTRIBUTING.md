# Contributing

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Update the language reference, schemas, and compiler fixtures when their behavior changes. Run the site build when changing the playground. [Browser compiler guide](docs/browser-compiler.md)
