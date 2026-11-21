# Changelog

## [1.1.0-alpha.2](https://github.com/oliver-morrow/pagescript/releases/tag/v1.1.0-alpha.2) - 2026-11-21

### Added

- flat lists and text tables in Markdown blocks
- `pagescript-web`, the compiler built for the browser with WebAssembly, which runs the playground at pagescript.olivermorrow.com
- a portable agent skill (`skills/pagescript/SKILL.md`) for writing, validating, and rendering `.page` files
- token benchmarks that compare authored PageScript with hand-written HTML for five examples
- `ArtifactTokenMeasure::from_text` for counting an artifact with the stats tokenizer

### Changed

- a new default theme for compiled pages: a cooler palette, smaller headings, tighter spacing, 44px controls, and no card hover lift. Light accent colors get dark button text unless `color.accent-ink` is set
- some generated class names changed, for example `ps-scene-inner` is now `ps-scene-body`; custom CSS that targets the old names needs updating
- `pagescript new` templates use realistic sample content
- the revenue-map output grows from 4,975 to 5,901 tokens with the new theme, so the checked figure moves from 64.08% to 69.72% fewer tokens in the authored source
- the site deploys to Cloudflare from the `prod` branch instead of GitHub Pages

### Removed

- contributor templates, SECURITY.md, CONTRIBUTING.md, RATIONALE.md, the per-tool agent templates (replaced by the skill), and draft-era documentation pages

## [1.1.0-alpha.1](https://github.com/oliver-morrow/pagescript/releases/tag/v1.1.0-alpha.1) - 2026-08-07

### Added

- deterministic `pagescript stats` reports with a checked-in `o200k_base` token-savings fixture
- source-cited Evidence Bundle and Explainer Spec schemas with an offline standalone explainer renderer
- compiler validation for executable URL schemes, style-tag termination, unresolved imports, recipe-expansion cycles, and resolver root boundaries
- release artifacts checksums and release, package, docs, and CI checks for the public demo contracts

### Changed

- public docs now lead with the reproducible revenue-map benchmark: 1,787 authored tokens versus 4,975 standalone-HTML tokens (64.08% reduction)
- GitHub release tags containing a pre-release suffix publish as GitHub pre-releases

### Alpha scope

- Evidence bundles are currently reviewed JSON input. Repository and dbt extraction adapters remain planned for a later release.

## [1.0.0](https://github.com/oliver-morrow/pagescript/releases/tag/v1.0.0) - 2026-05-11

### Added

- add Rust PageScript compiler and CLI

### Fixed

- fix/release version ([#7](https://github.com/oliver-morrow/pagescript/pull/7))

### Other

- Feature/deterministic extension boundaries ([#5](https://github.com/oliver-morrow/pagescript/pull/5))
- Codex/peer useful cli mvp ([#4](https://github.com/oliver-morrow/pagescript/pull/4))
- Harden Draft 0.6 release gates ([#3](https://github.com/oliver-morrow/pagescript/pull/3))
- Pagescript draft 06 standard library ([#2](https://github.com/oliver-morrow/pagescript/pull/2))

## 0.1.0

- Draft 0.1 PageScript standard.
- TypeScript reference parser, validator, adapters, and CLI.
- Shepherd.js and Intro.js adapter output.
- Conformance fixture structure and JSON schemas.
- Rust reference implementation with native CLI and conformance tests.
- Renderable PageScript components and standalone HTML rendering.
- TypeScript implementation moved to `legacy/typescript-reference`; Rust is main.
- Draft 0.5 Web Core Kernel with generic elements, attributes, style rules, recipe expansion, safety validation, and a rendered kernel example.
- Draft 0.6 standard library with embedded product, data, docs, and layout recipe packs.
- Named recipe slots, recursive imports, local recipe override semantics, and IR snapshot conformance coverage.
- Release hardening for public schemas, CLI argument diagnostics, import path safety, and stdlib-heavy rendered smoke tests.
