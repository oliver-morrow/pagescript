#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

while IFS= read -r file; do
    jq empty "$file"
done < <(find schemas examples conformance -type f -name '*.json' -print | sort)

cargo run -p pagescript-rs --locked -- validate docs/index.page

rm -rf _site
mkdir -p _site/docs _site/examples _site/schemas _site/conformance _site/playground/compiler
cp README.md SPEC.md RATIONALE.md CONFORMANCE.md CHANGELOG.md LICENSE _site/
cp -R docs/. _site/docs/
cp -R examples/. _site/examples/
cp -R schemas/. _site/schemas/
cp -R conformance/. _site/conformance/

cargo run -p pagescript-rs --locked -- render docs/index.page --page pagescript-docs > _site/index.html
cargo run -p pagescript-rs --locked -- render examples/product-metrics-demo.page \
    --page product-metrics-demo --out _site/examples/product-metrics-demo.html
cargo run -p pagescript-rs --locked -- stats examples/revenue-map-demo.page --page revenue-map > _site/examples/revenue-map.o200k.json
diff -u conformance/stats/revenue-map.o200k.json _site/examples/revenue-map.o200k.json
grep -Fq 'PageScript' _site/index.html
grep -Fq '64.08%' _site/index.html
grep -Fq 'ps-runtime-config' _site/index.html
grep -Fq 'Product Metrics' _site/examples/product-metrics-demo.html

wasm-pack build --target web --release --no-opt --out-name pagescript \
    --out-dir "$PWD/_site/playground/compiler" rust/pagescript-web -- --locked
cp LICENSE _site/playground/compiler/
node --test rust/pagescript-web/tests/compiler.mjs
