# Compiler tests

Run the fixture tests with `cargo test -p pagescript-rs --test conformance --locked` or run the full suite with `cargo test --workspace --locked`.

The [language reference](SPEC.md) describes the syntax. The `conformance/` directory records what the compiler should produce for specific inputs.

- [Valid input](conformance/valid/) pairs `.page` and `.tour` files with expected parser or compiler output.
- [Invalid input](conformance/invalid/) pairs rejected source with expected messages and source locations.
- [Evidence](conformance/evidence/) and [explainer](conformance/explainer/) fixtures cover the JSON inputs for source-cited diagrams.
- The [revenue-map report](conformance/stats/revenue-map.o200k.json) records the expected token counts for a shipped example.

Run the relevant fixture when changing syntax, diagnostics, or rendering. Update its expected output when the behavior changes intentionally.
