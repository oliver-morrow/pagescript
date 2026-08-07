use std::collections::BTreeMap;

use crate::evidence::{
    Citation, Confidence, ExplainerEntityIr, ExplainerIr, ExplainerRelationshipIr, Source,
    TokenValue,
};

pub fn render_explainer_to_html(ir: &ExplainerIr) -> String {
    let sources = ir
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    let accent = ir
        .tokens
        .get("color.accent")
        .and_then(token_as_css_value)
        .filter(|value| is_safe_css_color(value))
        .unwrap_or("#4dd6a0");
    let navigation = ir
        .views
        .iter()
        .map(|view| {
            format!(
                "<li><a href=\"#view-{}\">{}</a></li>",
                escape_attr(&view.id),
                escape_html(&view.title)
            )
        })
        .collect::<String>();
    let views = ir
        .views
        .iter()
        .map(|view| {
            let entities = view
                .entities
                .iter()
                .map(|entity| render_entity(entity, &sources))
                .collect::<String>();
            let relationships = view
                .relationships
                .iter()
                .map(|relationship| render_relationship(relationship, &sources))
                .collect::<String>();
            let callouts = view
                .callouts
                .iter()
                .map(|callout| {
                    format!(
                        "<article class=\"ps-callout\" id=\"callout-{}\"><h3>{}</h3><p>{}</p><p class=\"ps-reference\">Entities: {}</p>{}</article>",
                        escape_attr(&callout.id),
                        escape_html(&callout.title),
                        escape_html(&callout.body),
                        callout
                            .entities
                            .iter()
                            .map(|id| format!("<code>{}</code>", escape_html(id)))
                            .collect::<Vec<_>>()
                            .join(", "),
                        if callout.relationships.is_empty() {
                            String::new()
                        } else {
                            format!(
                                "<p class=\"ps-reference\">Relationships: {}</p>",
                                callout
                                    .relationships
                                    .iter()
                                    .map(|id| format!("<code>{}</code>", escape_html(id)))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        }
                    )
                })
                .collect::<String>();
            format!(
                "<section class=\"ps-view\" id=\"view-{}\" aria-labelledby=\"view-heading-{}\"><header><p class=\"ps-eyebrow\">{} view</p><h2 id=\"view-heading-{}\">{}</h2>{}</header><div class=\"ps-layout\"><section aria-label=\"Entities\"><h3>Entities</h3><div class=\"ps-entities\">{}</div></section><section aria-label=\"Relationships\"><h3>Relationships</h3><ol class=\"ps-relationships\">{}</ol></section></div>{}</section>",
                escape_attr(&view.id),
                escape_attr(&view.id),
                escape_html(view_kind_label(&view.kind)),
                escape_attr(&view.id),
                escape_html(&view.title),
                view.summary.as_deref().map(|summary| format!("<p class=\"ps-summary\">{}</p>", escape_html(summary))).unwrap_or_default(),
                entities,
                relationships,
                if callouts.is_empty() { String::new() } else { format!("<section class=\"ps-callouts\" aria-label=\"Explanations\"><h3>What to know</h3>{callouts}</section>") }
            )
        })
        .collect::<String>();

    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta name=\"color-scheme\" content=\"dark light\"><title>{}</title><style>:root{{--ps-accent:{};--ps-ink:#e9eef5;--ps-muted:#aeb8c8;--ps-surface:#142033;--ps-page:#0c1422;--ps-border:#29415e}}*{{box-sizing:border-box}}body{{margin:0;background:var(--ps-page);color:var(--ps-ink);font:16px/1.55 ui-sans-serif,system-ui,sans-serif}}a{{color:var(--ps-accent)}}code{{font:0.88em ui-monospace,SFMono-Regular,Menlo,monospace}}.ps-shell{{max-width:1120px;margin:auto;padding:3rem 1.25rem 5rem}}.ps-header{{border-bottom:1px solid var(--ps-border);padding-bottom:2rem}}h1,h2,h3,p{{margin-top:0}}h1{{font-size:clamp(2rem,6vw,4rem);line-height:1.05;max-width:15ch}}h2{{font-size:1.7rem;margin-bottom:.4rem}}h3{{font-size:1rem;margin-bottom:.8rem}}.ps-eyebrow,.ps-reference{{color:var(--ps-muted);font-size:.85rem;letter-spacing:.04em;text-transform:uppercase}}.ps-summary{{color:var(--ps-muted);max-width:70ch}}.ps-nav ul{{display:flex;flex-wrap:wrap;gap:.75rem 1.2rem;padding:0;list-style:none}}.ps-view{{padding:3rem 0;border-bottom:1px solid var(--ps-border)}}.ps-layout{{display:grid;grid-template-columns:minmax(0,1.2fr) minmax(260px,.8fr);gap:1.5rem}}.ps-entities{{display:grid;gap:.75rem}}.ps-entity,.ps-callout,.ps-relationship{{background:var(--ps-surface);border:1px solid var(--ps-border);border-radius:.7rem;padding:1rem}}.ps-entity h4{{margin:.1rem 0 .3rem}}.ps-kind{{color:var(--ps-accent);font-size:.8rem;font-weight:700;text-transform:uppercase}}.ps-relationships{{display:grid;gap:.75rem;padding:0;list-style:none}}.ps-route{{display:flex;flex-wrap:wrap;gap:.4rem;align-items:center;font-weight:600}}.ps-arrow{{color:var(--ps-accent)}}.ps-callouts{{margin-top:1.5rem;display:grid;gap:1rem}}details{{margin-top:.75rem}}summary{{cursor:pointer;color:var(--ps-accent)}}.ps-citations{{margin:.75rem 0 0;padding-left:1.2rem;color:var(--ps-muted)}}.ps-footer{{padding-top:2rem;color:var(--ps-muted);font-size:.875rem}}@media (max-width:760px){{.ps-shell{{padding-top:2rem}}.ps-layout{{grid-template-columns:1fr}}}}</style></head><body><main class=\"ps-shell\"><header class=\"ps-header\"><p class=\"ps-eyebrow\">PageScript source-cited explainer</p><h1>{}</h1>{}<nav class=\"ps-nav\" aria-label=\"Explainer views\"><ul>{}</ul></nav></header>{}<footer class=\"ps-footer\">Evidence bundle <code>{}</code>. Rendered locally with no external requests.</footer></main></body></html>",
        escape_html(&ir.title),
        escape_attr(accent),
        escape_html(&ir.title),
        ir.summary
            .as_deref()
            .map(|summary| format!("<p class=\"ps-summary\">{}</p>", escape_html(summary)))
            .unwrap_or_default(),
        navigation,
        views,
        escape_html(&ir.bundle_digest)
    )
}

fn render_entity(entity: &ExplainerEntityIr, sources: &BTreeMap<&str, &Source>) -> String {
    format!(
        "<article class=\"ps-entity\" id=\"entity-{}\"><p class=\"ps-kind\">{} · {}</p><h4>{}</h4>{}{}{}</article>",
        escape_attr(&entity.id),
        escape_html(entity_kind_label(&entity.kind)),
        escape_html(confidence_label(&entity.provenance.confidence)),
        escape_html(&entity.label),
        entity
            .description
            .as_deref()
            .map(|description| format!("<p>{}</p>", escape_html(description)))
            .unwrap_or_default(),
        entity
            .group
            .as_deref()
            .map(|group| format!(
                "<p class=\"ps-reference\">Group: <code>{}</code></p>",
                escape_html(group)
            ))
            .unwrap_or_default(),
        render_citations(&entity.provenance.evidence, sources)
    )
}

fn render_relationship(
    relationship: &ExplainerRelationshipIr,
    sources: &BTreeMap<&str, &Source>,
) -> String {
    format!(
        "<li class=\"ps-relationship\" id=\"relationship-{}\"><p class=\"ps-route\"><code>{}</code><span class=\"ps-arrow\" aria-hidden=\"true\">→</span><code>{}</code></p><p>{}{}</p><p class=\"ps-reference\">{} · {}</p>{}</li>",
        escape_attr(&relationship.id),
        escape_html(&relationship.from),
        escape_html(&relationship.to),
        escape_html(relationship_kind_label(&relationship.kind)),
        relationship
            .label
            .as_deref()
            .map(|label| format!(": {}", escape_html(label)))
            .unwrap_or_default(),
        escape_html(confidence_label(&relationship.provenance.confidence)),
        escape_html(&relationship.id),
        render_citations(&relationship.provenance.evidence, sources)
    )
}

fn render_citations(citations: &[Citation], sources: &BTreeMap<&str, &Source>) -> String {
    if citations.is_empty() {
        return String::new();
    }
    let items = citations
        .iter()
        .map(|citation| {
            let path = sources
                .get(citation.source.as_str())
                .map(|source| source.path.as_str())
                .unwrap_or(citation.source.as_str());
            let locator = match (
                citation.start_line,
                citation.end_line,
                citation.symbol.as_deref(),
                citation.json_pointer.as_deref(),
            ) {
                (Some(start), Some(end), _, _) if start == end => format!("{path}:{start}"),
                (Some(start), Some(end), _, _) => format!("{path}:{start}-{end}"),
                (_, _, Some(symbol), _) => format!("{path}#{symbol}"),
                (_, _, _, Some(pointer)) => format!("{path}{pointer}"),
                _ => path.to_string(),
            };
            format!("<li><code>{}</code></li>", escape_html(&locator))
        })
        .collect::<String>();
    format!(
        "<details><summary>Sources ({})</summary><ul class=\"ps-citations\">{items}</ul></details>",
        citations.len()
    )
}

fn token_as_css_value(value: &TokenValue) -> Option<&str> {
    match value {
        TokenValue::String(value) => Some(value),
        _ => None,
    }
}

fn is_safe_css_color(value: &str) -> bool {
    value.starts_with('#')
        && matches!(value.len(), 4 | 5 | 7 | 9)
        && value[1..]
            .bytes()
            .all(|character| character.is_ascii_hexdigit())
}

fn confidence_label(confidence: &Confidence) -> &'static str {
    match confidence {
        Confidence::Extracted => "extracted",
        Confidence::Inferred => "inferred",
        Confidence::Declared => "declared",
    }
}

fn entity_kind_label(kind: &crate::evidence::EntityKind) -> &'static str {
    match kind {
        crate::evidence::EntityKind::Module => "module",
        crate::evidence::EntityKind::Package => "package",
        crate::evidence::EntityKind::Service => "service",
        crate::evidence::EntityKind::Database => "database",
        crate::evidence::EntityKind::Dataset => "dataset",
        crate::evidence::EntityKind::Model => "model",
        crate::evidence::EntityKind::Source => "source",
        crate::evidence::EntityKind::Test => "test",
        crate::evidence::EntityKind::Exposure => "exposure",
        crate::evidence::EntityKind::File => "file",
        crate::evidence::EntityKind::Symbol => "symbol",
        crate::evidence::EntityKind::External => "external",
        crate::evidence::EntityKind::Group => "group",
    }
}

fn relationship_kind_label(kind: &crate::evidence::RelationshipKind) -> &'static str {
    match kind {
        crate::evidence::RelationshipKind::Contains => "contains",
        crate::evidence::RelationshipKind::Imports => "imports",
        crate::evidence::RelationshipKind::DependsOn => "depends on",
        crate::evidence::RelationshipKind::Calls => "calls",
        crate::evidence::RelationshipKind::Reads => "reads",
        crate::evidence::RelationshipKind::Writes => "writes",
        crate::evidence::RelationshipKind::Produces => "produces",
        crate::evidence::RelationshipKind::Consumes => "consumes",
        crate::evidence::RelationshipKind::Tests => "tests",
        crate::evidence::RelationshipKind::Documents => "documents",
        crate::evidence::RelationshipKind::Custom => "custom",
    }
}

fn view_kind_label(kind: &crate::evidence::ViewKind) -> &'static str {
    match kind {
        crate::evidence::ViewKind::Overview => "overview",
        crate::evidence::ViewKind::Architecture => "architecture",
        crate::evidence::ViewKind::Lineage => "lineage",
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value)
}
