# Generation Guide

PageScript works for generation only if the model gets local examples and compiler feedback. Do not assume the model knows `.page` syntax from training data.

Use this compact guide in generation prompts or run `pagescript guide`.

## Compact Syntax Guide

- Blocks start with `::name key=value` and end with `::/name`.
- Attribute values are bare strings, quoted strings, booleans, numbers, or JSON objects.
- Keep nesting predictable: `page` contains semantic components; layout components contain content components.
- Prefer semantic primitives over HTML aliases: `hero`, `section`, `nav`, `form`, `filter`, `table`, `empty-state`, `scene`, `panel`, `metric`, `card`.
- Use `recipe` and `use` for reusable patterns.
- Use `el`, `attr`, and `text` only for generic browser shapes that do not deserve a semantic primitive yet.
- Do not emit raw HTML or JavaScript. `raw` and `script` are rejected in conformance mode.

## Main Examples

Minimal page:

```text
::page id=demo title="Demo"
  ::hero heading="Useful page" body="Compact source, deterministic HTML."
  ::/hero
::/page
```

Dashboard table:

```text
::page id=ops title="Ops"
  ::nav label="Primary"
    ::nav-item label="Overview" href="#overview"
    ::/nav-item
  ::/nav

  ::section id=overview heading="Accounts"
    ::filter id=account-search label="Search accounts" placeholder="Company or owner"
    ::/filter
    ::table id=accounts
      ::column label="Account"
      ::/column
      ::column label="Status"
      ::/column
      ::row
        ::cell value="Acme"
        ::/cell
        ::cell value="Healthy"
        ::/cell
      ::/row
    ::/table
  ::/section
::/page
```

## Repair Loop

1. Generate a `.page` file from the task and the guide above.
2. Run `pagescript validate file.page --json`.
3. Fix the first diagnostic by line number.
4. Repeat until diagnostics are `[]`.
5. Run `pagescript render file.page --out index.html`.

## Audit Question

Ask this for every new primitive or recipe:

> Is PageScript compressing intent into higher-level, validated UI semantics, or is it just making the model learn a new spelling of HTML?

If it is only a new spelling of HTML, prefer a semantic primitive or recipe instead.
