import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const site = new URL("../../../_site/", import.meta.url);
const compiler = new URL("playground/compiler/", site);
const { default: initialize, compile_page } = await import(new URL("pagescript.js", compiler));
await initialize({ module_or_path: await readFile(new URL("pagescript_bg.wasm", compiler)) });

function readSiteFile(path) {
  return readFile(new URL(path, site), "utf8");
}

const example = await readSiteFile("examples/product-metrics-demo.page");

test("browser compiler matches the CLI for a page using built-in recipes", async () => {
  const expectedHtml = await readSiteFile("examples/product-metrics-demo.html");
  const result = JSON.parse(compile_page(example, "product-metrics-demo"));
  assert.deepEqual(result, { status: "rendered", html: expectedHtml });
});

test("invalid source returns diagnostics with the CLI's source locations", async () => {
  const source = await readSiteFile("conformance/invalid/required-fields.page");
  const expected = await readSiteFile("conformance/invalid/required-fields.diagnostics.json");
  const result = JSON.parse(compile_page(source));
  assert.deepEqual(result, { status: "invalid", diagnostics: JSON.parse(expected) });
});

test("a missing page selection returns a render error", () => {
  const result = JSON.parse(compile_page(example, "missing"));
  assert.deepEqual(result, { status: "render_error", message: 'Page "missing" was not found.' });
});
