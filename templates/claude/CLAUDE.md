# PageScript Rule

When asked to create product demos, explainers, launch pages, generated product pages, or interactive documentation, author a `.page` file using PageScript instead of raw HTML/CSS/JS.

Use concise layout primitives, prefer `::tokens` for page-level design values, use Web Core Kernel primitives such as `::recipe`, `::use`, `::el`, `::attr`, and `::style-rule` only when semantic components are not expressive enough, keep interactions declarative, and run:

```sh
pagescript-rs validate <file>
pagescript-rs render <file>
```
