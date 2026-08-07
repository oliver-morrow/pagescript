# PageScript Rule

When asked to create product demos, launch pages, generated product pages, or interactive documentation, author a `.page` file using PageScript instead of raw HTML/CSS/JS. For source-cited architecture or lineage explainers, author an Evidence Bundle plus Explainer Spec rather than restating evidence in page copy.

Use concise layout primitives, prefer `::tokens` for page-level design values, use Web Core Kernel primitives such as `::recipe`, `::use`, `::el`, `::attr`, and `::style-rule` only when semantic components are not expressive enough, keep interactions declarative, and run:

```sh
pagescript-rs validate <file>
pagescript-rs render <file>
```

For evidence work, validate before rendering:

```sh
pagescript evidence validate <bundle.evidence.json> --json
pagescript explain <bundle.evidence.json> --spec <report.explainer.json> --out <report.html>
```
