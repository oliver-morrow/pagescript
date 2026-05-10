use std::collections::HashMap;

use serde_json::{Value, json};

use crate::{
    ir::{ComponentIr, GraphIr, GraphNodeIr, IrNode, MarkdownIr, PageIr, compile_page_ir},
    types::{AttributeValue, DocumentNode},
};

pub fn render_to_html(document: &DocumentNode, page_id: Option<&str>) -> Result<String, String> {
    let page = compile_page_ir(document, page_id)?;
    let context = RenderContext { page: &page };
    let body = page
        .body
        .iter()
        .map(|node| render_node(node, &context))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>{}</title>\n  <style>{}\n{}\n{}</style>\n</head>\n<body data-pagescript-page=\"{}\">\n{}\n  <script type=\"application/json\" id=\"ps-runtime-config\">{}</script>\n  <script>{}</script>\n</body>\n</html>\n",
        escape_html(&page.title),
        base_css(),
        token_css(&page),
        page.scoped_css,
        escape_attr(page.id.as_deref().unwrap_or("")),
        body,
        escape_script_json(&runtime_json(&page)),
        base_js()
    ))
}

struct RenderContext<'a> {
    page: &'a PageIr,
}

#[derive(Clone, Copy)]
struct GraphPoint {
    x: i64,
    y: i64,
}

const GRAPH_NODE_WIDTH: i64 = 148;
const GRAPH_NODE_HEIGHT: i64 = 58;
const GRAPH_NODE_HALF_WIDTH: i64 = GRAPH_NODE_WIDTH / 2;
const GRAPH_NODE_HALF_HEIGHT: i64 = GRAPH_NODE_HEIGHT / 2;

fn runtime_json(page: &PageIr) -> String {
    let states = page
        .states
        .iter()
        .map(|state| json!({"id": state.id, "default": state.default_value}))
        .collect::<Vec<_>>();
    let events = page
        .events
        .iter()
        .map(|event| json!({"on": event.on, "set": event.set, "value": event.value}))
        .collect::<Vec<_>>();
    serde_json::to_string(&json!({ "states": states, "events": events })).unwrap_or_default()
}

fn token_css(page: &PageIr) -> String {
    if page.tokens.is_empty() {
        return String::new();
    }
    let declarations = page
        .tokens
        .iter()
        .flat_map(|(key, value)| token_declarations(key, value))
        .collect::<Vec<_>>()
        .join("");
    format!(":root{{{declarations}}}")
}

fn token_declarations(key: &str, value: &Value) -> Vec<String> {
    let Some(value) = token_value(key, value) else {
        return Vec::new();
    };
    let token_name = sanitize_class(key);
    let mut declarations = vec![format!("--ps-token-{token_name}:{value};")];
    if let Some(alias) = token_alias(key) {
        declarations.push(format!("{alias}:{value};"));
    }
    declarations
}

fn token_value(key: &str, value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(escape_css_value(value)),
        Value::Number(value) if key.starts_with("radius.") || key.starts_with("spacing.") => {
            Some(format!("{value}px"))
        }
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn token_alias(key: &str) -> Option<&'static str> {
    match key {
        "color.bg" | "color-bg" => Some("--ps-bg"),
        "color.ink" | "color-ink" => Some("--ps-ink"),
        "color.muted" | "color-muted" => Some("--ps-muted"),
        "color.line" | "color-line" => Some("--ps-line"),
        "color.accent" | "color-accent" => Some("--ps-accent"),
        "color.accent-ink" | "color-accent-ink" => Some("--ps-accent-ink"),
        "color.panel" | "color-panel" => Some("--ps-panel"),
        "radius.panel" | "radius-panel" => Some("--ps-radius"),
        _ => None,
    }
}

fn render_node(node: &IrNode, context: &RenderContext<'_>) -> String {
    match node {
        IrNode::Markdown(markdown) => render_markdown(markdown),
        IrNode::Component(component) => render_component(component, context),
    }
}

fn render_component(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let children = node
        .children
        .iter()
        .map(|node| render_node(node, context))
        .collect::<Vec<_>>()
        .join("\n");
    match node.name.as_str() {
        "hero" => format!(
            "<section class=\"{}\">\n  <div class=\"ps-container ps-hero-inner\">\n    {}{}{}\n  </div>\n</section>",
            component_classes(node, &["ps-hero", &tone(node), &spacing(node)], context),
            heading(node, "h1"),
            body(node),
            children
        ),
        "section" => format!(
            "<section class=\"{}\"{}>\n  <div class=\"ps-container\">{}{}{}</div>\n</section>",
            component_classes(node, &["ps-section", &tone(node), &spacing(node)], context),
            id_attr(node),
            heading(node, "h2"),
            body(node),
            children
        ),
        "scene" => render_scene(node, context),
        "panel" => render_panel(node, context),
        "stack" => format!(
            "<div class=\"{}\"{}>{children}</div>",
            component_classes(node, &["ps-stack", &gap(node)], context),
            id_attr(node)
        ),
        "grid" => format!(
            "<div class=\"{}\"{} style=\"--ps-columns:{}\">{children}</div>",
            component_classes(node, &["ps-grid", &gap(node)], context),
            id_attr(node),
            columns(node, 2)
        ),
        "card" => format!(
            "<article class=\"{}\"{}>\n  {}{}{}{children}\n</article>",
            component_classes(node, &["ps-card", &tone(node)], context),
            id_attr(node),
            icon(node),
            heading(node, "h3"),
            body(node)
        ),
        "button" => format!(
            "<button class=\"{}\"{}{}>{}</button>",
            component_classes(node, &["ps-button", &variant(node)], context),
            id_attr(node),
            action_attrs(node),
            escape_html(string_attr(node, "label").unwrap_or("Button"))
        ),
        "text" => format!(
            "<div class=\"{}\"{}>{}{}{children}</div>",
            component_classes(node, &["ps-text"], context),
            id_attr(node),
            heading(node, "h3"),
            body(node)
        ),
        "image" => format!(
            "<figure class=\"{}\"{}><img src=\"{}\" alt=\"{}\">{}</figure>",
            component_classes(node, &["ps-image"], context),
            id_attr(node),
            escape_attr(string_attr(node, "src").unwrap_or("")),
            escape_attr(string_attr(node, "alt").unwrap_or("")),
            caption(node)
        ),
        "modal" => format!(
            "<dialog class=\"ps-modal\" id=\"{}\">\n  <form method=\"dialog\"><button class=\"ps-modal-close\" aria-label=\"Close\">x</button></form>\n  {}{}{children}\n</dialog>",
            escape_attr(string_attr(node, "id").unwrap_or("")),
            heading(node, "h2"),
            body(node)
        ),
        "form" => format!(
            "<form class=\"{}\"{} action=\"{}\" method=\"{}\">{children}</form>",
            component_classes(node, &["ps-form"], context),
            id_attr(node),
            escape_attr(string_attr(node, "action").unwrap_or("#")),
            escape_attr(string_attr(node, "method").unwrap_or("post"))
        ),
        "input" => format!(
            "<label class=\"ps-field\"><span>{}</span><input name=\"{}\" type=\"{}\" placeholder=\"{}\"></label>",
            escape_html(string_attr(node, "label").unwrap_or("")),
            escape_attr(string_attr(node, "name").unwrap_or("")),
            escape_attr(string_attr(node, "kind").unwrap_or("text")),
            escape_attr(string_attr(node, "placeholder").unwrap_or(""))
        ),
        "metric" => render_metric(node, context),
        "log" => render_log(node),
        _ => children,
    }
}

fn render_scene(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let children = node
        .children
        .iter()
        .map(|node| render_node(node, context))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<section class=\"{}\"{}>\n  <div class=\"ps-container\">\n    <header class=\"ps-scene-header\">{}{}</header>\n    <div class=\"ps-scene-body\">{children}</div>\n  </div>\n</section>",
        component_classes(
            node,
            &[
                "ps-scene",
                &format!("ps-scene-{}", layout_mode(node).unwrap_or("split"))
            ],
            context
        ),
        id_attr(node),
        heading(node, "h2"),
        body(node)
    )
}

fn render_panel(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let graph = node
        .graph
        .as_ref()
        .map(|graph| render_graph(graph, context))
        .unwrap_or_default();
    let children = node
        .children
        .iter()
        .map(|node| render_node(node, context))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<article class=\"{}\"{}>\n  <header>{}{}</header>\n  {}{}\n</article>",
        component_classes(node, &["ps-panel", &tone(node)], context),
        id_attr(node),
        heading(node, "h3"),
        body(node),
        graph,
        children
    )
}

fn render_graph(graph: &GraphIr, context: &RenderContext<'_>) -> String {
    let node_map = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let edge_markup = graph
        .edges
        .iter()
        .filter_map(|edge| {
            let from = node_map.get(edge.from.as_str())?;
            let to = node_map.get(edge.to.as_str())?;
            Some(format!(
                "<path class=\"{}\" d=\"{}\" />",
                graph_effect_class(edge.effect.as_deref(), context),
                graph_edge_path(from, to)
            ))
        })
        .collect::<Vec<_>>()
        .join("\n");
    let node_markup = graph
        .nodes
        .iter()
        .map(|node| {
            format!(
                "<g class=\"{}\" data-ps-node=\"{}\" data-ps-state-value=\"{}\" transform=\"translate({}, {})\" tabindex=\"0\" role=\"button\" aria-label=\"{}\"><rect x=\"-74\" y=\"-29\" width=\"148\" height=\"58\" rx=\"16\"/><text class=\"ps-graph-icon\" x=\"-56\" y=\"5\">{}</text><text class=\"ps-graph-label\" x=\"-26\" y=\"5\">{}</text><circle cx=\"62\" cy=\"-19\" r=\"5\"/></g>",
                graph_node_class(node, context),
                escape_attr(&node.id),
                escape_attr(&node.id),
                node.x,
                node.y,
                escape_attr(&node.label),
                escape_html(node.icon.as_deref().unwrap_or("")),
                escape_html(&node.label)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<svg class=\"ps-graph\" viewBox=\"0 0 760 380\" role=\"img\" aria-label=\"Data lineage graph\">\n  <defs><marker id=\"ps-arrow\" markerWidth=\"8\" markerHeight=\"8\" refX=\"7\" refY=\"4\" orient=\"auto\"><path d=\"M0,0 L8,4 L0,8 z\" fill=\"#8aa89b\" /></marker></defs>\n  <g class=\"ps-edges\">{edge_markup}</g>\n  <g class=\"ps-nodes\">{node_markup}</g>\n</svg>"
    )
}

fn graph_edge_path(from: &GraphNodeIr, to: &GraphNodeIr) -> String {
    let dx = to.x - from.x;
    let dy = to.y - from.y;

    let (start, end, control_a, control_b) = if dx.abs() < GRAPH_NODE_WIDTH && dy != 0 {
        let vertical_direction = dy.signum();
        let start = GraphPoint {
            x: from.x,
            y: from.y + (vertical_direction * GRAPH_NODE_HALF_HEIGHT),
        };
        let end = GraphPoint {
            x: to.x,
            y: to.y - (vertical_direction * GRAPH_NODE_HALF_HEIGHT),
        };
        let control_offset = (end.y - start.y).abs().max(72) / 2;
        (
            start,
            end,
            GraphPoint {
                x: from.x,
                y: from.y + (vertical_direction * control_offset),
            },
            GraphPoint {
                x: to.x,
                y: to.y - (vertical_direction * control_offset),
            },
        )
    } else {
        let horizontal_direction = if dx >= 0 { 1 } else { -1 };
        let start = GraphPoint {
            x: from.x + (horizontal_direction * GRAPH_NODE_HALF_WIDTH),
            y: from.y,
        };
        let end = GraphPoint {
            x: to.x - (horizontal_direction * GRAPH_NODE_HALF_WIDTH),
            y: to.y,
        };
        let control_offset = (end.x - start.x).abs().max(96) / 2;
        (
            start,
            end,
            GraphPoint {
                x: start.x + (horizontal_direction * control_offset),
                y: start.y,
            },
            GraphPoint {
                x: end.x - (horizontal_direction * control_offset),
                y: end.y,
            },
        )
    };

    format!(
        "M {} {} C {} {}, {} {}, {} {}",
        start.x, start.y, control_a.x, control_a.y, control_b.x, control_b.y, end.x, end.y
    )
}

fn render_metric(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    format!(
        "<div class=\"{}\"{} data-ps-metric><span>{}</span><strong>{}</strong></div>",
        component_classes(node, &["ps-metric", &tone(node)], context),
        id_attr(node),
        escape_html(string_attr(node, "label").unwrap_or("Metric")),
        escape_html(string_attr(node, "value").unwrap_or("0"))
    )
}

fn render_log(node: &ComponentIr) -> String {
    let max = number_attr(node, "max", 5);
    let source = string_attr(node, "source").unwrap_or("events");
    let items = (1..=max.min(6))
        .map(|index| {
            format!(
                "<li><span>{}</span><strong>{}</strong></li>",
                escape_html(source),
                escape_html(&format!("event {}", index))
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<ol class=\"ps-log\" data-source=\"{}\">{items}</ol>",
        escape_attr(source)
    )
}

fn render_markdown(node: &MarkdownIr) -> String {
    node.value
        .split("\n\n")
        .filter_map(|block| {
            let text = block.trim();
            if text.is_empty() {
                None
            } else if let Some(heading) = text.strip_prefix("# ") {
                Some(format!("<h1>{}</h1>", escape_html(heading)))
            } else if let Some(heading) = text.strip_prefix("## ") {
                Some(format!("<h2>{}</h2>", escape_html(heading)))
            } else {
                Some(format!("<p>{}</p>", escape_html(text)))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn heading(node: &ComponentIr, tag: &str) -> String {
    string_attr(node, "heading")
        .or_else(|| string_attr(node, "title"))
        .map(|text| format!("<{tag}>{}</{tag}>", escape_html(text)))
        .unwrap_or_default()
}

fn body(node: &ComponentIr) -> String {
    string_attr(node, "body")
        .map(|text| format!("<p>{}</p>", escape_html(text)))
        .unwrap_or_default()
}

fn icon(node: &ComponentIr) -> String {
    string_attr(node, "icon")
        .map(|text| format!("<div class=\"ps-icon\">{}</div>", escape_html(text)))
        .unwrap_or_default()
}

fn caption(node: &ComponentIr) -> String {
    string_attr(node, "caption")
        .map(|text| format!("<figcaption>{}</figcaption>", escape_html(text)))
        .unwrap_or_default()
}

fn action_attrs(node: &ComponentIr) -> String {
    let Some(action) = string_attr(node, "action") else {
        return String::new();
    };
    let target = string_attr(node, "target")
        .map(|target| format!(" data-target=\"{}\"", escape_attr(target)))
        .unwrap_or_default();
    format!(" data-action=\"{}\"{}", escape_attr(action), target)
}

fn id_attr(node: &ComponentIr) -> String {
    string_attr(node, "id")
        .map(|id| format!(" id=\"{}\"", escape_attr(id)))
        .unwrap_or_default()
}

fn component_classes(node: &ComponentIr, base: &[&str], context: &RenderContext<'_>) -> String {
    let mut values = base
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    if let Some(custom) = string_attr(node, "class") {
        values.push(custom.to_string());
    }
    if let Some(layout) = &node.layout
        && let Some(density) = &layout.density
    {
        values.push(format!("ps-density-{density}"));
    }
    if let Some(effect) = string_attr(node, "effect") {
        values.push(effect_class(effect, context));
    }
    classes_owned(&values)
}

fn graph_node_class(node: &GraphNodeIr, context: &RenderContext<'_>) -> String {
    let mut values = vec![
        "ps-graph-node".to_string(),
        format!("ps-status-{}", node.status),
    ];
    if let Some(effect) = &node.effect {
        values.push(effect_class(effect, context));
    }
    classes_owned(&values)
}

fn graph_effect_class(effect: Option<&str>, context: &RenderContext<'_>) -> String {
    let mut values = vec!["ps-graph-edge".to_string()];
    if let Some(effect) = effect {
        values.push(effect_class(effect, context));
    }
    classes_owned(&values)
}

fn effect_class(effect_id: &str, context: &RenderContext<'_>) -> String {
    match context
        .page
        .effects
        .get(effect_id)
        .map(|effect| effect.effect_type.as_str())
    {
        Some("flow") => "ps-effect-flow".to_string(),
        Some("pulse") => "ps-effect-pulse".to_string(),
        Some("glow") => "ps-effect-glow".to_string(),
        Some("count-up") => "ps-effect-count-up".to_string(),
        Some("reveal") => "ps-effect-reveal".to_string(),
        _ => format!("ps-effect-{}", sanitize_class(effect_id)),
    }
}

fn string_attr<'a>(node: &'a ComponentIr, key: &str) -> Option<&'a str> {
    node.attributes.get(key).and_then(AttributeValue::as_str)
}

fn number_attr(node: &ComponentIr, key: &str, fallback: i64) -> i64 {
    node.attributes
        .get(key)
        .and_then(AttributeValue::as_i64)
        .unwrap_or(fallback)
}

fn layout_mode(node: &ComponentIr) -> Option<&str> {
    node.layout
        .as_ref()
        .and_then(|layout| layout.mode.as_deref())
}

fn columns(node: &ComponentIr, fallback: i64) -> i64 {
    node.layout
        .as_ref()
        .and_then(|layout| layout.columns)
        .unwrap_or_else(|| number_attr(node, "columns", fallback))
}

fn tone(node: &ComponentIr) -> String {
    format!("ps-tone-{}", string_attr(node, "tone").unwrap_or("default"))
}

fn spacing(node: &ComponentIr) -> String {
    format!("ps-space-{}", string_attr(node, "spacing").unwrap_or("lg"))
}

fn gap(node: &ComponentIr) -> String {
    let gap = node
        .layout
        .as_ref()
        .and_then(|layout| layout.gap.as_deref())
        .or_else(|| string_attr(node, "gap"))
        .unwrap_or("md");
    format!("ps-gap-{gap}")
}

fn variant(node: &ComponentIr) -> String {
    format!(
        "ps-button-{}",
        string_attr(node, "variant").unwrap_or("primary")
    )
}

fn classes_owned(values: &[String]) -> String {
    values
        .iter()
        .filter(|value| !value.is_empty())
        .map(|value| escape_attr(value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_class(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value).replace('\'', "&#39;")
}

fn escape_script_json(value: &str) -> String {
    value.replace("</", "<\\/")
}

fn escape_css_value(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !matches!(ch, ';' | '{' | '}'))
        .collect()
}

fn base_css() -> &'static str {
    r#":root{color-scheme:light;--ps-bg:#f7f7f2;--ps-ink:#202124;--ps-muted:#62645f;--ps-line:#deded4;--ps-accent:#116149;--ps-accent-ink:#fff;--ps-panel:#fff;--ps-radius:12px;font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}
*{box-sizing:border-box}body{margin:0;background:var(--ps-bg);color:var(--ps-ink);line-height:1.5}h1,h2,h3,p{margin:0}h1{font-size:clamp(2.4rem,6vw,5rem);line-height:.98;max-width:11ch}h2{font-size:clamp(1.8rem,3vw,3rem);line-height:1.05}h3{font-size:1.1rem}.ps-container{width:min(1180px,calc(100% - 32px));margin:0 auto}.ps-hero{min-height:72vh;display:flex;align-items:center}.ps-hero-inner{display:grid;gap:24px}.ps-section{padding:72px 0}.ps-space-sm{padding-block:32px}.ps-space-md{padding-block:56px}.ps-space-lg{padding-block:88px}.ps-space-xl{padding-block:120px}.ps-density-compact{--ps-density-pad:16px;--ps-density-gap:12px}.ps-density-spacious{--ps-density-pad:32px;--ps-density-gap:24px}.ps-tone-dark{background:#16201d;color:#f7f7f2}.ps-tone-accent{background:#dcefe7}.ps-tone-muted{background:#ecece3}.ps-tone-good{background:#e3f8ee;color:#12382a}.ps-stack{display:grid}.ps-gap-sm{gap:12px}.ps-gap-md{gap:20px}.ps-gap-lg{gap:32px}.ps-grid{display:grid;grid-template-columns:repeat(var(--ps-columns),minmax(0,1fr));gap:24px}.ps-card,.ps-panel{background:var(--ps-panel);border:1px solid var(--ps-line);border-radius:var(--ps-radius);padding:var(--ps-density-pad,24px);display:grid;gap:var(--ps-density-gap,16px);box-shadow:0 1px 1px rgba(0,0,0,.04)}.ps-icon{font-size:1.4rem}.ps-button{width:max-content;border:0;border-radius:999px;padding:12px 18px;font:inherit;font-weight:700;cursor:pointer}.ps-button-primary{background:var(--ps-accent);color:var(--ps-accent-ink)}.ps-button-secondary{background:#fff;color:var(--ps-ink);border:1px solid var(--ps-line)}.ps-text{display:grid;gap:12px}.ps-image img{width:100%;height:auto;border-radius:var(--ps-radius)}.ps-modal{border:0;border-radius:16px;padding:28px;max-width:640px}.ps-modal::backdrop{background:rgba(0,0,0,.45)}.ps-modal-close{float:right}.ps-form{display:grid;gap:16px}.ps-field{display:grid;gap:6px}.ps-field input{font:inherit;padding:12px;border:1px solid var(--ps-line);border-radius:8px}.ps-scene{padding:88px 0;background:linear-gradient(135deg,#f8faf6,#e9f1ec)}.ps-scene-header{display:grid;gap:12px;margin-bottom:28px}.ps-scene-body{display:grid;gap:24px}.ps-scene-split .ps-scene-body{grid-template-columns:minmax(0,1.4fr) minmax(320px,.8fr);align-items:start}.ps-graph{width:100%;min-height:380px;overflow:visible}.ps-graph-edge{fill:none;stroke:#8aa89b;stroke-width:3;marker-end:url(#ps-arrow);stroke-linecap:round}.ps-graph-node{cursor:pointer}.ps-graph-node rect{fill:#fff;stroke:#b9cbc3;stroke-width:1.5;filter:drop-shadow(0 8px 16px rgba(17,97,73,.10))}.ps-graph-node text{font-size:13px;dominant-baseline:middle}.ps-graph-label{font-weight:700}.ps-graph-icon{font-size:16px}.ps-status-active rect{stroke:#116149}.ps-status-syncing rect{stroke:#3976d8}.ps-status-ready rect{stroke:#7b4fc5}.ps-status-warning rect{stroke:#b7791f}.ps-metric{display:grid;gap:4px;padding:16px;border:1px solid var(--ps-line);border-radius:12px;background:#fff}.ps-metric span{font-size:.82rem;color:var(--ps-muted);font-weight:700;text-transform:uppercase;letter-spacing:.08em}.ps-metric strong{font-size:1.7rem}.ps-log{display:grid;gap:8px;margin:0;padding:0;list-style:none}.ps-log li{display:flex;justify-content:space-between;gap:16px;padding:10px 12px;border:1px solid var(--ps-line);border-radius:10px;background:#fff}.ps-effect-flow{stroke-dasharray:10 10;animation:ps-flow 1.4s linear infinite}.ps-effect-pulse{animation:ps-pulse 1.6s ease-in-out infinite;transform-box:fill-box;transform-origin:center}.ps-graph-node.ps-effect-pulse{animation:ps-pulse-svg 1.6s ease-in-out infinite}.ps-effect-glow rect,.ps-effect-glow{filter:drop-shadow(0 0 12px rgba(17,97,73,.35))}.ps-effect-reveal{animation:ps-reveal .7s ease-out both}.ps-effect-count-up strong{animation:ps-pulse 1.2s ease-in-out 2}@keyframes ps-flow{to{stroke-dashoffset:-40}}@keyframes ps-pulse{50%{opacity:.62;filter:brightness(1.08)}}@keyframes ps-pulse-svg{50%{opacity:.64}}@keyframes ps-reveal{from{opacity:0;transform:translateY(12px)}to{opacity:1;transform:translateY(0)}}@media(max-width:860px){.ps-grid,.ps-scene-split .ps-scene-body{grid-template-columns:1fr}.ps-hero{min-height:auto}.ps-space-lg,.ps-space-xl{padding-block:56px}}"#
}

fn base_js() -> &'static str {
    r#"(()=>{const cfg=JSON.parse(document.getElementById("ps-runtime-config")?.textContent||"{\"states\":[],\"events\":[]}");const state={};for(const item of cfg.states||[]){state[item.id]=item.default;document.body.dataset["state"+item.id.charAt(0).toUpperCase()+item.id.slice(1)]=item.default}function setState(id,value){state[id]=value;document.body.dataset["state"+id.charAt(0).toUpperCase()+id.slice(1)]=value;document.querySelectorAll("[data-ps-node]").forEach(node=>node.classList.toggle("is-selected",node.dataset.psStateValue===value))}document.addEventListener("click",(event)=>{const button=event.target.closest("[data-action]");if(button){const action=button.dataset.action;const target=button.dataset.target;if(action==="open-modal"&&target){document.getElementById(target)?.showModal?.()}if(action==="toggle"&&target){document.getElementById(target)?.toggleAttribute("hidden")}}const graphNode=event.target.closest("[data-ps-node]");if(graphNode){for(const rule of cfg.events||[]){if(rule.on==="node.click"){const value=rule.value==="$node.id"?graphNode.dataset.psStateValue:rule.value;setState(rule.set,value)}}}});for(const [id,value] of Object.entries(state)){setState(id,value)}})()"#
}
