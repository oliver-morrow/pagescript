---
name: pagescript
description: Write and render PageScript .page files to produce standalone HTML. Use when the user asks for PageScript, supplies a .page file, or asks to turn a brief into a PageScript page.
license: MIT
---

# PageScript

Keep the editable source in a `.page` file. Use PageScript's compiler to produce the HTML.

1. Read the project's existing examples or run `pagescript guide` for the syntax supported by the installed compiler. Start with a small page that serves the brief.
2. Open each block with `::name key=value` and close it with `::/name`. Give the page an ID. Quote attribute values that contain spaces. Prefer named components such as `hero`, `section`, `grid`, and `card`.
3. Run `pagescript validate page.page --json`. Fix diagnostics at their reported lines, then run validation again. Render with `pagescript render page.page --out page.html`. Use `--page <id>` to select a page from a multi-page file.
4. Return the source and the rendered HTML. If the CLI is unavailable, return the source and tell the user it still needs to be rendered. The browser playground can render a single pasted page with bundled recipes.

```text
::page id=welcome title="Welcome"
  ::hero heading="Welcome" body="This page was written as PageScript."
  ::/hero
::/page
```

Use declared components and interactions in place of raw HTML or source-authored scripts. Consult the [language reference](https://github.com/oliver-morrow/pagescript/blob/main/SPEC.md) for layouts, styling, and imports.
