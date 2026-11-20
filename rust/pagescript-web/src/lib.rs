use pagescript_rs::{Diagnostic, Resolver, parse_page_script, render_to_html, validate_document};
use serde::Serialize;
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Compilation {
    Rendered { html: String },
    Invalid { diagnostics: Vec<Diagnostic> },
    RenderError { message: String },
}

#[wasm_bindgen]
pub fn compile_page(source: &str, page_id: Option<String>) -> Result<String, JsError> {
    let document = parse_page_script(source);
    let resolver = Resolver::new(None);
    let compilation = match render_to_html(&document, page_id.as_deref(), &resolver) {
        Ok(html) => Compilation::Rendered { html },
        Err(message) => {
            let diagnostics = validate_document(&document, &resolver);
            if diagnostics.is_empty() {
                Compilation::RenderError { message }
            } else {
                Compilation::Invalid { diagnostics }
            }
        }
    };

    serde_json::to_string(&compilation).map_err(|error| JsError::new(&error.to_string()))
}
