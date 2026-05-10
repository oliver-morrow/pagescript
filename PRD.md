# PRD: LLM‑Native Demo Script Format for Interactive HTML Walkthroughs

## 1. Overview

This document specifies a new plain‑text format for authoring **interactive product demos and walkthroughs** that can both be consumed efficiently by large language models (LLMs) and rendered as guided tours on real HTML pages via a small JavaScript runtime. The format acts as a single source of truth for demo narratives, step sequences, UI targets, and layout components, compiling down to existing tour libraries (e.g., Shepherd.js, Intro.js) or custom in‑app overlays.[1][2][3][4][5][6]

## 2. Problem Statement

Today, interactive walkthroughs are typically defined via ad‑hoc JavaScript configs (arrays of steps) tightly coupled to specific libraries and DOM structures. Separately, documentation and AI‑facing content live in Markdown or HTML, causing duplication, drift, and high token costs when used in LLM prompts.[7][2][8][4][5][9][10][1]

Key problems:

- **Split representations:** Narrative docs (Markdown) and tour configs (JS/JSON) diverge over time, making demos stale or misleading.[8][4][1]
- **Token inefficiency:** Raw HTML or JS configs for tours are noisy and expensive to feed into LLMs compared to clean, structured text formats.[11][12][4][1]
- **Tool lock‑in:** Tour definitions are usually coupled to one library (e.g., Intro.js, Shepherd.js) with incompatible config structures.[2][3][5][13]
- **Poor introspection:** Existing formats lack explicit semantics for demo flows, personas, prerequisites, and variants, limiting automated analysis and generation by LLMs.[14][15][7]

There is no standard, LLM‑native way to express interactive walkthroughs that simultaneously serves as:

1. A compact, readable textual spec.
2. A source for HTML/JS tour runtimes.
3. An efficient prompt or context format for LLMs.

## 3. Product Vision

The proposed format—provisionally **TourScript (TSX)**—is a **demo script language** that blends Markdown‑style text with concise, declarative blocks describing tours, steps, and UI bindings. It is designed:

- To be **authored by humans and LLMs** alike, with predictable patterns and low syntactic noise.[9][1][7]
- To **compile into concrete tour configs** for popular JS libraries (Shepherd.js, Intro.js) and custom runtimes.[3][5][6][2]
- To be **token‑efficient** enough to use directly in prompts, RAG corpora, and agent responses.[12][4][11][1]

In the long term, TSX should be a **standard format for interactive walkthroughs**, analogous to how Markdown is a standard for lightweight docs but extended with first‑class support for tours, flows, and UI targets.

## 4. Goals and Non‑Goals

### 4.1 Goals

- **Single source of truth:** Represent narrative, steps, and UI bindings in one text file that drives both docs and live tours.[4][1][7]
- **Token‑efficient for LLMs:** Achieve significantly fewer tokens than equivalent HTML / JS configs while retaining explicit tour structure.[11][12][1]
- **Runtime‑agnostic:** Define an abstract tour model and syntax that can be compiled to multiple JS libraries (Shepherd.js, Intro.js, etc.) and custom renderers.[5][10][2][3]
- **LLM‑friendly syntax:** Use a small, regular set of constructs that models can learn and reproduce reliably from examples.[16][1][7]
- **Deterministic parsing:** Provide an unambiguous grammar that maps to a well‑defined AST for tooling and validation.[17][18][19]

### 4.2 Non‑Goals

- **Full HTML replacement:** TSX is not a general‑purpose HTML substitute; it focuses on demos, flows, and related narrative content.
- **Generic scripting:** TSX will not embed arbitrary JavaScript or programming logic; behavior is declarative and limited to what tour runtimes can support.
- **Visual theming:** Detailed styling is out of scope; theming is handled by the consuming runtime or application.

## 5. Target Users and Use Cases

### 5.1 Target users

- **Product and UX teams** authoring onboarding flows, feature tours, and in‑app guides.[10][2][8]
- **Developer experience and docs teams** who want demos that stay in sync with documentation and API guides.[1][7][4]
- **LLM app builders** who want models to propose or modify walkthroughs directly in a standard format.

### 5.2 Core use cases

- **Onboarding tours:** Step‑by‑step overlays guiding new users through key features, with per‑persona variants.
- **Feature walkthroughs:** Contextual tours triggered when users visit specific pages or enable new modules.
- **Interactive documentation:** Docs pages embedding runnable examples and guided flows, powered from the same TSX spec.[7][4][1]
- **AI‑generated tours:** LLMs propose or update tours based on product changes, using TSX as both input and output format.

## 6. High‑Level Format Design

### 6.1 Design principles

1. **Markdown‑first:** Use standard Markdown for headings, paragraphs, lists, and basic code, so docs are readable out of the box.[9][1][7]
2. **Block components for tours:** Introduce a compact `::directive` syntax for tours, steps, flows, and bindings.
3. **Attribute‑light:** Use concise key/value attributes; avoid HTML‑style verbosity and inline CSS.[12][11]
4. **Separation of content and wiring:** Keep human‑facing text inside Markdown and step bodies; keep selectors and runtime hints as attributes.
5. **AST‑centric:** Define a clear schema for `Tour`, `Step`, `Flow`, `Trigger`, etc., minimizing ambiguity.[18][19][17]

### 6.2 Core building blocks

- **Tour block:** Encapsulates a named tour.
- **Step block:** Child of a tour, bound to a target element.
- **Flow / variant:** Optional grouping for persona or scenario‑specific subsets.
- **Trigger block:** Specifies when a tour should start (URL pattern, event, user state).

Example (illustrative):

```text
# Dashboard Onboarding

::tour id=dashboard-onboarding library=shepherd
  title="Dashboard Onboarding"
  description="Guides new users through key metrics and filters."

  ::trigger type=url pattern="/dashboard" autoStart=true
  ::/trigger

  ::step id=welcome target=".hero" position=bottom
  title="Welcome to your dashboard"
  body="This is where you see your live metrics."
  ::/step

  ::step id=filters target="#filter-panel" position=right
  title="Filter your data"
  body="Use these filters to narrow what you see."
  ::/step

::/tour
```

This is valid TSX plus Markdown: humans can read it easily; a parser can map it to an AST; a runtime can generate Shepherd.js or Intro.js configs.[2][3][5]

## 7. Syntax Specification (Proposed)

> Note: syntax details are illustrative and should be validated via prototypes, tokenizer analysis, and LLM experiments.

### 7.1 Tours

```text
::tour id=<id> [library=<name>] [group=<group>] [variant=<variant>]
  [title="..."]
  [description="..."]
  [options={...}]
  ... steps, triggers, narrative ...
::/tour
```

Attributes:

- `id` (required): stable identifier.
- `library` (optional): target runtime (e.g., `shepherd`, `intro`, `custom`).
- `group` / `variant`: grouping for A/B tests, personas, or product tiers.
- `options`: JSON‑like object for library‑specific config (e.g., default step options).

### 7.2 Steps

```text
::step id=<id> target="ss-selector>" [position=<pos>] [order=<n>]
  [when=dition>] [options={...}]
  title="..."
  body="..."
::/step
```

Attributes:

- `target`: CSS selector or element reference; used by runtime to attach overlays.[3][5][2]
- `position`: position relative to target (e.g., `top`, `bottom`, `left`, `right`, `auto`).
- `when`: optional condition (e.g., feature flag, user role, URL pattern key).
- `options`: library‑specific step options (e.g., `scrollTo`, `modal`, etc.).

The `title` and `body` fields may be inline or encoded as Markdown paragraphs inside the block; runtimes can treat them as HTML after Markdown rendering.

### 7.3 Triggers

```text
::trigger type=url pattern="/dashboard" autoStart=true
::/trigger
```

Supported trigger types may include `url`, `event`, `manual`, `feature-enabled`, etc. These map to runtime hooks in the host application.

### 7.4 Flows and variants

To support more complex scenarios, TSX can group steps into flows:

```text
::flow id=admin-only variant="admin" label="Admin flow"
  ::step ...
  ::step ...
::/flow
```

Runtimes can select flows based on user context, while LLMs can reason over flows to propose alternative sequences.

### 7.5 Frontmatter

TSX supports optional YAML/JSON frontmatter for metadata:

```text
---
product: "Analytics Dashboard"
version: "1.2"
owner: "Growth Team"
---
```

This metadata can be used for governance, search, and analytics.

## 8. AST and Data Model

### 8.1 Node types

A reference schema might include:

- `Document`
- `Tour` (fields: `id`, `title`, `description`, `options`, `children`)
- `Trigger` (fields: `type`, `pattern`, `autoStart`, `options`)
- `Step` (fields: `id`, `target`, `position`, `order`, `when`, `title`, `body`, `options`)
- `Flow` (fields: `id`, `variant`, `label`, `children`)
- Markdown nodes (`Heading`, `Paragraph`, `List`, etc.)

### 8.2 Runtime mapping

Reference mappers should convert the AST to:

- **Shepherd.js config**: array of step objects with `id`, `attachTo`, `title`, `text`, `when`, plus tour‑level defaults.[20][6][5]
- **Intro.js config**: array of step definitions with `element`, `intro`, `position`, and options.[10][3]
- **Custom runtime config**: JSON models for in‑house tour engines.

## 9. LLM and Token Efficiency Considerations

### 9.1 Token savings vs HTML/JS

By using compact `::` blocks and key/value attributes, TSX avoids the verbosity of full HTML tags and JS object syntax while still capturing the same semantics. Existing work on converting HTML to Markdown for LLMs suggests reductions of 20–80% in token counts depending on page complexity; similar savings are expected when replacing HTML/JS tour definitions with TSX.[21][22][11][12][1]

### 9.2 LLM‑friendly structure

- TSX uses consistent patterns (`::tour`, `::step`, `::trigger`) that models can imitate from few‑shot examples.[16][1][7]
- It avoids complex nesting and mixed concerns (e.g., scripts inside HTML), making parsing and validation simpler.[19][17][18]
- Human‑readable text stays in Markdown, which is a known sweet spot for LLM comprehension and generation.[1][7][9]

### 9.3 Structured outputs

TSX documents can be converted automatically to JSON ASTs that are suitable for strict structured outputs APIs, enabling tools to validate tours before execution.[17][18][19]

## 10. Security and Safety

- TSX does not permit arbitrary JavaScript or executable code; runtimes must treat it as declarative configuration only.
- URL fields, selectors, and text should be sanitized by consuming applications.
- When generating TSX via LLMs, guardrails must ensure selectors and triggers cannot target security‑sensitive UI elements without review.

## 11. Ecosystem and Interoperability

### 11.1 Interop with existing docs

- TSX can be embedded into Markdown docs, or entire docs can be written as TSX files that render to static sites using SSGs that already support Markdown.[15][4][7]
- Existing content can be progressively enhanced: narrative docs first, tours added later as `::tour` blocks.

### 11.2 Interop with tour libraries

Adapter packages should be provided for:

- Shepherd.js (JS/TS package that reads TSX → Shepherd config).[6][20][5]
- Intro.js (TSX → Intro steps).[3][10]
- At least one React‑based tour library to demonstrate front‑end framework integration.[2][10]

### 11.3 Interop with AI tooling

- CLI tools and SDKs to validate and pretty‑print TSX.
- Converters to/from JSON for structured output workflows.[18][19][17]
- Support in documentation generators and AI‑assisted authoring tools.[23][4][7]

## 12. Milestones

### 12.1 Phase 1 – Prototype (0–2 months)

- Define minimal TSX grammar (tours, steps, triggers).
- Implement reference parser and AST in TypeScript.
- Implement mappers to Shepherd.js and Intro.js for core use cases.[5][6][2][3]
- Build sample tours and run token‑count benchmarks vs equivalent HTML/JS configs.[11][12]

### 12.2 Phase 2 – Beta (2–4 months)

- Expand syntax to include flows, variants, and metadata.
- Publish a CLI for validation, preview, and conversion.
- Integrate TSX into one or two real product onboarding flows.
- Gather feedback from LLM‑assisted authoring workflows.

### 12.3 Phase 3 – Standardization (4–8 months)

- Publish full specification and reference implementations.
- Document best practices for LLM prompting and generation using TSX.[7][9][1]
- Engage with ecosystem partners (docs tools, tour vendors, AI platforms) to adopt TSX as an interchange format.

## 13. Open Questions

- Should TSX adopt indentation‑based nesting (Pug/Slim‑style) to further reduce tokens, or keep explicit `::/end` markers for robustness?[24][25]
- How prescriptive should the spec be about library‑specific options vs a minimal core that leaves detail to adapters?
- Should TSX also support static screenshot/video walkthrough metadata, or focus solely on in‑app tours?
- How should versioning and extensions be governed if third parties define new directives?