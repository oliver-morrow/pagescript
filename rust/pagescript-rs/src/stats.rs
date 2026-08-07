use serde::{Deserialize, Serialize};
use tiktoken_rs::o200k_base;

/// A reproducible measurement of the authored source required to create a page.
///
/// It deliberately compares source to the compiler's generated standalone HTML,
/// not to an LLM prompt or a hand-written implementation. Prompt and repair-turn
/// costs are model- and workflow-specific and must be measured separately.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactTokenMeasure {
    pub bytes: usize,
    pub tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenSavingsReport {
    pub schema_version: String,
    pub tokenizer: String,
    pub comparison: String,
    pub authored_source: ArtifactTokenMeasure,
    pub generated_html: ArtifactTokenMeasure,
    pub authored_source_token_reduction_percent: f64,
    pub methodology: String,
}

pub fn measure_token_savings(
    authored_source: &str,
    generated_html: &str,
) -> Result<TokenSavingsReport, String> {
    let tokenizer = o200k_base().map_err(|error| format!("Could not load o200k_base: {error}"))?;
    let source = measure(&tokenizer, authored_source);
    let html = measure(&tokenizer, generated_html);
    let reduction = if html.tokens == 0 {
        0.0
    } else {
        round_to_two_decimal_places(100.0 * (1.0 - source.tokens as f64 / html.tokens as f64))
    };

    Ok(TokenSavingsReport {
        schema_version: "1.0".to_string(),
        tokenizer: "o200k_base".to_string(),
        comparison: "authored PageScript source vs generated standalone HTML".to_string(),
        authored_source: source,
        generated_html: html,
        authored_source_token_reduction_percent: reduction,
        methodology: "Counts both checked-in artifacts with o200k_base; excludes prompts, system instructions, tool calls, repair turns, and prior context.".to_string(),
    })
}

fn measure(tokenizer: &tiktoken_rs::CoreBPE, content: &str) -> ArtifactTokenMeasure {
    ArtifactTokenMeasure {
        bytes: content.len(),
        tokens: tokenizer.encode_with_special_tokens(content).len(),
    }
}

fn round_to_two_decimal_places(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
