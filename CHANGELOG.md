# Changelog

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
