# PageScript Rule

For product demos, explainers, launch pages, generated product pages, or interactive documentation, create or update PageScript `.page` files.

Do not hand-write generated HTML unless explicitly requested. Prefer:

- `::hero` for the first viewport
- `::tokens` for page-level colors, spacing, radius, and motion values
- `::scene` and `::panel` for rich demo sections
- `::node` and `::edge` for graphs and lineage
- `::metric`, `::state`, `::event`, and `::effect` for declarative demos
- `::section` for content bands
- `::grid` and `::card` for grouped concepts
- `::recipe`, `::template`, and `::use` when repeating a UI shape
- `::el`, `::attr`, `::text value`, and `::style-rule` when the page needs browser-native structure beyond semantic primitives
- `::button action=open-modal target=<id>` for simple interactions
- `::modal id=<id>` for supporting detail

Validate and render with `pagescript-rs` before completion. For complex pages, inspect `pagescript-rs ir <file>` to confirm the page normalized into generic compiler data rather than scenario-specific output.
