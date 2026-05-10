use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::parser::parse_page_script;
use crate::resolver::Resolver;
use crate::types::{AttributeValue, ComponentNode, DocumentNode, MarkdownNode, Node, PageNode};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageIr {
    pub id: Option<String>,
    pub title: String,
    pub tokens: Map<String, Value>,
    pub recipes: HashMap<String, RecipeIr>,
    pub body: Vec<IrNode>,
    pub effects: HashMap<String, EffectIr>,
    pub states: Vec<StateIr>,
    pub events: Vec<EventIr>,
    pub scoped_css: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IrNode {
    #[serde(rename = "markdown")]
    Markdown(MarkdownIr),
    #[serde(rename = "component")]
    Component(Box<ComponentIr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkdownIr {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentIr {
    pub name: String,
    pub attributes: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutIr>,
    pub children: Vec<IrNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph: Option<GraphIr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutIr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphIr {
    pub nodes: Vec<GraphNodeIr>,
    pub edges: Vec<GraphEdgeIr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNodeIr {
    pub id: String,
    pub label: String,
    pub status: String,
    pub icon: Option<String>,
    pub x: i64,
    pub y: i64,
    pub effect: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdgeIr {
    pub from: String,
    pub to: String,
    pub effect: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectIr {
    pub effect_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateIr {
    pub id: String,
    pub default_value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventIr {
    pub on: String,
    pub set: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeIr {
    pub name: String,
    pub attributes: Map<String, Value>,
}

pub fn compile_page_ir(
    document: &DocumentNode,
    page_id: Option<&str>,
    resolver: &Resolver,
) -> Result<PageIr, String> {
    let page = select_page(document, page_id)?;
    let mut context = IrContext {
        resolver,
        tokens: Map::new(),
        recipes: HashMap::new(),
        effects: HashMap::new(),
        states: Vec::new(),
        events: Vec::new(),
        scoped_css: String::new(),
    };
    for child in &page.children {
        collect_head_data(child, &mut context);
    }

    let body = page
        .children
        .iter()
        .flat_map(|node| normalize_node(node, &context))
        .collect::<Vec<_>>();

    Ok(PageIr {
        id: page.id.clone(),
        title: string_attr_page(page, "title")
            .or(page.id.as_deref())
            .unwrap_or("PageScript Page")
            .to_string(),
        tokens: context.tokens,
        recipes: context
            .recipes
            .iter()
            .map(|(name, recipe)| {
                (
                    name.clone(),
                    RecipeIr {
                        name: name.clone(),
                        attributes: recipe.attributes.clone(),
                    },
                )
            })
            .collect(),
        body,
        effects: context.effects,
        states: context.states,
        events: context.events,
        scoped_css: context.scoped_css,
    })
}

struct IrContext<'a> {
    resolver: &'a Resolver,
    tokens: Map<String, Value>,
    recipes: HashMap<String, RecipeDef>,
    effects: HashMap<String, EffectIr>,
    states: Vec<StateIr>,
    events: Vec<EventIr>,
    scoped_css: String,
}

#[derive(Debug, Clone)]
struct RecipeDef {
    attributes: Map<String, Value>,
    template: Vec<Node>,
}

fn collect_head_data(node: &Node, context: &mut IrContext) {
    let Node::Component(component) = node else {
        return;
    };

    match component.name.as_str() {
        "effect" => {
            if let (Some(id), Some(effect_type)) =
                (string_attr(component, "id"), string_attr(component, "type"))
            {
                context.effects.insert(
                    id.to_string(),
                    EffectIr {
                        effect_type: effect_type.to_string(),
                    },
                );
            }
        }
        "state" => {
            if let (Some(id), Some(default_value)) = (
                string_attr(component, "id"),
                string_attr(component, "default"),
            ) {
                context.states.push(StateIr {
                    id: id.to_string(),
                    default_value: default_value.to_string(),
                });
            }
        }
        "event" => {
            if let (Some(on), Some(set), Some(value)) = (
                string_attr(component, "on"),
                string_attr(component, "set"),
                string_attr(component, "value"),
            ) {
                context.events.push(EventIr {
                    on: on.to_string(),
                    set: set.to_string(),
                    value: value.to_string(),
                });
            }
        }
        "style" => {
            context.scoped_css.push_str(&style_body(component));
            context.scoped_css.push('\n');
        }
        "style-rule" => {
            if let Some(rule) = style_rule_body(component) {
                context.scoped_css.push_str(&rule);
                context.scoped_css.push('\n');
            }
        }
        "tokens" => {
            for (key, value) in &component.attributes {
                context.tokens.insert(key.clone(), value.clone());
            }
        }
        "recipe" => {
            if let Some(name) = string_attr(component, "name") {
                context.recipes.insert(
                    name.to_string(),
                    RecipeDef {
                        attributes: component.attributes.clone(),
                        template: recipe_template(component),
                    },
                );
            }
        }
        "import" => {
            if let Some(from) = string_attr(component, "from")
                && let Ok(source) = context.resolver.resolve(from)
            {
                let imported_doc = parse_page_script(&source);
                for child in &imported_doc.children {
                    collect_imported_recipes(child, context);
                }
            }
        }
        _ => {}
    }

    for child in &component.children {
        collect_head_data(child, context);
    }
}

fn collect_imported_recipes(node: &Node, context: &mut IrContext) {
    match node {
        Node::Page(page) => {
            for child in &page.children {
                collect_imported_recipes(child, context);
            }
        }
        Node::Component(component) => {
            if component.name == "recipe"
                && let Some(name) = string_attr(component, "name")
                && !context.recipes.contains_key(name)
            {
                context.recipes.insert(
                    name.to_string(),
                    RecipeDef {
                        attributes: component.attributes.clone(),
                        template: recipe_template(component),
                    },
                );
            }
            for child in &component.children {
                collect_imported_recipes(child, context);
            }
        }
        _ => {}
    }
}

fn normalize_node<'a>(node: &Node, context: &IrContext<'a>) -> Vec<IrNode> {
    match node {
        Node::Markdown(markdown) => vec![IrNode::Markdown(normalize_markdown(markdown))],
        Node::Component(component) if component.name == "use" => expand_recipe(component, context),
        Node::Component(component) if is_compiler_only_component(component) => Vec::new(),
        Node::Component(component) => vec![IrNode::Component(Box::new(normalize_component(
            component, context,
        )))],
        _ => Vec::new(),
    }
}

fn normalize_markdown(node: &MarkdownNode) -> MarkdownIr {
    MarkdownIr {
        value: node.value.clone(),
    }
}

fn normalize_component<'a>(node: &ComponentNode, context: &IrContext<'a>) -> ComponentIr {
    let graph = if node.name == "panel" {
        normalize_graph(node)
    } else {
        None
    };
    let children = node
        .children
        .iter()
        .flat_map(|child| match child {
            Node::Component(component) if matches!(component.name.as_str(), "node" | "edge") => {
                Vec::new()
            }
            _ => normalize_node(child, context),
        })
        .collect::<Vec<_>>();

    ComponentIr {
        name: node.name.clone(),
        attributes: node.attributes.clone(),
        layout: normalize_layout(node),
        children,
        graph,
    }
}

fn expand_recipe<'a>(component: &ComponentNode, context: &IrContext<'a>) -> Vec<IrNode> {
    let Some(name) = string_attr(component, "recipe") else {
        return Vec::new();
    };
    let Some(recipe) = context.recipes.get(name).cloned() else {
        return Vec::new();
    };
    let mut values = recipe.attributes;
    for (key, value) in &component.attributes {
        if key != "recipe" {
            values.insert(key.clone(), value.clone());
        }
    }

    let mut slots = HashMap::new();
    let mut default_slot_children = Vec::new();

    for child in &component.children {
        match child {
            Node::Component(slot_comp) if slot_comp.name == "slot" => {
                let slot_name = string_attr(slot_comp, "name").unwrap_or("");
                slots.insert(slot_name.to_string(), slot_comp.children.clone());
            }
            _ => {
                default_slot_children.push(child.clone());
            }
        }
    }

    if !default_slot_children.is_empty() {
        slots.entry("".to_string()).or_insert(default_slot_children);
    }

    recipe
        .template
        .iter()
        .flat_map(|node| substitute_node(node, &values, &slots))
        .flat_map(|node| normalize_node(&node, context))
        .collect()
}

fn recipe_template(component: &ComponentNode) -> Vec<Node> {
    component
        .children
        .iter()
        .find_map(|child| match child {
            Node::Component(template) if template.name == "template" => {
                Some(template.children.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| component.children.clone())
}

fn substitute_node(
    node: &Node,
    values: &Map<String, Value>,
    slots: &HashMap<String, Vec<Node>>,
) -> Vec<Node> {
    match node {
        Node::Markdown(markdown) => vec![Node::Markdown(MarkdownNode {
            source: markdown.source.clone(),
            value: substitute_string(&markdown.value, values),
        })],
        Node::Component(component) if component.name == "slot" => {
            let slot_name = string_attr(component, "name").unwrap_or("");
            if let Some(content) = slots.get(slot_name) {
                content.clone()
            } else {
                component
                    .children
                    .iter()
                    .flat_map(|child| substitute_node(child, values, slots))
                    .collect()
            }
        }
        Node::Component(component) => vec![Node::Component(ComponentNode {
            name: component.name.clone(),
            source: component.source.clone(),
            attributes: component
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), substitute_value(value, values)))
                .collect(),
            children: component
                .children
                .iter()
                .flat_map(|child| substitute_node(child, values, slots))
                .collect(),
        })],
        other => vec![other.clone()],
    }
}

fn substitute_value(value: &Value, values: &Map<String, Value>) -> Value {
    match value {
        Value::String(text) => Value::String(substitute_string(text, values)),
        _ => value.clone(),
    }
}

fn substitute_string(input: &str, values: &Map<String, Value>) -> String {
    let mut output = input.to_string();
    for (key, value) in values {
        output = output.replace(&format!("${key}"), &stringify_value(value));
    }
    output
}

fn stringify_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => value.to_string(),
    }
}

fn normalize_layout(node: &ComponentNode) -> Option<LayoutIr> {
    let layout = LayoutIr {
        mode: string_attr(node, "layout").map(ToOwned::to_owned),
        density: string_attr(node, "density").map(ToOwned::to_owned),
        gap: string_attr(node, "gap").map(ToOwned::to_owned),
        columns: number_attr_optional(node, "columns"),
        align: string_attr(node, "align").map(ToOwned::to_owned),
    };
    if layout.mode.is_none()
        && layout.density.is_none()
        && layout.gap.is_none()
        && layout.columns.is_none()
        && layout.align.is_none()
    {
        None
    } else {
        Some(layout)
    }
}

fn normalize_graph(panel: &ComponentNode) -> Option<GraphIr> {
    let nodes = panel
        .children
        .iter()
        .filter_map(|child| match child {
            Node::Component(component) if component.name == "node" => Some(GraphNodeIr {
                id: string_attr(component, "id").unwrap_or("").to_string(),
                label: string_attr(component, "label").unwrap_or("").to_string(),
                status: string_attr(component, "status")
                    .unwrap_or("default")
                    .to_string(),
                icon: string_attr(component, "icon").map(ToOwned::to_owned),
                x: number_attr(component, "x", 80),
                y: number_attr(component, "y", 80),
                effect: string_attr(component, "effect").map(ToOwned::to_owned),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    let edges = panel
        .children
        .iter()
        .filter_map(|child| match child {
            Node::Component(component) if component.name == "edge" => Some(GraphEdgeIr {
                from: string_attr(component, "from").unwrap_or("").to_string(),
                to: string_attr(component, "to").unwrap_or("").to_string(),
                effect: string_attr(component, "effect").map(ToOwned::to_owned),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();

    if nodes.is_empty() && edges.is_empty() {
        None
    } else {
        Some(GraphIr { nodes, edges })
    }
}

fn is_compiler_only_component(component: &ComponentNode) -> bool {
    matches!(
        component.name.as_str(),
        "state"
            | "event"
            | "effect"
            | "style"
            | "style-rule"
            | "tokens"
            | "recipe"
            | "template"
            | "node"
            | "edge"
            | "import"
            | "slot"
    )
}

fn select_page<'a>(
    document: &'a DocumentNode,
    page_id: Option<&str>,
) -> Result<&'a PageNode, String> {
    let pages = document.children.iter().filter_map(|node| match node {
        Node::Page(page) => Some(page),
        _ => None,
    });
    if let Some(page_id) = page_id {
        pages
            .into_iter()
            .find(|page| page.id.as_deref() == Some(page_id))
            .ok_or_else(|| format!("Page \"{page_id}\" was not found."))
    } else {
        pages
            .into_iter()
            .next()
            .ok_or_else(|| "No page was found.".to_string())
    }
}

fn style_body(component: &ComponentNode) -> String {
    component
        .children
        .iter()
        .filter_map(|child| match child {
            Node::Markdown(markdown) => Some(markdown.value.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn style_rule_body(component: &ComponentNode) -> Option<String> {
    let selector = string_attr(component, "selector")?;
    let body = string_attr(component, "props")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| style_body(component));
    Some(format!("{selector}{{{body}}}"))
}

fn string_attr<'a>(node: &'a ComponentNode, key: &str) -> Option<&'a str> {
    node.attributes.get(key).and_then(AttributeValue::as_str)
}

fn string_attr_page<'a>(node: &'a PageNode, key: &str) -> Option<&'a str> {
    node.attributes.get(key).and_then(AttributeValue::as_str)
}

fn number_attr(node: &ComponentNode, key: &str, fallback: i64) -> i64 {
    node.attributes
        .get(key)
        .and_then(AttributeValue::as_i64)
        .unwrap_or(fallback)
}

fn number_attr_optional(node: &ComponentNode, key: &str) -> Option<i64> {
    node.attributes.get(key).and_then(AttributeValue::as_i64)
}
