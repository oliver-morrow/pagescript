# Browser compiler

Run `bash scripts/build-site.sh` from the repository root with Rust, wasm-pack, Node.js, and jq installed. It builds the site in `_site/` and checks the browser compiler's output against the CLI.

A page served from `playground/` can load the compiled module as follows.

```js
import initialize, { compile_page } from "./compiler/pagescript.js";

await initialize();
const result = JSON.parse(compile_page(source, pageId));
```

Pass a page ID to select a page from a multi-page input. Leave it undefined to render the first page. The result has one of three shapes.

- `{ status: "rendered", html }` contains the complete HTML document.
- `{ status: "invalid", diagnostics }` contains parser and validator messages with line and column numbers.
- `{ status: "render_error", message }` describes a rendering error without a source position.

Preview the generated document in a sandboxed iframe without same-origin access. The browser compiler resolves built-in recipe imports. Use the CLI for imports from disk and for the evidence explainer.
