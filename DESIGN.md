---
name: PageScript
description: "Wayfinding for the PageScript playground"
colors:
  navy: "#182c4b"
  shell: "#e8edf4"
  surface: "#fbfcfe"
  sign: "#dbe6f2"
  border: "#becddd"
  rule: "#d9e3ef"
  blue: "#3a5b8a"
  focus: "#235c9e"
  yellow: "#f9d65c"
  yellow-hover: "#ffe591"
  code-field: "#edf3f9"
  error: "#7f332b"
  error-field: "#ffefe9"
  directive: "#234f89"
  string: "#236256"
  control-border: "#91a8c3"
  disabled-fill: "#e7eaf0"
  disabled-text: "#4d627f"
  download-hover: "#edf4fc"
  copy-hover: "#eaf1f9"
typography:
  display:
    fontFamily: '"Barlow Semi Condensed", Arial, sans-serif'
    fontSize: "clamp(35px, 4vw, 56px)"
    fontWeight: 700
    lineHeight: 1.03
    letterSpacing: "-.025em"
  headline:
    fontFamily: '"Barlow Semi Condensed", Arial, sans-serif'
    fontSize: "30px"
    fontWeight: 700
    lineHeight: 1.15
    letterSpacing: "-.02em"
  title:
    fontFamily: "Barlow, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 700
    lineHeight: 1.2
  body:
    fontFamily: "Barlow, system-ui, sans-serif"
    fontSize: "17px"
    fontWeight: 400
    lineHeight: 1.45
  label:
    fontFamily: "Barlow, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1.2
  copy-label:
    fontFamily: "Barlow, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1.2
  source:
    fontFamily: '"IBM Plex Mono", ui-monospace, monospace'
    fontSize: "14px"
    fontWeight: 400
  filename:
    fontFamily: '"IBM Plex Mono", ui-monospace, monospace'
    fontSize: "13px"
    fontWeight: 500
    lineHeight: 1.25
rounded:
  control: "6px"
  field: "8px"
  pane: "13px"
spacing:
  page-padding: "clamp(16px, 3.6vw, 60px)"
  pane-gap: "20px"
  editor-inset: "20px"
  section-gap: "34px"
  section-inset: "24px"
components:
  button-render:
    backgroundColor: "{colors.yellow}"
    textColor: "{colors.navy}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0 12px"
  button-render-hover:
    backgroundColor: "{colors.yellow-hover}"
  button-render-disabled:
    backgroundColor: "{colors.disabled-fill}"
    textColor: "{colors.disabled-text}"
  button-download:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.navy}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "0 12px"
  button-download-hover:
    backgroundColor: "{colors.download-hover}"
  button-copy:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.navy}"
    typography: "{typography.copy-label}"
    rounded: "{rounded.control}"
    padding: "0 12px"
  button-copy-hover:
    backgroundColor: "{colors.copy-hover}"
  button-copy-disabled:
    textColor: "{colors.blue}"
  prompt-field:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.navy}"
    typography: "{typography.body}"
    rounded: "{rounded.field}"
    padding: "10px 12px"
  navigation:
    backgroundColor: "{colors.navy}"
    textColor: "{colors.surface}"
  pane:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.pane}"
  pane-bar:
    backgroundColor: "{colors.sign}"
    textColor: "{colors.navy}"
    typography: "{typography.title}"
    padding: "7px 18px"
  editor-gutter:
    backgroundColor: "{colors.code-field}"
    textColor: "{colors.blue}"
    typography: "{typography.source}"
  preview-notice:
    backgroundColor: "{colors.code-field}"
    textColor: "{colors.blue}"
    padding: "9px 14px"
  preview-notice-error:
    backgroundColor: "{colors.error-field}"
    textColor: "{colors.error}"
  examples-table:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.navy}"
    typography: "{typography.body}"
  examples-table-header:
    backgroundColor: "{colors.sign}"
---

# Design System: PageScript

## Overview

**Creative North Star: "Wayfinding"**

The compiler workspace uses labeled panels for source and output. Each panel header puts its action beside the panel title.

## Colors

### Primary

Use yellow for Render and text selection. Secondary controls use pale surfaces.

### Neutral

Use navy for the site header and main text. The shell fills the page, the surface fills panels and the table, and the sign color marks their headers. Use blue for links, file labels, and neutral notices. The code field color fills both the editor gutter and neutral notices.

Use the warm background and dark red text for errors. Source directives are blue and strings are green.

## Typography

Barlow Semi Condensed sets the page heading, section headings, and 29px wordmark. Barlow sets pane titles, body copy, and control labels. IBM Plex Mono sets filenames, code, and the numbered editor, with a 27px line height at desktop sizes.

At 760px and below, the wordmark is 25px, pane titles 18px, filenames 11px, and source 16px on 26px lines. Use tabular figures for line numbers and table comparisons.

## Layout

The frame caps at 1440px with fluid side padding. The workbench splits Source and Preview 2:3 and scales from 500px to 670px tall. Supporting sections start with a divider and top inset.

At 1160px and below, the split shifts to 1:1.15 with a 14px panel gap and 43px gutter. At 760px and below, stack Source above Preview with a 12px gap, a 194px source body, a 320px preview body, and a 37px source gutter. Move the prompt below the agent instructions and reduce table padding. At 390px and below, increase the prompt field’s minimum height to 148px. At 360px and below, reduce the navigation gap to 8px.

## Elevation & Depth

Use flat layers and one-pixel dividers.

## Shapes

Round and clip the pane around its header and body. Keep the editor and table square.

## Components

- **Buttons.** Keep a 44px minimum height. Render lightens on hover and turns gray when disabled or busy. Download HTML and Copy prompt use outlined pale buttons with cool hover fills. Render and Download HTML narrow to 9px horizontal padding on phones.
- **Focus.** Use a 3px blue outline with a 3px offset for keyboard focus. Header links use yellow. Editor and preview focus rings sit inside their edges.
- **Navigation.** Keep the project links beside the wordmark on the navy bar at desktop and phone widths. Underline header links on hover.
- **Editor and preview.** Ellipsize the filename when space is tight. Show loading, rendering, stale, and error notices below the preview. Preserve the last render on source errors and place any line and column jumps beneath the editor.
- **Prompt and examples.** Use a bordered, vertically resizable prompt field. Give the table a tinted header, fine horizontal rules, and right-aligned numeric columns.

## Do's and Don'ts

- **Do** use the PageScript name as a plain text wordmark.
- **Do** keep the preview footer clear when the result is current.
