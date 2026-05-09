# Contributing

PageScript is draft-stage standard work. Contributions should distinguish between standard changes and reference implementation changes.

## Development

```sh
pnpm install
pnpm verify
```

## Standard Changes

Changes to syntax, AST shape, diagnostics, or validation rules must update:

- `SPEC.md`
- `CONFORMANCE.md`
- JSON schemas
- conformance fixtures
- tests

## Implementation Changes

Reference implementation changes should keep runtime dependencies at zero unless a standards requirement makes that impossible.
