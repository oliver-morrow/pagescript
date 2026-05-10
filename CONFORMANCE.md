# PageScript Conformance

This document defines what it means for an implementation to conform to the PageScript draft standard.

## Parser Requirements

A conforming parser must:

- Accept UTF-8 `.page` documents and compatible `.tour` documents.
- Preserve Markdown as raw `markdown` nodes outside blocks and inside `page` and `tour` blocks.
- Preserve step Markdown as the `markdown` field on `step` nodes.
- Parse explicit block directives: `page`, renderable components, and compatibility `tour`, `step`, and `trigger` blocks.
- Require explicit closing tags for every block.
- Parse attribute values as strings, quoted strings, booleans, numbers, or JSON objects.
- Accept dotted attribute keys for namespaced tokens.
- Return a document AST even when diagnostics are present.

## Validator Requirements

A conforming validator must report structured diagnostics for:

- malformed directives
- unknown directives
- mismatched closing directives
- unexpected closing directives
- unclosed blocks
- missing required `page.id`
- missing required `tour.id`
- missing required `step.id`
- missing required `step.target`
- missing required `trigger.type`
- missing required renderable component attributes
- invalid token values
- invalid effect types and style scopes
- unsafe Web Core Kernel tags and attributes
- duplicate scene, node, state, effect, and recipe names within a page
- unknown recipe references
- invalid `import` paths (absolute or containing `..`)
- duplicate page IDs in a document
- duplicate tour IDs in a document
- duplicate step IDs within a tour

## Fixture Contract

The `conformance/` directory is the compatibility contract:

- `conformance/valid/*.page` files must parse and validate without diagnostics.
- `conformance/valid/*.ast.json` files define expected AST output.
- `conformance/invalid/*.page` files must parse and return diagnostics.
- `conformance/invalid/*.diagnostics.json` files define expected diagnostics.

Implementations in other languages should pass the fixture suite before claiming Draft 0.6 support.

## Compiler IR Requirements

A conforming compiler should normalize validated AST into PageScript IR before rendering. The IR must preserve:

- page ID and title
- page-level design tokens
- recipe definitions as the compile-time expansion contract
- imported recipe resolution
- named slot replacement during expansion
- normalized component layout metadata
- renderable component tree
- Markdown content
- graph nodes and edges as data, not pre-rendered SVG strings
- declared state, events, effects, and scoped CSS

The reference CLI exposes this boundary through `pagescript-rs ir <file> [--page id]`. Future conformance fixtures may include expected IR snapshots alongside AST and diagnostic snapshots.

`schemas/page-ir.schema.json` defines the current machine-readable IR shape.

## Reference Implementations

- Canonical: `rust/pagescript-rs`
- Legacy: `legacy/typescript-reference`

The Rust implementation must pass the conformance fixtures before release. The TypeScript implementation is archived and may lag behind.

## Render Requirements

A conforming Draft 0.6 renderer must compile PageScript IR to standalone HTML/CSS/SVG and support:

- scene, panel, metric, log, graph node, and graph edge primitives
- Web Core Kernel `el`, `attr`, `text value`, `style-rule`, `recipe`, `template`, `use`, `slot name`, `bind`, and `on`
- compile-time recipe expansion before rendering, including recursive imports and named slot substitution
- design token aliases for core colors and panel radius
- layout metadata for scene mode, grid columns, gap, and density
- `button action=open-modal target=<id>` for `modal id=<id>`
- `button action=toggle target=<id>`
- `event on=node.click set=<state> value="$node.id"`
- fixed runtime JavaScript emitted by the compiler, with no source-authored JavaScript in `.page` files
