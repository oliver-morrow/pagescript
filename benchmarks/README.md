# Authored token counts

Each pair has the same visible content and behavior. The hand-written HTML includes its styles and any scripts, as does the PageScript source. The build summary uses a recipe from the compiler's standard library. Compiler and library code count toward the generated HTML column when bundled.

| Example | PageScript | Hand-written HTML | Generated HTML | PageScript vs hand-written |
| --- | ---: | ---: | ---: | ---: |
| Paper sizes · [source](../examples/paper-sizes.page) · [HTML](html/paper-sizes.html) | 830 | 678 | 2,954 | 22.42% more |
| Service status · [source](../examples/service-status.page) · [HTML](html/service-status.html) | 386 | 465 | 2,558 | 16.99% fewer |
| Rollout guide · [source](../examples/rollout-guide.page) · [HTML](html/rollout-guide.html) | 435 | 472 | 2,513 | 7.84% fewer |
| Build summary · [source](../examples/build-summary.page) · [HTML](html/build-summary.html) | 521 | 457 | 2,700 | 14.00% more |
| Incident timeline · [source](../examples/incident-timeline.page) · [HTML](html/incident-timeline.html) | 277 | 395 | 2,364 | 29.87% fewer |

From the repo root:

```sh
cargo run --quiet --locked -p pagescript-rs --example token_benchmarks
```

The runner normalizes line endings to LF and uses the CLI's `o200k_base` tokenizer. The results measure files only. Prompts, tool calls, and revisions are outside this measurement. [Checked counts](token-counts.json)

The original revenue map compared 1,787 PageScript tokens with 4,975 tokens of compiler output. [Revenue map report](../conformance/stats/revenue-map.o200k.json)
