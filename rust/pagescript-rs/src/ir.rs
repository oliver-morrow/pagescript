use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::types::{AttributeValue, ComponentNode, DocumentNode, MarkdownNode, Node, PageNode};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageIr {
    pub id: Option<String>,
    pub title: String,
    pub tokens: Map<String, Value>,
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

pub fn compile_page_ir(document: &DocumentNode, page_id: Option<&str>) -> Result<PageIr, String> {
    let page = select_page(document, page_id)?;
    let mut context = IrContext::default();
    for child in &page.children {
        collect_head_data(child, &mut context);
    }

    let body = page
        .children
        .iter()
        .filter_map(normalize_node)
        .collect::<Vec<_>>();

    Ok(PageIr {
        id: page.id.clone(),
        title: string_attr_page(page, "title")
            .or(page.id.as_deref())
            .unwrap_or("PageScript Page")
            .to_string(),
        tokens: context.tokens,
        body,
        effects: context.effects,
        states: context.states,
        events: context.events,
        scoped_css: context.scoped_css,
    })
}

#[derive(Default)]
struct IrContext {
    tokens: Map<String, Value>,
    effects: HashMap<String, EffectIr>,
    states: Vec<StateIr>,
    events: Vec<EventIr>,
    scoped_css: String,
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
        "tokens" => {
            for (key, value) in &component.attributes {
                context.tokens.insert(key.clone(), value.clone());
            }
        }
        _ => {}
    }

    for child in &component.children {
        collect_head_data(child, context);
    }
}

fn normalize_node(node: &Node) -> Option<IrNode> {
    match node {
        Node::Markdown(markdown) => Some(IrNode::Markdown(normalize_markdown(markdown))),
        Node::Component(component) if is_compiler_only_component(component) => None,
        Node::Component(component) => {
            Some(IrNode::Component(Box::new(normalize_component(component))))
        }
        _ => None,
    }
}

fn normalize_markdown(node: &MarkdownNode) -> MarkdownIr {
    MarkdownIr {
        value: node.value.clone(),
    }
}

fn normalize_component(node: &ComponentNode) -> ComponentIr {
    let graph = if node.name == "panel" {
        normalize_graph(node)
    } else {
        None
    };
    let children = node
        .children
        .iter()
        .filter_map(|child| match child {
            Node::Component(component) if matches!(component.name.as_str(), "node" | "edge") => {
                None
            }
            _ => normalize_node(child),
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
        "state" | "event" | "effect" | "style" | "tokens" | "node" | "edge"
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
