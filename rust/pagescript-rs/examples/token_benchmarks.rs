use std::{error::Error, fs, path::Path};

use pagescript_rs::{
    ArtifactTokenMeasure, Resolver, measure_token_savings, parse_page_script, render_to_html,
};
use serde_json::json;

const CASES: [(&str, &str); 5] = [
    ("paper-sizes", "Paper sizes"),
    ("service-status", "Service status"),
    ("rollout-guide", "Rollout guide"),
    ("build-summary", "Build summary"),
    ("incident-timeline", "Incident timeline"),
];

fn main() -> Result<(), Box<dyn Error>> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let resolver = Resolver::new(Some(repo.join("examples")));
    let mut cases = Vec::with_capacity(CASES.len());

    for (id, title) in CASES {
        let page_path = format!("examples/{id}.page");
        let html_path = format!("benchmarks/html/{id}.html");
        let source = fs::read_to_string(repo.join(&page_path))?;
        let baseline = fs::read_to_string(repo.join(&html_path))?;
        let compiled = render_to_html(&parse_page_script(&source), Some(id), &resolver)?;
        let compiled_report = measure_token_savings(&source, &compiled)?;
        let handwritten_html = ArtifactTokenMeasure::from_text(&baseline)?;
        if handwritten_html.tokens == 0 {
            return Err(format!("{html_path} is empty").into());
        }
        let ratio = compiled_report.authored_source.tokens as f64 / handwritten_html.tokens as f64;
        let difference_percent = (10000.0 * (ratio - 1.0)).round() / 100.0;

        cases.push(json!({
            "id": id,
            "title": title,
            "source_path": page_path,
            "handwritten_path": html_path,
            "authored_source": compiled_report.authored_source,
            "handwritten_html": handwritten_html,
            "generated_html": compiled_report.generated_html,
            "source_delta_vs_handwritten_percent": difference_percent
        }));
    }

    let report = json!({
        "tokenizer": "o200k_base",
        "line_endings": "LF",
        "cases": cases
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
