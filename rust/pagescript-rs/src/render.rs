use std::collections::HashMap;

use serde_json::{Value, json};

use crate::{
    ir::{ComponentIr, GraphIr, GraphNodeIr, IrNode, MarkdownIr, PageIr, compile_page_ir},
    resolver::Resolver,
    types::{AttributeValue, DocumentNode},
};

pub fn render_to_html(
    document: &DocumentNode,
    page_id: Option<&str>,
    resolver: &Resolver,
) -> Result<String, String> {
    let page = compile_page_ir(document, page_id, resolver)?;
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
    let mut declarations = page
        .tokens
        .iter()
        .flat_map(|(key, value)| token_declarations(key, value))
        .collect::<Vec<_>>()
        .join("");
    let has_accent_ink = page
        .tokens
        .keys()
        .any(|key| token_alias(key) == Some("--ps-accent-ink"));
    let light_accent = page
        .tokens
        .iter()
        .filter(|(key, _)| token_alias(key) == Some("--ps-accent"))
        .filter_map(|(_, value)| value.as_str())
        .next_back()
        .and_then(relative_luminance)
        .is_some_and(prefers_dark_text);
    if light_accent && !has_accent_ink {
        declarations.push_str("--ps-accent-ink:#18263b;");
    }
    format!(":root{{{declarations}}}")
}

/// Relative luminance of a `#rgb` or `#rrggbb` color, per WCAG 2.x.
fn relative_luminance(color: &str) -> Option<f64> {
    let hex = color.trim().strip_prefix('#')?;
    let expanded = match hex.len() {
        3 => hex.chars().flat_map(|ch| [ch, ch]).collect::<String>(),
        6 => hex.to_string(),
        _ => return None,
    };
    let channel = |index: usize| -> Option<f64> {
        let value =
            f64::from(u8::from_str_radix(expanded.get(index..index + 2)?, 16).ok()?) / 255.0;
        Some(if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        })
    };
    Some(0.2126 * channel(0)? + 0.7152 * channel(2)? + 0.0722 * channel(4)?)
}

/// True when the default dark ink (#18263b) contrasts better than white.
fn prefers_dark_text(luminance: f64) -> bool {
    const INK_LUMINANCE: f64 = 0.0187;
    (luminance + 0.05) / (INK_LUMINANCE + 0.05) > 1.05 / (luminance + 0.05)
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
        "el" => render_el(node, context),
        "slot" => children,
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
        "nav" => render_nav(node, context),
        "nav-item" => format!(
            "<a class=\"ps-nav-item\" href=\"{}\">{}</a>",
            escape_attr(string_attr(node, "href").unwrap_or("#")),
            escape_html(string_attr(node, "label").unwrap_or("Link"))
        ),
        "button" => format!(
            "<button class=\"{}\"{}{}>{}</button>",
            component_classes(node, &["ps-button", &variant(node)], context),
            id_attr(node),
            action_attrs(node),
            escape_html(string_attr(node, "label").unwrap_or("Button"))
        ),
        "text" => string_attr(node, "value")
            .map(escape_html)
            .unwrap_or_else(|| {
                format!(
                    "<div class=\"{}\"{}>{}{}{children}</div>",
                    component_classes(node, &["ps-text"], context),
                    id_attr(node),
                    heading(node, "h3"),
                    body(node)
                )
            }),
        "image" => format!(
            "<figure class=\"{}\"{}><img src=\"{}\" alt=\"{}\">{}</figure>",
            component_classes(node, &["ps-image"], context),
            id_attr(node),
            escape_attr(string_attr(node, "src").unwrap_or("")),
            escape_attr(string_attr(node, "alt").unwrap_or("")),
            caption(node)
        ),
        "attr" | "bind" | "on" => String::new(),
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
        "filter" => render_filter(node, context),
        "table" => render_table(node, context),
        "row" => render_row(node, context),
        "cell" => render_cell(node, context),
        "column" => String::new(),
        "empty-state" => format!(
            "<section class=\"{}\"{}>{}{}{children}</section>",
            component_classes(node, &["ps-empty-state"], context),
            id_attr(node),
            icon(node),
            heading(node, "h3")
        ),
        "metric" => render_metric(node, context),
        "log" => render_log(node),
        _ => children,
    }
}

fn render_nav(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let children = node
        .children
        .iter()
        .map(|node| render_node(node, context))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<nav class=\"{}\"{} aria-label=\"{}\"><div class=\"ps-container ps-nav-inner\">{children}</div></nav>",
        component_classes(node, &["ps-nav"], context),
        id_attr(node),
        escape_attr(string_attr(node, "label").unwrap_or("Page navigation"))
    )
}

fn render_filter(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let kind = string_attr(node, "kind").unwrap_or("search");
    format!(
        "<label class=\"{}\"{}><span>{}</span><input name=\"{}\" type=\"{}\" placeholder=\"{}\"></label>",
        component_classes(node, &["ps-filter"], context),
        id_attr(node),
        escape_html(string_attr(node, "label").unwrap_or("Filter")),
        escape_attr(string_attr(node, "id").unwrap_or("filter")),
        escape_attr(kind),
        escape_attr(string_attr(node, "placeholder").unwrap_or("Filter"))
    )
}

fn render_table(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let header = node
        .children
        .iter()
        .filter_map(|child| match child {
            IrNode::Component(component) if component.name == "column" => Some(format!(
                "<th scope=\"col\">{}</th>",
                escape_html(string_attr(component, "label").unwrap_or(""))
            )),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    let rows = node
        .children
        .iter()
        .filter_map(|child| match child {
            IrNode::Component(component) if component.name == "row" => {
                Some(render_row(component, context))
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    table_markup(
        &component_classes(node, &["ps-table"], context),
        &id_attr(node),
        &header,
        &rows,
    )
}

fn table_markup(classes: &str, id: &str, header: &str, rows: &str) -> String {
    let head = if header.is_empty() {
        String::new()
    } else {
        format!("<thead><tr>{header}</tr></thead>")
    };

    format!("<div class=\"{classes}\"{id}><table>{head}<tbody>{rows}</tbody></table></div>")
}

fn render_row(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let cells = node
        .children
        .iter()
        .filter_map(|child| match child {
            IrNode::Component(component) if component.name == "cell" => {
                Some(render_cell(component, context))
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<tr>{cells}</tr>")
}

fn render_cell(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let value = string_attr(node, "value")
        .map(escape_html)
        .unwrap_or_else(|| {
            node.children
                .iter()
                .map(|child| render_node(child, context))
                .collect::<Vec<_>>()
                .join("\n")
        });
    format!("<td>{value}</td>")
}

fn render_el(node: &ComponentIr, context: &RenderContext<'_>) -> String {
    let tag = string_attr(node, "tag")
        .filter(|tag| is_safe_tag(tag))
        .unwrap_or("div");
    let attrs = generic_attrs(node);
    if is_void_tag(tag) {
        return format!("<{tag}{attrs}>");
    }
    let children = node
        .children
        .iter()
        .filter_map(|child| match child {
            IrNode::Component(component)
                if matches!(component.name.as_str(), "attr" | "bind" | "on") =>
            {
                None
            }
            _ => Some(render_node(child, context)),
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<{tag}{attrs}>{children}</{tag}>")
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
                Some(
                    render_markdown_list(block)
                        .or_else(|| render_markdown_table(text))
                        .unwrap_or_else(|| format!("<p>{}</p>", escape_html(text))),
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_markdown_list(block: &str) -> Option<String> {
    let mut lines = block
        .trim_end()
        .lines()
        .skip_while(|line| line.trim().is_empty());
    let first_line = lines.next()?;
    let first_text = first_line.trim_start();
    let indent = first_line.len() - first_text.len();
    let (start, first_item) = list_item(first_text)?;
    let mut items = format!("<li>{}</li>", escape_html(first_item));
    for line in lines {
        let text = line.trim_start();
        let (number, item) = list_item(text)?;
        if line.len() - text.len() != indent || number.is_some() != start.is_some() {
            return None;
        }
        items.push_str(&format!("<li>{}</li>", escape_html(item)));
    }

    Some(match start {
        None => format!("<ul>{items}</ul>"),
        Some(1) => format!("<ol>{items}</ol>"),
        Some(start) => format!("<ol start=\"{start}\">{items}</ol>"),
    })
}

fn list_item(line: &str) -> Option<(Option<u32>, &str)> {
    let (number, text) = if let Some(text) = line.strip_prefix("- ") {
        (None, text)
    } else {
        let (digits, text) = line.split_once(". ")?;
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        (Some(digits.parse().ok()?), text)
    };
    let text = text.trim();
    (!text.is_empty()).then_some((number, text))
}

fn render_markdown_table(block: &str) -> Option<String> {
    let mut lines = block.lines();
    let columns = pipe_cells(lines.next()?)?;
    let separator = pipe_cells(lines.next()?)?;
    if separator.len() != columns.len()
        || !separator
            .iter()
            .all(|cell| cell.len() >= 3 && cell.bytes().all(|byte| byte == b'-'))
    {
        return None;
    }

    let header = columns
        .iter()
        .map(|column| format!("<th scope=\"col\">{}</th>", escape_html(column)))
        .collect::<String>();
    let mut rows = Vec::new();
    for line in lines {
        let cells = pipe_cells(line)?;
        if cells.len() != columns.len() {
            return None;
        }
        let cells = cells
            .iter()
            .map(|cell| format!("<td>{}</td>", escape_html(cell)))
            .collect::<String>();
        rows.push(format!("<tr>{cells}</tr>"));
    }
    Some(table_markup("ps-table", "", &header, &rows.join("\n")))
}

fn pipe_cells(line: &str) -> Option<Vec<String>> {
    let mut chars = line.trim().strip_prefix('|')?.chars().peekable();
    let mut cells = Vec::new();
    let mut cell = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' if matches!(chars.peek(), Some('|' | '\\')) => {
                cell.push(chars.next().expect("escaped character"));
            }
            '|' => {
                cells.push(cell.trim().to_string());
                cell.clear();
            }
            _ => cell.push(ch),
        }
    }
    (cell.is_empty() && !cells.is_empty()).then_some(cells)
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

fn generic_attrs(node: &ComponentIr) -> String {
    let mut attrs = node
        .attributes
        .iter()
        .filter_map(|(name, value)| {
            if matches!(
                name.as_str(),
                "tag" | "layout" | "density" | "gap" | "columns" | "align"
            ) {
                None
            } else {
                render_attr(name, value)
            }
        })
        .collect::<Vec<_>>();

    for child in &node.children {
        let IrNode::Component(component) = child else {
            continue;
        };
        match component.name.as_str() {
            "attr" => {
                if let (Some(name), Some(value)) = (
                    string_attr(component, "name"),
                    component.attributes.get("value"),
                ) && let Some(rendered) = render_attr(name, value)
                {
                    attrs.push(rendered);
                }
            }
            "bind" => {
                if let Some(state) = string_attr(component, "state") {
                    attrs.push(format!("data-ps-bind=\"{}\"", escape_attr(state)));
                }
                if let Some(target) = string_attr(component, "target") {
                    attrs.push(format!("data-ps-bind-target=\"{}\"", escape_attr(target)));
                }
            }
            "on" => {
                if let Some(event) = string_attr(component, "event") {
                    attrs.push(format!("data-ps-on=\"{}\"", escape_attr(event)));
                }
                if let Some(action) = string_attr(component, "action") {
                    attrs.push(format!("data-ps-action=\"{}\"", escape_attr(action)));
                }
                if let Some(target) = string_attr(component, "target") {
                    attrs.push(format!("data-ps-target=\"{}\"", escape_attr(target)));
                }
            }
            _ => {}
        }
    }

    if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    }
}

fn render_attr(name: &str, value: &Value) -> Option<String> {
    if !is_safe_attr_name(name) {
        return None;
    }
    match value {
        Value::Bool(true) => Some(escape_attr(name)),
        Value::Bool(false) | Value::Null => None,
        Value::String(value) => Some(format!("{}=\"{}\"", escape_attr(name), escape_attr(value))),
        Value::Number(value) => Some(format!("{}=\"{}\"", escape_attr(name), value)),
        _ => Some(format!(
            "{}=\"{}\"",
            escape_attr(name),
            escape_attr(&value.to_string())
        )),
    }
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

fn is_safe_tag(value: &str) -> bool {
    !matches!(value, "script" | "iframe" | "object" | "embed")
        && !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn is_void_tag(value: &str) -> bool {
    matches!(
        value,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn is_safe_attr_name(value: &str) -> bool {
    !value.is_empty()
        && !value.eq_ignore_ascii_case("srcdoc")
        && !value.to_ascii_lowercase().starts_with("on")
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':'))
}

fn base_css() -> &'static str {
    r#":root{color-scheme:light;--ps-bg:#f5f7fa;--ps-panel:#fff;--ps-ink:#18263b;--ps-muted:#4f5d72;--ps-line:#d3dbe6;--ps-accent:#2b5a96;--ps-accent-ink:#fff;--ps-radius:8px;--ps-shadow:none;--ps-good:#2d7650;--ps-warning:#a9690f;--ps-tint:color-mix(in srgb,var(--ps-ink) 5%,var(--ps-panel));--ps-control:color-mix(in srgb,var(--ps-ink) 32%,var(--ps-panel));--ps-link:color-mix(in srgb,var(--ps-accent) 82%,var(--ps-ink));font-family:system-ui,-apple-system,"Segoe UI",Roboto,sans-serif}
*{box-sizing:border-box}
body{margin:0;background:var(--ps-bg);color:var(--ps-ink);line-height:1.55}
h1,h2,h3,p,figure{margin:0}
h1,h2,h3{line-height:1.15;text-wrap:balance}
h1{max-width:20ch;font-size:clamp(2.25rem,5vw,3.5rem);line-height:1.05;letter-spacing:-.02em}
h2{font-size:clamp(1.5rem,2.6vw,2rem)}
h3{font-size:1.125rem}
p{max-width:68ch;text-wrap:pretty}
a{color:var(--ps-link);text-underline-offset:.2em}
a:hover{color:var(--ps-ink)}
:focus-visible{outline:2px solid var(--ps-link);outline-offset:2px}
.ps-container{width:min(1120px,100% - 2*clamp(16px,4vw,40px));margin:0 auto}
.ps-hero-inner,.ps-section>.ps-container,.ps-scene-header{display:grid;gap:12px}
.ps-section>.ps-container>:not(h1,h2,p){margin-top:16px}
:is(.ps-hero-inner,.ps-section>.ps-container,.ps-scene-header)>p{color:var(--ps-muted)}
.ps-hero-inner>p{font-size:1.125rem}
.ps-space-sm{padding-block:24px}
.ps-space-md{padding-block:40px}
.ps-space-lg,.ps-scene{padding-block:64px}
.ps-space-xl{padding-block:clamp(64px,10vw,112px)}
:is(.ps-hero,.ps-section).ps-tone-default+.ps-section.ps-tone-default{padding-top:0}
.ps-density-compact{--ps-density-pad:14px;--ps-density-gap:8px}
.ps-density-spacious{--ps-density-pad:28px;--ps-density-gap:20px}
.ps-stack,.ps-grid,.ps-text,.ps-image,.ps-form{display:grid;align-content:start;gap:16px}
.ps-grid{grid-template-columns:repeat(var(--ps-columns,2),minmax(0,1fr))}
.ps-gap-sm{gap:8px}
.ps-gap-lg{gap:28px}
.ps-card,.ps-panel{display:grid;align-content:start;gap:var(--ps-density-gap,12px);padding:var(--ps-density-pad,20px);border:1px solid var(--ps-line);border-radius:var(--ps-radius);background:var(--ps-tone-bg,var(--ps-panel));color:var(--ps-ink)}
.ps-card{gap:8px}
.ps-card>p,.ps-panel>header>p{color:var(--ps-muted)}
.ps-panel>header{display:grid;gap:4px}
.ps-panel>header:empty,.ps-scene-header:empty{display:none}
:is(.ps-panel,.ps-card) .ps-card.ps-tone-default{padding:12px 0 0;border:0;border-top:1px solid var(--ps-line);border-radius:0;background:none}
.ps-icon{color:var(--ps-link);font-weight:700}
:is(.ps-hero,.ps-section):not(.ps-tone-default,.ps-tone-neutral){background:var(--ps-tone-bg)}
.ps-tone-default,.ps-tone-neutral{--ps-tone-bg:initial;--ps-tone-mark:initial}
.ps-tone-accent{--ps-tone-bg:color-mix(in srgb,var(--ps-accent) 9%,var(--ps-panel));--ps-tone-mark:var(--ps-accent)}
.ps-tone-muted{--ps-tone-bg:var(--ps-tint);--ps-tone-mark:var(--ps-muted)}
.ps-tone-good,.ps-tone-positive{--ps-tone-bg:#e8f3ec;--ps-tone-mark:var(--ps-good)}
.ps-tone-warning{--ps-tone-bg:#fbf1dc;--ps-tone-mark:var(--ps-warning)}
.ps-tone-dark{--ps-tone-bg:#18263b;--ps-tone-mark:var(--ps-accent);--ps-muted:#b9c3d1;--ps-line:#3a4a61;--ps-link:#f5f7fa;color:#f5f7fa}
.ps-tone-dark :is(.ps-card,.ps-panel){--ps-tone-bg:#ffffff12;color:inherit}
.ps-nav{border-bottom:1px solid var(--ps-line);background:var(--ps-panel)}
.ps-nav-inner{display:flex;flex-wrap:wrap;align-items:center;gap:0 24px;min-height:56px}
.ps-nav-item{display:inline-flex;align-items:center;min-height:44px;color:inherit;font-weight:600;text-decoration:none}
.ps-nav-item:hover{text-decoration:underline}
.ps-button{justify-self:start;max-width:100%;min-height:44px;padding:0 16px;border:1px solid var(--ps-control);border-radius:6px;background:var(--ps-panel);color:var(--ps-ink);font:inherit;font-weight:600;cursor:pointer;transition:background-color .15s}
.ps-button:hover{background:var(--ps-tint)}
.ps-button-primary{border-color:transparent;background:var(--ps-accent);color:var(--ps-accent-ink)}
.ps-button-primary:hover{background:color-mix(in srgb,var(--ps-accent) 86%,var(--ps-ink))}
.ps-image img{display:block;width:100%;height:auto;border-radius:var(--ps-radius)}
.ps-modal{width:min(640px,100% - 32px);padding:24px;border:1px solid var(--ps-line);border-radius:12px;color:var(--ps-ink)}
.ps-modal::backdrop{background:rgb(24 38 59/.5)}
.ps-modal-close{float:right;width:44px;height:44px;margin:-12px -12px 0 12px;border:0;border-radius:6px;background:none;color:inherit;font-size:1.25rem;cursor:pointer}
.ps-modal-close:hover{background:var(--ps-tint)}
.ps-form{max-width:36rem}
.ps-field,.ps-filter{display:grid;gap:6px;font-weight:600}
.ps-field input,.ps-filter input{width:100%;min-height:44px;padding:8px 12px;border:1px solid var(--ps-control);border-radius:6px;background:var(--ps-panel);color:var(--ps-ink);font:inherit;font-weight:400}
.ps-table{overflow-x:auto;border:1px solid var(--ps-line);border-radius:var(--ps-radius);background:var(--ps-panel);color:var(--ps-ink)}
.ps-table table{width:100%;border-collapse:collapse;font-variant-numeric:tabular-nums}
.ps-table th,.ps-table td{padding:10px 14px;border-bottom:1px solid var(--ps-line);text-align:left;vertical-align:top}
.ps-table th{background:var(--ps-tint);font-size:.875rem;font-weight:600}
.ps-table tr:last-child td{border-bottom:0}
.ps-empty-state{display:grid;justify-items:center;gap:8px;padding:32px 24px;border:1px dashed var(--ps-control);border-radius:var(--ps-radius);color:var(--ps-muted);text-align:center}
.ps-scene-header{margin-bottom:24px}
.ps-scene-body{display:grid;align-items:start;gap:16px}
.ps-scene-split .ps-scene-body{grid-template-columns:1fr 1fr}
.ps-metric{display:grid;padding-left:12px;border-left:3px solid var(--ps-tone-mark,var(--ps-line))}
.ps-metric>span{color:var(--ps-muted);font-size:.875rem}
.ps-metric>strong{font-size:1.75rem;line-height:1.2;font-variant-numeric:tabular-nums}
.ps-log{display:grid;gap:4px;margin:0;padding:14px 16px;border-radius:var(--ps-radius);background:var(--ps-ink);color:var(--ps-bg);font:.875rem/1.5 ui-monospace,Menlo,Consolas,monospace;list-style:none}
.ps-log li{display:flex;gap:12px}
.ps-log span{opacity:.7}
.ps-log strong{font-weight:400}
.ps-graph{display:block;width:100%;height:auto;overflow:visible}
.ps-graph-edge{fill:none;stroke:var(--ps-control);stroke-width:2}
.ps-graph-node{cursor:pointer;outline:none}
.ps-graph-node rect{fill:var(--ps-panel);stroke:var(--ps-control);stroke-width:1.5}
.ps-graph-node circle{fill:var(--ps-control)}
.ps-status-active rect{stroke:var(--ps-accent);stroke-width:2}
.ps-status-active circle{fill:var(--ps-accent)}
.ps-status-ready circle{fill:var(--ps-good)}
.ps-status-syncing circle{fill:var(--ps-warning)}
.ps-graph-node.is-selected rect{fill:color-mix(in srgb,var(--ps-accent) 12%,var(--ps-panel));stroke:var(--ps-accent);stroke-width:2}
.ps-graph-node:focus-visible rect{stroke:var(--ps-ink);stroke-width:3}
.ps-graph-label{fill:var(--ps-ink);font-size:13px;font-weight:600}
.ps-graph-icon{fill:var(--ps-link);font-size:14px;font-weight:700}
.ps-effect-flow{stroke-dasharray:8 8}
.ps-effect-glow{filter:drop-shadow(0 0 6px var(--ps-accent))}
@keyframes ps-flow{to{stroke-dashoffset:-16}}
@keyframes ps-pulse{50%{opacity:.45}}
@media (prefers-reduced-motion:no-preference){html{scroll-behavior:smooth}.ps-effect-flow{animation:ps-flow 1s linear infinite}.ps-effect-pulse,.ps-pulse-svg{animation:ps-pulse 2s ease-in-out infinite}.ps-reveal{opacity:0;transform:translateY(12px);transition:opacity .5s,transform .5s}.ps-reveal-visible{opacity:1;transform:none}}
@media (max-width:720px){.ps-grid,.ps-scene-split .ps-scene-body{grid-template-columns:minmax(0,1fr)}.ps-space-lg,.ps-scene{padding-block:48px}.ps-table th,.ps-table td{padding:8px 10px}}"#
}

fn base_js() -> &'static str {
    r#"(()=>{const cfg=JSON.parse(document.getElementById("ps-runtime-config")?.textContent||"{\"states\":[],\"events\":[]}");const state={};for(const item of cfg.states||[]){state[item.id]=item.default;document.body.dataset["state"+item.id.charAt(0).toUpperCase()+item.id.slice(1)]=String(item.default)}function setState(id,value){state[id]=value;document.body.dataset["state"+id.charAt(0).toUpperCase()+id.slice(1)]=String(value);document.querySelectorAll("[data-ps-node]").forEach(node=>node.classList.toggle("is-selected",node.dataset.psStateValue===String(value)))}document.addEventListener("click",(event)=>{const button=event.target.closest("[data-action]");if(button){const action=button.dataset.action;const target=button.dataset.target;if(action==="open-modal"&&target){document.getElementById(target)?.showModal?.()}if(action==="toggle"&&target){document.getElementById(target)?.toggleAttribute("hidden")}}const graphNode=event.target.closest("[data-ps-node]");if(graphNode){for(const rule of cfg.events||[]){if(rule.on==="node.click"){const value=rule.value==="$node.id"?graphNode.dataset.psStateValue:rule.value;setState(rule.set,value)}}}});if("IntersectionObserver" in window&&!matchMedia("(prefers-reduced-motion: reduce)").matches){const obs=new IntersectionObserver((entries)=>{entries.forEach(entry=>{if(entry.isIntersecting){entry.target.classList.add("ps-reveal-visible");obs.unobserve(entry.target)}})},{rootMargin:"0px 0px -10% 0px"});document.querySelectorAll(".ps-effect-reveal").forEach(el=>{el.classList.add("ps-reveal");obs.observe(el)});}for(const [id,value] of Object.entries(state)){setState(id,value)}})()"#
}
