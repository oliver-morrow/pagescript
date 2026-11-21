import { readFileSync, writeFileSync } from "node:fs";

const [reportPath, indexPath] = process.argv.slice(2);
const { cases } = JSON.parse(readFileSync(reportPath, "utf8"));
const format = new Intl.NumberFormat("en", { maximumFractionDigits: 2 });
const longest = Math.max(...cases.flatMap(({ authored_source, handwritten_html }) => [authored_source.tokens, handwritten_html.tokens]));

// Draws both counts as dimension lines on one shared scale.
function dimension(tokens, y, kind) {
  const end = (2 + 296 * tokens / longest).toFixed(1);
  return `<line class="${kind} ext" x1="2" y1="${y - 5}" x2="2" y2="${y + 5}"/><line class="${kind} ext" x1="${end}" y1="${y - 5}" x2="${end}" y2="${y + 5}"/><line class="${kind}" x1="2" y1="${y}" x2="${end}" y2="${y}"/>`;
}

const rows = cases.map(({ id, title, authored_source, handwritten_html, source_delta_vs_handwritten_percent }) => {
  if (!/^[a-z0-9-]+$/.test(id)) throw new Error("Unexpected example identifier");
  const name = title.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
  const delta = source_delta_vs_handwritten_percent;
  const change = Math.round(delta) === 0 ? "Same" : Math.round(Math.abs(delta)) + "% " + (delta < 0 ? "fewer" : "more");
  const count = format.format(handwritten_html.tokens);
  return `<tr>
            <th scope="row"><a href="./?example=${id}" data-example="${id}">${name}</a></th>
            <td data-label="PageScript">${format.format(authored_source.tokens)}</td>
            <td data-label="HTML"><a href="./benchmarks/html/${id}.html" aria-label="${count} tokens, view ${name} HTML">${count}</a></td>
            <td data-label="Change">${change}</td>
            <td class="scale-cell"><svg class="scale" viewBox="0 0 300 30" preserveAspectRatio="none" aria-hidden="true">${dimension(authored_source.tokens, 8, "page")}${dimension(handwritten_html.tokens, 22, "html")}</svg></td>
          </tr>`;
});

const index = readFileSync(indexPath, "utf8");
const marker = "<!-- benchmark rows -->";
if (!index.includes(marker)) throw new Error("Missing benchmark table marker");
writeFileSync(indexPath, index.replace(marker, () => rows.join("\n          ")));
