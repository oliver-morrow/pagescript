# PageScript for Agent Workflows

PageScript is intended to be an org-level standard for LLM-native web composition authored by AI coding tools and humans.

## Why It Fits Cursor, Claude Code, and Codex

- Agents can write concise composition primitives instead of verbose HTML/CSS/JS.
- Humans can review page intent, structure, copy, and interactions in one small file.
- CI can validate `.page` files, inspect compiler IR, and render standalone HTML.
- The same source works across tools because the standard is file-based, not editor-specific.

## Recommended Repository Pattern

```text
docs/
  product.page
  onboarding.page
  architecture.page
```

Each `.page` file should live next to the feature or docs area it explains. Generated HTML should be treated as build output unless the publishing system requires checked-in artifacts.

## Agent Instruction

Tell agents:

```text
When creating product demos, explainers, launch pages, or interactive documentation, write PageScript `.page` files.
Use high-level primitives like ::tokens, ::scene, ::panel, ::node, ::edge, ::metric, ::effect, ::hero, ::grid, and ::card.
Prefer stdlib recipes and Web Core Kernel primitives when the page needs new structure.
Do not hand-write raw HTML or JavaScript in conforming PageScript; propose a recipe or typed declarative primitive instead.
Run `pagescript-rs validate`, inspect `pagescript-rs ir` for complex pages, and run `pagescript-rs render` before finishing.
```

## CI Gate

```sh
pagescript-rs validate docs/product.page
pagescript-rs ir docs/product.page
pagescript-rs render docs/product.page > public/product.html
```
