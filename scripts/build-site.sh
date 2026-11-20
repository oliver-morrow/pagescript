#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

while IFS= read -r file; do
    jq empty "$file"
done < <(find schemas examples conformance -type f -name '*.json' -print | sort)

cargo run -p pagescript-rs --locked -- validate docs/index.page

rm -rf _site
mkdir -p _site/docs _site/examples _site/schemas _site/conformance _site/playground/compiler \
    _site/benchmarks/html _site/skills/pagescript
cp -R web/. _site/
cp README.md SPEC.md RATIONALE.md CONFORMANCE.md CHANGELOG.md LICENSE _site/
cp -R docs/. _site/docs/
cp -R examples/. _site/examples/
cp -R schemas/. _site/schemas/
cp -R conformance/. _site/conformance/
cp benchmarks/README.md _site/benchmarks/
cp benchmarks/html/*.html _site/benchmarks/html/
cp skills/pagescript/SKILL.md _site/skills/pagescript/

cargo run -p pagescript-rs --locked -- render docs/index.page --page pagescript-docs --out _site/docs/index.html
cargo run -p pagescript-rs --locked -- render examples/product-metrics-demo.page \
    --page product-metrics-demo --out _site/examples/product-metrics-demo.html
cargo run --quiet --locked -p pagescript-rs --example token_benchmarks > _site/benchmarks/token-counts.json
diff -u benchmarks/token-counts.json _site/benchmarks/token-counts.json
case_ids=$(jq -r '.cases[].id' _site/benchmarks/token-counts.json)
while IFS= read -r id; do
    cargo run -p pagescript-rs --locked -- render "examples/$id.page" --page "$id" --out "_site/examples/$id.html"
done <<< "$case_ids"
node scripts/render-benchmarks.mjs _site/benchmarks/token-counts.json _site/index.html
cargo run -p pagescript-rs --locked -- stats examples/revenue-map-demo.page --page revenue-map > _site/examples/revenue-map.o200k.json
diff -u conformance/stats/revenue-map.o200k.json _site/examples/revenue-map.o200k.json
grep -Fq 'PageScript' _site/index.html
grep -Fq '64.08%' _site/docs/index.html
grep -Fq 'ps-runtime-config' _site/docs/index.html
grep -Fq 'Product Metrics' _site/examples/product-metrics-demo.html
grep -Fq 'Paper sizes' _site/examples/paper-sizes.html

skill_stage=$(mktemp -d)
trap 'rm -rf -- "$skill_stage"' EXIT
mkdir "$skill_stage/pagescript"
cp skills/pagescript/SKILL.md LICENSE "$skill_stage/pagescript/"
skill_archive="$PWD/_site/pagescript-skill.zip"
(cd "$skill_stage" && zip -q -X "$skill_archive" pagescript/SKILL.md pagescript/LICENSE)

wasm-pack build --target web --release --no-opt --out-name pagescript \
    --out-dir "$PWD/_site/playground/compiler" rust/pagescript-web -- --locked
cp LICENSE _site/playground/compiler/
node --test rust/pagescript-web/tests/compiler.mjs
