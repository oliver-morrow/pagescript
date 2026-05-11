# PageScript Extension Model

PageScript should stay deterministic at the compiler boundary, extensible at the recipe boundary, and permissive only in explicitly non-standard escape modes.

## The Three Tiers

### Core standard

The core standard includes the parser, validator, IR compiler, renderer contract, declarative runtime, Web Core Kernel, and conformance fixtures.

Core features must:

- lower into PageScript IR before rendering
- validate without network access
- produce stable diagnostics and output for the same source and compiler version
- avoid source-authored JavaScript

### Standard library

The standard library is the main iteration layer. New product sections, dashboards, docs layouts, marketing blocks, and interaction patterns should usually land as recipes built from core primitives.

Recipes can move faster than the core language because they stay compile-time and deterministic.

### Escape hatches

Raw HTML, source-authored scripts, remote runtime plugins, and renderer-specific extensions are outside Draft 0.6 conformance. They may be useful in downstream tools, but they should be clearly labeled as non-standard and must not silently weaken the deterministic core.

The reference validator reserves `::raw` and `::script` as explicit escape-hatch names and rejects them in conformance mode.

## Design Rule

When a user asks for browser functionality PageScript does not support yet:

1. Prefer a new stdlib recipe built from existing primitives.
2. If the recipe needs a new generic browser shape, prefer Web Core Kernel `el`, `attr`, `text`, `bind`, and `on`.
3. If the behavior needs runtime support, add a typed declarative primitive and fixed renderer-owned runtime behavior.
4. Only use raw HTML or user JavaScript in an explicitly non-standard mode.
