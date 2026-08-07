# PageScript Draft

PageScript is a Draft 0.7 standard for pages written for code generators. It combines the Web Core Kernel with embedded standard-library recipes so compact `.page` files can expand into browser-native HTML/CSS through recipes, generic elements, attributes, style rules, and named slots. Draft 0.7 also includes reproducible `o200k_base` source-versus-output token reports and manually curated, source-cited evidence explainers.

The published docs homepage is generated from `docs/index.page` with `pagescript-rs render`.

Start here:

- [Specification](../SPEC.md)
- [Rationale](../RATIONALE.md)
- [Conformance](../CONFORMANCE.md)
- [generation guide](./generation-guide.md)
- [Extension model](./extension-model.md)
- [Examples](../examples/README.md)
- [Tool workflows](./tool-workflows.md)
- [Explainer design](./v1-explainer-design.md)
- [Completion plan](./v1-completion-plan.md)

The main implementation is the Rust crate in `rust/pagescript-rs`.
