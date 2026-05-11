# Early Product Notes

This file is kept as background on the original guided-tour idea.

The first version of PageScript focused on compact tour files that could compile to Shepherd.js or Intro.js configs. That work is still supported through `.tour` compatibility fixtures and the `pagescript convert` command.

The current project is broader: `.page` files describe small web pages, demos, docs, graphs, metrics, forms, and simple interactions, then compile to standalone HTML. The tour format is now one compatibility path, not the main product.

Current source of truth:

- [SPEC.md](./SPEC.md)
- [CONFORMANCE.md](./CONFORMANCE.md)
- [docs/generation-guide.md](./docs/generation-guide.md)
- [docs/extension-model.md](./docs/extension-model.md)
