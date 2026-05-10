# PageScript Standard Library

This directory contains reusable PageScript recipe packs.

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
