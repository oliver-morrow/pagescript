import { readFileSync, writeFileSync } from "node:fs";

const [reportPath, indexPath] = process.argv.slice(2);
const { cases } = JSON.parse(readFileSync(reportPath, "utf8"));
const format = new Intl.NumberFormat("en", { maximumFractionDigits: 2 });

const rows = cases.map(({ id, title, authored_source, handwritten_html, source_delta_vs_handwritten_percent }) => {
  if (!/^[a-z0-9-]+$/.test(id)) throw new Error("Unexpected example identifier");
  const name = title.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
  const delta = source_delta_vs_handwritten_percent;
  const change = delta === 0 ? "Same" : format.format(Math.abs(delta)) + "% " + (delta < 0 ? "fewer" : "more");
  const count = format.format(handwritten_html.tokens);
  return `<tr>
            <th scope="row"><a href="./?example=${id}" data-example="${id}">${name}</a></th>
            <td>${format.format(authored_source.tokens)}</td>
            <td><a href="./benchmarks/html/${id}.html" aria-label="${count} tokens, view ${name} HTML">${count}</a></td>
            <td>${change}</td>
          </tr>`;
});

const index = readFileSync(indexPath, "utf8");
const marker = "<!-- benchmark rows -->";
if (!index.includes(marker)) throw new Error("Missing benchmark table marker");
writeFileSync(indexPath, index.replace(marker, () => rows.join("\n          ")));
