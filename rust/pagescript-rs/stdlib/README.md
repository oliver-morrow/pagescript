# PageScript Draft 0.6 Standard Library

This directory contains reusable PageScript recipe packs embedded into the Rust reference implementation.

## Usage

Import a library in your `.page` file:

```text
::import from="stdlib/product.page"
::/import
```

Then use the recipes:

```text
::use recipe=product-hero title="My Product" subtitle="Best in class"
::/use
```

## Available Libraries

- `product.page`: Marketing and product landing page components.
- `data.page`: Data visualization and dashboard components.
- `docs.page`: Documentation and API reference components.
- `layout.page`: Layout and grid systems.

Imports are compile-time only. Local recipes override imported recipes with the same name, and imported files may recursively import other safe relative paths.
