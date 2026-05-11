# PageScript LLM-Native Web Composition

PageScript is a compact source language for LLM-native web composition. Agents and humans write `.page` files with semantic layout, design tokens, data, interaction, effect, and Web Core Kernel primitives; the compiler emits polished standalone HTML/CSS/SVG with a fixed declarative runtime.

Interactive documentation is one use case. The broader target is product demo pages, architecture explainers, launch pages, onboarding pages, generated product pages, and codebase-aware web experiences.

## File Model

- Canonical extension: `.page`
- Encoding: UTF-8 text
- Output target: standalone HTML through a normalized PageScript IR
- Primary implementation: Rust compiler and CLI
- Interaction model: declarative runtime only; source-authored JavaScript is not allowed
- Design system: declarative tokens through `::tokens`
- Advanced styling: scoped CSS through `::style scope=page|component` and focused `::style-rule`

The canonical parser AST remains generic: `document`, `page`, `component`, `markdown`, and compatibility nodes for `tour`, `step`, and `trigger`.

## Compiler Pipeline

PageScript is not a collection of hardcoded demo templates. A conforming compiler should keep these stages separate:

1. Parse `.page` source into the canonical AST.
2. Validate source-level syntax, required attributes, and duplicate IDs.
3. Normalize the AST into PageScript IR.
4. Render the IR into a target such as standalone HTML/CSS/SVG.
5. Attach only the fixed declarative runtime required by state, events, and effects.

The IR is the compiler boundary. It contains normalized page metadata, design tokens, recipe definitions, layout metadata, component nodes, graph nodes and edges, declared state, events, effects, and scoped CSS. Output renderers should consume IR rather than walking raw source syntax directly.

Draft 0.5 adds a Web Core Kernel. The intent is similar to compressed data transport: `.page` source stays small and semantic for LLM generation, while recipes and kernel primitives act as the decompression key that expands into browser-native HTML/CSS/SVG at render time.

## Deterministic Extension Boundary

PageScript keeps velocity by expanding what can be expressed with recipes and Web Core Kernel primitives, not by accepting arbitrary browser code in the core language.

Conforming implementations must preserve this deterministic contract:

- same source, compiler version, and imported recipe versions produce the same AST, diagnostics, IR, and rendered output
- compilers must not perform network access during parse, validation, IR compilation, or rendering
- generated IDs, ordering, diagnostics, and emitted runtime hooks must be stable
- source-authored JavaScript is not part of Draft 0.6
- every conforming feature must lower into typed IR before rendering

Extension tiers:

- Core standard: deterministic primitives, declarative interactions, Web Core Kernel, validation, IR, and rendering.
- Standard library: fast-moving recipes built from core primitives and imported at compile time.
- Escape hatches: raw HTML, source-authored scripts, remote runtime plugins, or renderer-specific extensions. These are outside the deterministic core and must be rejected by conforming validators unless an implementation explicitly offers a non-standard mode.

## Core Web Composition Example

```text
::page id=lineage-demo title="Data Lineage Demo"
  ::state id=selectedNode default=warehouse
  ::/state

  ::event on=node.click set=selectedNode value="$node.id"
  ::/event

  ::effect id=flow type=flow speed=medium
  ::/effect

  ::tokens
    color.accent="#4dd6a0"
    color.bg="#080f0c"
    radius.panel=14
  ::/tokens

  ::scene id=lineage layout=split title="Live Data Lineage"
    ::panel id=pipeline title="Pipeline graph"
      ::node id=source label="Stripe Events" status=active x=90 y=90
      ::/node
      ::node id=warehouse label="Snowflake" status=syncing x=290 y=165
      ::/node
      ::edge from=source to=warehouse effect=flow
      ::/edge
    ::/panel

    ::panel id=live title="Live updates"
      ::metric id=rows label="Rows synced" value="128,932" tone=good
      ::/metric
      ::log id=lineage-log source=lineage-events max=4
      ::/log
    ::/panel
  ::/scene
::/page
```

## Syntax

Directives begin with `::` and use explicit closing tags:

```text
::component key=value
::/component
```

Directive headers and field lines use `key=value` attributes. Supported values are bare strings, quoted strings, booleans, numbers, and JSON object values.

Attribute keys may contain letters, numbers, `_`, `-`, and `.`. Dotted keys are intended for token namespaces such as `color.accent` and `radius.panel`.

## General Page Primitives

- `::page`: root page
- `::hero`: first viewport
- `::section`: content band
- `::stack`: vertical grouping
- `::grid`: responsive grid
- `::card`: content panel
- `::button`: declarative action trigger
- `::text`: text group
- `::image`: image with alt text
- `::modal`: declarative dialog
- `::form` and `::input`: basic form primitives

## Web Composition Primitives

- `::scene id layout=split|full|canvas title`: visual demo section
- `::panel id title tone`: bounded UI region inside a scene
- `::node id label status icon x y`: graph or flow node
- `::edge from to effect`: graph edge rendered with SVG
- `::metric id label value tone effect`: live-style data display
- `::log id source max`: event log placeholder
- `::state id default`: declarative state slot
- `::event on set value`: state transition rule
- `::effect id type speed duration`: reusable visual effect
- `::style scope=page|component`: scoped CSS escape hatch
- `::tokens`: page-level design tokens

Allowed effect types are `flow`, `pulse`, `glow`, `count-up`, and `reveal`.

## Standard Library

PageScript standard libraries are `.page` files containing reusable `recipe` definitions.
They are imported with `::import from="..."`.
Imports are compile-time only.

- `::import from=<path>`: load recipes from another `.page` file

Implementation rules:
- `from` must be a relative path.
- Absolute paths and paths containing `..` must be rejected for safety.
- Imported recipes are merged into the current page recipe context.
- Local recipes override imported recipes with the same name.
- Imports are recursive; imported files may themselves import other files.

## Web Core Kernel

The Web Core Kernel gives PageScript native browser reach without allowing source-authored JavaScript:

- `::el tag=<html-tag>`: generic browser element
- `::attr name=<attr> value=<value>`: child attribute for `el`
- `::text value=<text>`: raw escaped text node
- `::style-rule selector=<css-selector>`: focused CSS rule body
- `::recipe name=<name>`: reusable expansion unit
- `::template`: recipe body
- `::use recipe=<name>`: compile-time recipe invocation
- `::slot name=<name>`: named or default expansion slot
- `::bind state=<id>`: declarative binding hook
- `::on event=<event> action=<action>`: declarative event hook

Recipes are expanded into IR before rendering. For example:

```text
::recipe name=link-card
  ::template
    ::el tag=a class="card" href="$href"
      ::attr name=aria-label value="$title"
      ::/attr
      ::text value="$title"
      ::/text
    ::/el
  ::/template
::/recipe

::use recipe=link-card title="Spec" href="./SPEC.md"
::/use
```

The compiler substitutes `$title` and `$href`, then renders an ordinary HTML anchor.

### Named Slots

Draft 0.6 introduces named slots for recipe composition:

```text
::recipe name=product-hero
  ::template
    ::hero
      ::el tag=h1
        ::text value=$title
        ::/text
      ::/el
      ::slot name=actions
      ::/slot
    ::/hero
  ::/template
::/recipe
```

Usage with named slots:

```text
::use recipe=product-hero title="PageScript"
  ::slot name=actions
    ::button label="Get Started"
    ::/button
  ::/slot
::/use
```

Compiler behavior:
- Replace `::slot name=<name>` inside the recipe template with matching slot children from the `::use`.
- If no matching slot exists in the `::use`, render the recipe slot's default children.
- If a `::use` provides children without a `::slot` wrapper, they are treated as the content for the unnamed (default) slot.

Validators must reject unsafe element tags such as `script`, `iframe`, `object`, and `embed`, and unsafe attributes such as `onclick` and `srcdoc`.

`::raw` and `::script` are reserved escape-hatch names. They are intentionally outside the Draft 0.6 deterministic core and must produce a validation diagnostic in conformance mode.

## Design Tokens

Tokens are compiler-readable design inputs, not raw CSS. Renderers may map known tokens to CSS variables and preserve unknown tokens under implementation-specific names.

```text
::tokens
  color.bg="#080f0c"
  color.ink="#f7fff9"
  color.accent="#4dd6a0"
  radius.panel=14
::/tokens
```

Draft 0.5 defines these common aliases:

- `color.bg` -> page background
- `color.ink` -> primary text
- `color.muted` -> muted text
- `color.line` -> borders
- `color.accent` -> primary accent
- `color.accent-ink` -> text on accent
- `color.panel` -> panel background
- `radius.panel` -> panel radius

Numeric `radius.*` and `spacing.*` tokens are interpreted as pixels by the reference renderer.

## Layout Metadata

Renderable components may carry generic layout metadata:

- `layout`: layout mode, such as `split`, `full`, or `canvas`
- `density`: `compact`, default, or `spacious`
- `gap`: `sm`, `md`, or `lg`
- `columns`: grid column count
- `align`: renderer-defined alignment hint

The compiler normalizes these attributes into IR so future renderers can target HTML, React, Web Components, or native UI without re-parsing source syntax.

## Declarative Runtime

The compiler emits a fixed runtime that can:

- open modals from `button action=open-modal target=<id>`
- toggle elements from `button action=toggle target=<id>`
- update declared state from events such as `event on=node.click set=selectedNode value="$node.id"`
- activate CSS/SVG effects declared by `effect=<id>`

`.page` files do not contain raw JavaScript.

## Scoped CSS

Scoped CSS is an advanced escape hatch:

```text
::style scope=page
  .lineage-hero { background: radial-gradient(circle, #dff8ef, transparent 28%); }
::/style
```

Renderers inject scoped style text into the compiled document. Validators must reject unsupported scopes.

## Validation

Conforming validators report structured diagnostics for malformed directives, unknown directives, mismatched closing tags, unclosed blocks, missing required attributes, duplicate page/scene/node/state/effect/recipe IDs or names, invalid effect types, invalid style scopes, invalid token values, unsafe Web Core Kernel tags or attributes, deterministic-core escape hatches, unknown recipes, and compatibility-tour errors.

## Org-Level Agent Workflow

1. Cursor, Claude Code, Codex, or humans author `.page` files.
2. Humans review semantic web composition instead of generated HTML/CSS/JS.
3. CI runs `pagescript-rs validate`.
4. CI or review can inspect `pagescript-rs ir`.
5. Publishing runs `pagescript-rs render`.

This creates a shared source format for LLM-generated product demos, explainers, interactive docs, and codebase-aware web pages.
