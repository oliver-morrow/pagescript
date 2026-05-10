use serde_json::{Map, Number, Value};

use crate::types::{
    Attributes, ComponentNode, Diagnostic, DocumentNode, MarkdownNode, Node, PageNode, StepNode,
    TourNode, TriggerNode, error, source,
};

const COMPONENT_DIRECTIVES: &[&str] = &[
    "section",
    "hero",
    "stack",
    "grid",
    "card",
    "button",
    "text",
    "image",
    "modal",
    "form",
    "input",
    "scene",
    "panel",
    "node",
    "edge",
    "metric",
    "log",
    "state",
    "event",
    "effect",
    "style",
    "tokens",
    "el",
    "attr",
    "style-rule",
    "slot",
    "recipe",
    "template",
    "use",
    "bind",
    "on",
    "import",
];

enum Context {
    Document {
        node: DocumentNode,
        markdown: Vec<String>,
    },
    Page {
        node: PageNode,
        markdown: Vec<String>,
    },
    Component {
        node: ComponentNode,
        markdown: Vec<String>,
    },
    Tour {
        node: TourNode,
        markdown: Vec<String>,
    },
    Step {
        node: StepNode,
        markdown: Vec<String>,
    },
    Trigger {
        node: TriggerNode,
        markdown: Vec<String>,
    },
}

impl Context {
    fn kind(&self) -> &'static str {
        match self {
            Context::Document { .. } => "document",
            Context::Page { .. } => "page",
            Context::Component { .. } => "component",
            Context::Tour { .. } => "tour",
            Context::Step { .. } => "step",
            Context::Trigger { .. } => "trigger",
        }
    }

    fn source_line(&self) -> usize {
        match self {
            Context::Document { node, .. } => node.source.line,
            Context::Page { node, .. } => node.source.line,
            Context::Component { node, .. } => node.source.line,
            Context::Tour { node, .. } => node.source.line,
            Context::Step { node, .. } => node.source.line,
            Context::Trigger { node, .. } => node.source.line,
        }
    }
}

struct Directive {
    name: String,
    attributes: Attributes,
}

pub fn parse_tour_script(input: &str) -> DocumentNode {
    parse_page_script(input)
}

pub fn parse_page_script(input: &str) -> DocumentNode {
    let mut diagnostics = Vec::new();
    let document = DocumentNode {
        node_type: "document".to_string(),
        source: source(1),
        children: Vec::new(),
        diagnostics: Vec::new(),
    };
    let mut stack = vec![Context::Document {
        node: document,
        markdown: Vec::new(),
    }];
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");

    for (index, line) in normalized.split('\n').enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("::") {
            flush_markdown(stack.last_mut().expect("document context exists"));
            if trimmed.starts_with("::/") {
                close_directive(trimmed, line_number, &mut stack, &mut diagnostics);
            } else {
                open_directive(trimmed, line_number, &mut stack, &mut diagnostics);
            }
            continue;
        }

        let current = stack.last_mut().expect("document context exists");
        if !matches!(current, Context::Document { .. })
            && let Some((key, value)) = parse_field_line(trimmed, line_number, &mut diagnostics)
        {
            set_field(current, key, value);
            continue;
        }
        push_markdown(current, line);
    }

    while stack.len() > 1 {
        let mut current = stack.pop().expect("non-document context exists");
        flush_markdown(&mut current);
        diagnostics.push(error(
            "unclosed_block",
            format!("Unclosed {} block.", current.kind()),
            current.source_line(),
        ));
        append_to_parent(
            stack.last_mut().expect("parent context exists"),
            context_to_node(current),
        );
    }

    let mut root = stack.pop().expect("document context exists");
    flush_markdown(&mut root);
    let mut document = match root {
        Context::Document { node, .. } => node,
        _ => unreachable!("root context is document"),
    };
    document.diagnostics = diagnostics;
    document
}

fn open_directive(
    trimmed: &str,
    line: usize,
    stack: &mut Vec<Context>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(parsed) = parse_directive(trimmed, line, diagnostics) else {
        return;
    };
    let parent_kind = stack.last().expect("context exists").kind();

    match parsed.name.as_str() {
        "page" => {
            if parent_kind != "document" {
                diagnostics.push(error(
                    "invalid_parent",
                    "A page block must be top-level.",
                    line,
                ));
                return;
            }
            let mut node = PageNode {
                source: source(line),
                attributes: parsed.attributes,
                children: Vec::new(),
                id: None,
                title: None,
                description: None,
                audience: None,
                goal: None,
                context: None,
                url: None,
                options: None,
            };
            apply_page_attributes(&mut node);
            stack.push(Context::Page {
                node,
                markdown: Vec::new(),
            });
        }
        "tour" => {
            if parent_kind != "document" && parent_kind != "page" {
                diagnostics.push(error(
                    "invalid_parent",
                    "A tour block must be top-level or inside a page block.",
                    line,
                ));
                return;
            }
            let mut node = TourNode {
                source: source(line),
                attributes: parsed.attributes,
                children: Vec::new(),
                id: None,
                library: None,
                group: None,
                variant: None,
                title: None,
                description: None,
                options: None,
            };
            apply_tour_attributes(&mut node);
            stack.push(Context::Tour {
                node,
                markdown: Vec::new(),
            });
        }
        name if COMPONENT_DIRECTIVES.contains(&name) => {
            if parent_kind != "page"
                && parent_kind != "component"
                && !matches!(name, "recipe" | "import")
            {
                diagnostics.push(error(
                    "invalid_parent",
                    format!("A {name} block must be inside a page or component block."),
                    line,
                ));
                return;
            }
            stack.push(Context::Component {
                node: ComponentNode {
                    name: name.to_string(),
                    source: source(line),
                    attributes: parsed.attributes,
                    children: Vec::new(),
                },
                markdown: Vec::new(),
            });
        }
        "step" => {
            if parent_kind != "tour" {
                diagnostics.push(error(
                    "invalid_parent",
                    "A step block must be inside a tour block.",
                    line,
                ));
                return;
            }
            let mut node = StepNode {
                source: source(line),
                attributes: parsed.attributes,
                id: None,
                target: None,
                position: None,
                order: None,
                when: None,
                title: None,
                body: None,
                markdown: None,
                options: None,
            };
            apply_step_attributes(&mut node);
            stack.push(Context::Step {
                node,
                markdown: Vec::new(),
            });
        }
        "trigger" => {
            if parent_kind != "tour" {
                diagnostics.push(error(
                    "invalid_parent",
                    "A trigger block must be inside a tour block.",
                    line,
                ));
                return;
            }
            let mut node = TriggerNode {
                source: source(line),
                attributes: parsed.attributes,
                trigger_type: None,
                pattern: None,
                event: None,
                auto_start: None,
                options: None,
            };
            apply_trigger_attributes(&mut node);
            stack.push(Context::Trigger {
                node,
                markdown: Vec::new(),
            });
        }
        unknown => diagnostics.push(error(
            "unknown_directive",
            format!("Unknown directive \"{unknown}\"."),
            line,
        )),
    }
}

fn close_directive(
    trimmed: &str,
    line: usize,
    stack: &mut Vec<Context>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(closing_name) = parse_closing_directive(trimmed) else {
        diagnostics.push(error(
            "malformed_closing_directive",
            "Malformed closing directive.",
            line,
        ));
        return;
    };

    let current_kind = stack.last().expect("context exists").kind();
    if current_kind == "document" {
        diagnostics.push(error(
            "unexpected_closing_directive",
            format!("Unexpected closing directive \"{closing_name}\"."),
            line,
        ));
        return;
    }

    let expected_name = match stack.last().expect("context exists") {
        Context::Component { node, .. } => node.name.as_str(),
        current => current.kind(),
    };
    if expected_name != closing_name {
        diagnostics.push(error(
            "mismatched_closing_directive",
            format!(
                "Expected closing directive for \"{expected_name}\" but found \"{closing_name}\"."
            ),
            line,
        ));
        return;
    }

    let mut current = stack.pop().expect("non-document context exists");
    flush_markdown(&mut current);
    let node = context_to_node(current);
    append_to_parent(stack.last_mut().expect("parent context exists"), node);
}

fn parse_directive(
    trimmed: &str,
    line: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Directive> {
    let rest = trimmed.strip_prefix("::")?;
    let Some((name, end)) = parse_key(rest, 0) else {
        diagnostics.push(error("malformed_directive", "Malformed directive.", line));
        return None;
    };
    if !rest[end..].is_empty() && !rest[end..].starts_with(char::is_whitespace) {
        diagnostics.push(error("malformed_directive", "Malformed directive.", line));
        return None;
    }
    let raw_attributes = rest[end..].trim();
    let attributes = parse_attributes(raw_attributes, line, diagnostics);
    Some(Directive { name, attributes })
}

fn parse_closing_directive(trimmed: &str) -> Option<&str> {
    let name = trimmed.strip_prefix("::/")?.trim();
    if name.is_empty() || name.contains(char::is_whitespace) {
        return None;
    }
    let mut chars = name.chars();
    if !chars.next().is_some_and(is_key_start) {
        return None;
    }
    if chars.all(is_key_continue) {
        Some(name)
    } else {
        None
    }
}

fn parse_field_line(
    trimmed: &str,
    line: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(String, Value)> {
    let equals = trimmed.find('=')?;
    let key = &trimmed[..equals];
    if !is_valid_key(key) {
        return None;
    }

    let value_source = trimmed[equals + 1..].trim();
    if value_source.is_empty() {
        diagnostics.push(error(
            "missing_field_value",
            format!("Missing value for field \"{key}\"."),
            line,
        ));
        return None;
    }

    let (value, next_index) = parse_value(value_source, line, diagnostics)?;
    if next_index != value_source.len() {
        diagnostics.push(error(
            "unexpected_field_text",
            format!("Unexpected text after field \"{key}\"."),
            line,
        ));
        return None;
    }
    Some((key.to_string(), value))
}

fn parse_attributes(source: &str, line: usize, diagnostics: &mut Vec<Diagnostic>) -> Attributes {
    let mut attributes = Map::new();
    let mut index = 0;

    while index < source.len() {
        index = skip_whitespace(source, index);
        if index >= source.len() {
            break;
        }

        let key_start = index;
        let Some((key, next_index)) = parse_key(source, index) else {
            diagnostics.push(error(
                "malformed_attribute",
                "Expected an attribute key.",
                line,
            ));
            break;
        };
        index = skip_whitespace(source, next_index);
        if !source[index..].starts_with('=') {
            diagnostics.push(crate::types::Diagnostic {
                severity: crate::types::Severity::Error,
                code: "malformed_attribute".to_string(),
                message: format!("Expected \"=\" after attribute \"{key}\"."),
                line,
                column: key_start + key.len() + 1,
            });
            break;
        }
        index += 1;
        index = skip_whitespace(source, index);

        let Some((value, next_index)) = parse_value(&source[index..], line, diagnostics) else {
            break;
        };
        attributes.insert(key, value);
        index += next_index;
    }

    attributes
}

fn parse_value(
    source: &str,
    line: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(Value, usize)> {
    if source.starts_with('"') {
        return parse_quoted_string(source, line, diagnostics)
            .map(|(value, next)| (Value::String(value), next));
    }
    if source.starts_with('{') {
        return parse_json_object(source, line, diagnostics);
    }

    let raw_end = source
        .char_indices()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
        .unwrap_or(source.len());
    if raw_end == 0 {
        diagnostics.push(error(
            "missing_attribute_value",
            "Expected an attribute value.",
            line,
        ));
        return None;
    }
    let raw = &source[..raw_end];
    let value = match raw {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ if is_number_literal(raw) && raw.contains('.') => raw
            .parse::<f64>()
            .ok()
            .and_then(Number::from_f64)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(raw.to_string())),
        _ if is_number_literal(raw) => raw
            .parse::<i64>()
            .map(|number| Value::Number(Number::from(number)))
            .unwrap_or_else(|_| Value::String(raw.to_string())),
        _ => Value::String(raw.to_string()),
    };
    Some((value, raw_end))
}

fn parse_quoted_string(
    source: &str,
    line: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;

    for (index, ch) in source.char_indices().skip(1) {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            return Some((value, index + ch.len_utf8()));
        }
        value.push(ch);
    }

    diagnostics.push(error(
        "unterminated_string",
        "Unterminated quoted string.",
        line,
    ));
    None
}

fn parse_json_object(
    source: &str,
    line: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<(Value, usize)> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in source.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let end = index + ch.len_utf8();
                    match serde_json::from_str::<Value>(&source[..end]) {
                        Ok(value) if value.is_object() => return Some((value, end)),
                        Ok(_) => diagnostics.push(error(
                            "invalid_json_object",
                            "JSON attribute values must be objects.",
                            line,
                        )),
                        Err(_) => diagnostics.push(error(
                            "invalid_json_object",
                            "Invalid JSON object attribute value.",
                            line,
                        )),
                    }
                    return None;
                }
            }
            _ => {}
        }
    }

    diagnostics.push(error(
        "unterminated_json_object",
        "Unterminated JSON object attribute value.",
        line,
    ));
    None
}

fn set_field(context: &mut Context, key: String, value: Value) {
    match context {
        Context::Page { node, .. } => {
            node.attributes.insert(key, value);
            apply_page_attributes(node);
        }
        Context::Component { node, .. } => {
            node.attributes.insert(key, value);
        }
        Context::Tour { node, .. } => {
            node.attributes.insert(key, value);
            apply_tour_attributes(node);
        }
        Context::Step { node, .. } => {
            node.attributes.insert(key, value);
            apply_step_attributes(node);
        }
        Context::Trigger { node, .. } => {
            node.attributes.insert(key, value);
            apply_trigger_attributes(node);
        }
        Context::Document { .. } => {}
    }
}

fn apply_page_attributes(node: &mut PageNode) {
    node.id = string_attr(&node.attributes, "id");
    node.title = string_attr(&node.attributes, "title");
    node.description = string_attr(&node.attributes, "description");
    node.audience = string_attr(&node.attributes, "audience");
    node.goal = string_attr(&node.attributes, "goal");
    node.context = string_attr(&node.attributes, "context");
    node.url = string_attr(&node.attributes, "url");
    node.options = object_attr(&node.attributes, "options");
}

fn apply_tour_attributes(node: &mut TourNode) {
    node.id = string_attr(&node.attributes, "id");
    node.library = string_attr(&node.attributes, "library");
    node.group = string_attr(&node.attributes, "group");
    node.variant = string_attr(&node.attributes, "variant");
    node.title = string_attr(&node.attributes, "title");
    node.description = string_attr(&node.attributes, "description");
    node.options = object_attr(&node.attributes, "options");
}

fn apply_step_attributes(node: &mut StepNode) {
    node.id = string_attr(&node.attributes, "id");
    node.target = string_attr(&node.attributes, "target");
    node.position = string_attr(&node.attributes, "position");
    node.order = node
        .attributes
        .get("order")
        .filter(|value| value.is_number())
        .cloned();
    node.when = string_attr(&node.attributes, "when");
    node.title = string_attr(&node.attributes, "title");
    node.body = string_attr(&node.attributes, "body");
    node.options = object_attr(&node.attributes, "options");
}

fn apply_trigger_attributes(node: &mut TriggerNode) {
    node.trigger_type = string_attr(&node.attributes, "type");
    node.pattern = string_attr(&node.attributes, "pattern");
    node.event = string_attr(&node.attributes, "event");
    node.auto_start = node.attributes.get("autoStart").and_then(Value::as_bool);
    node.options = object_attr(&node.attributes, "options");
}

fn flush_markdown(context: &mut Context) {
    let raw = match context {
        Context::Document { markdown, .. }
        | Context::Page { markdown, .. }
        | Context::Component { markdown, .. }
        | Context::Tour { markdown, .. }
        | Context::Step { markdown, .. }
        | Context::Trigger { markdown, .. } => std::mem::take(markdown).join("\n"),
    };
    if raw.trim().is_empty() {
        return;
    }

    match context {
        Context::Document { node, .. } => node.children.push(Node::Markdown(MarkdownNode {
            source: node.source.clone(),
            value: trim_trailing_newlines(&raw),
        })),
        Context::Page { node, .. } => node.children.push(Node::Markdown(MarkdownNode {
            source: node.source.clone(),
            value: trim_trailing_newlines(&raw),
        })),
        Context::Component { node, .. } => node.children.push(Node::Markdown(MarkdownNode {
            source: node.source.clone(),
            value: trim_trailing_newlines(&raw),
        })),
        Context::Tour { node, .. } => node.children.push(Node::Markdown(MarkdownNode {
            source: node.source.clone(),
            value: trim_trailing_newlines(&raw),
        })),
        Context::Step { node, .. } => {
            let value = raw.trim().to_string();
            node.markdown = Some(match node.markdown.take() {
                Some(existing) => format!("{existing}\n{value}"),
                None => value,
            });
        }
        Context::Trigger { .. } => {}
    }
}

fn append_to_parent(parent: &mut Context, node: Node) {
    match parent {
        Context::Document { node: parent, .. } => parent.children.push(node),
        Context::Page { node: parent, .. } => parent.children.push(node),
        Context::Component { node: parent, .. } => parent.children.push(node),
        Context::Tour { node: parent, .. } => parent.children.push(node),
        Context::Step { .. } | Context::Trigger { .. } => {}
    }
}

fn context_to_node(context: Context) -> Node {
    match context {
        Context::Page { node, .. } => Node::Page(node),
        Context::Component { node, .. } => Node::Component(node),
        Context::Tour { node, .. } => Node::Tour(node),
        Context::Step { node, .. } => Node::Step(node),
        Context::Trigger { node, .. } => Node::Trigger(node),
        Context::Document { .. } => unreachable!("document is never converted to child node"),
    }
}

fn push_markdown(context: &mut Context, line: &str) {
    match context {
        Context::Document { markdown, .. }
        | Context::Page { markdown, .. }
        | Context::Component { markdown, .. }
        | Context::Tour { markdown, .. }
        | Context::Step { markdown, .. }
        | Context::Trigger { markdown, .. } => markdown.push(line.to_string()),
    }
}

fn parse_key(source: &str, start: usize) -> Option<(String, usize)> {
    let mut chars = source[start..].char_indices();
    let (_, first) = chars.next()?;
    if !is_key_start(first) {
        return None;
    }
    let mut end = start + first.len_utf8();
    for (offset, ch) in chars {
        if is_key_continue(ch) {
            end = start + offset + ch.len_utf8();
        } else {
            break;
        }
    }
    Some((source[start..end].to_string(), end))
}

fn is_valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    chars.next().is_some_and(is_key_start) && chars.all(is_key_continue)
}

fn is_number_literal(raw: &str) -> bool {
    let mut chars = raw.chars().peekable();
    if chars.peek() == Some(&'-') {
        chars.next();
    }

    let mut whole_digits = 0usize;
    while chars.peek().is_some_and(|ch| ch.is_ascii_digit()) {
        whole_digits += 1;
        chars.next();
    }
    if whole_digits == 0 {
        return false;
    }

    if chars.peek() == Some(&'.') {
        chars.next();
        let mut fractional_digits = 0usize;
        while chars.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            fractional_digits += 1;
            chars.next();
        }
        if fractional_digits == 0 {
            return false;
        }
    }

    chars.next().is_none()
}

fn is_key_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_key_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.'
}

fn skip_whitespace(source: &str, mut index: usize) -> usize {
    while index < source.len() {
        let Some(ch) = source[index..].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

fn string_attr(attributes: &Attributes, key: &str) -> Option<String> {
    attributes
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn object_attr(attributes: &Attributes, key: &str) -> Option<Map<String, Value>> {
    attributes.get(key).and_then(Value::as_object).cloned()
}

fn trim_trailing_newlines(value: &str) -> String {
    value.trim_end_matches('\n').to_string()
}
