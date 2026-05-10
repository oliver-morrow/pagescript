use std::collections::HashSet;

use crate::types::{
    ComponentNode, Diagnostic, DocumentNode, Node, PageNode, StepNode, TourNode, TriggerNode, error,
};

pub fn validate_document(document: &DocumentNode) -> Vec<Diagnostic> {
    let mut diagnostics = document.diagnostics.clone();
    let mut page_ids = HashSet::new();
    let mut tour_ids = HashSet::new();

    for child in &document.children {
        match child {
            Node::Page(page) => validate_page(page, &mut diagnostics, &mut page_ids, &mut tour_ids),
            Node::Tour(tour) => validate_tour(tour, &mut diagnostics, &mut tour_ids),
            _ => {}
        }
    }

    diagnostics
}

fn validate_page(
    page: &PageNode,
    diagnostics: &mut Vec<Diagnostic>,
    page_ids: &mut HashSet<String>,
    tour_ids: &mut HashSet<String>,
) {
    match &page.id {
        Some(id) if page_ids.contains(id) => diagnostics.push(error(
            "duplicate_page_id",
            format!("Duplicate page id \"{id}\"."),
            page.source.line,
        )),
        Some(id) => {
            page_ids.insert(id.clone());
        }
        None => diagnostics.push(error(
            "missing_page_id",
            "Page is missing required id.",
            page.source.line,
        )),
    }

    let mut ids = PageIds::default();
    for child in &page.children {
        match child {
            Node::Tour(tour) => validate_tour(tour, diagnostics, tour_ids),
            Node::Component(component) => validate_component(component, diagnostics, &mut ids),
            _ => {}
        }
    }
}

#[derive(Default)]
struct PageIds {
    scene_ids: HashSet<String>,
    node_ids: HashSet<String>,
    state_ids: HashSet<String>,
    effect_ids: HashSet<String>,
}

fn validate_component(
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
    ids: &mut PageIds,
) {
    let required = match component.name.as_str() {
        "button" => &["label"][..],
        "image" => &["src", "alt"][..],
        "input" => &["name", "label"][..],
        "scene" => &["id"][..],
        "panel" => &["id"][..],
        "node" => &["id", "label"][..],
        "edge" => &["from", "to"][..],
        "metric" => &["id", "label", "value"][..],
        "state" => &["id", "default"][..],
        "event" => &["on", "set", "value"][..],
        "effect" => &["id", "type"][..],
        "style" => &["scope"][..],
        "tokens" => &[][..],
        _ => &[],
    };

    for key in required {
        if !component.attributes.contains_key(*key) {
            diagnostics.push(error(
                "missing_component_attribute",
                format!(
                    "Component \"{}\" is missing required attribute \"{}\".",
                    component.name, key
                ),
                component.source.line,
            ));
        }
    }

    validate_component_semantics(component, diagnostics, ids);

    for child in &component.children {
        if let Node::Component(component) = child {
            validate_component(component, diagnostics, ids);
        }
    }
}

fn validate_component_semantics(
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
    ids: &mut PageIds,
) {
    match component.name.as_str() {
        "scene" => record_unique_id(
            &mut ids.scene_ids,
            component,
            diagnostics,
            "duplicate_scene_id",
            "scene",
        ),
        "node" => record_unique_id(
            &mut ids.node_ids,
            component,
            diagnostics,
            "duplicate_node_id",
            "node",
        ),
        "state" => record_unique_id(
            &mut ids.state_ids,
            component,
            diagnostics,
            "duplicate_state_id",
            "state",
        ),
        "effect" => {
            record_unique_id(
                &mut ids.effect_ids,
                component,
                diagnostics,
                "duplicate_effect_id",
                "effect",
            );
            let effect_type = component
                .attributes
                .get("type")
                .and_then(|value| value.as_str());
            if let Some(effect_type) = effect_type
                && !matches!(
                    effect_type,
                    "flow" | "pulse" | "glow" | "count-up" | "reveal"
                )
            {
                diagnostics.push(error(
                    "invalid_effect_type",
                    format!("Effect type \"{effect_type}\" is not supported."),
                    component.source.line,
                ));
            }
        }
        "style" => {
            let scope = component
                .attributes
                .get("scope")
                .and_then(|value| value.as_str());
            if let Some(scope) = scope
                && !matches!(scope, "page" | "component")
            {
                diagnostics.push(error(
                    "invalid_style_scope",
                    format!("Style scope \"{scope}\" is not supported."),
                    component.source.line,
                ));
            }
        }
        "tokens" => {
            for (key, value) in &component.attributes {
                if !value.is_string() && !value.is_number() && !value.is_boolean() {
                    diagnostics.push(error(
                        "invalid_token_value",
                        format!("Token \"{key}\" must be a string, number, or boolean."),
                        component.source.line,
                    ));
                }
            }
        }
        _ => {}
    }
}

fn record_unique_id(
    set: &mut HashSet<String>,
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
    code: &str,
    label: &str,
) {
    let Some(id) = component
        .attributes
        .get("id")
        .and_then(|value| value.as_str())
    else {
        return;
    };
    if !set.insert(id.to_string()) {
        diagnostics.push(error(
            code,
            format!("Duplicate {label} id \"{id}\"."),
            component.source.line,
        ));
    }
}

fn validate_tour(
    tour: &TourNode,
    diagnostics: &mut Vec<Diagnostic>,
    tour_ids: &mut HashSet<String>,
) {
    match &tour.id {
        Some(id) if tour_ids.contains(id) => diagnostics.push(error(
            "duplicate_tour_id",
            format!("Duplicate tour id \"{id}\"."),
            tour.source.line,
        )),
        Some(id) => {
            tour_ids.insert(id.clone());
        }
        None => diagnostics.push(error(
            "missing_tour_id",
            "Tour is missing required id.",
            tour.source.line,
        )),
    }

    let mut step_ids = HashSet::new();
    for child in &tour.children {
        match child {
            Node::Step(step) => validate_step(step, diagnostics, &mut step_ids),
            Node::Trigger(trigger) => validate_trigger(trigger, diagnostics),
            _ => {}
        }
    }
}

fn validate_step(
    step: &StepNode,
    diagnostics: &mut Vec<Diagnostic>,
    step_ids: &mut HashSet<String>,
) {
    match &step.id {
        Some(id) if step_ids.contains(id) => diagnostics.push(error(
            "duplicate_step_id",
            format!("Duplicate step id \"{id}\"."),
            step.source.line,
        )),
        Some(id) => {
            step_ids.insert(id.clone());
        }
        None => diagnostics.push(error(
            "missing_step_id",
            "Step is missing required id.",
            step.source.line,
        )),
    }

    if step.target.is_none() {
        diagnostics.push(error(
            "missing_step_target",
            "Step is missing required target.",
            step.source.line,
        ));
    }
}

fn validate_trigger(trigger: &TriggerNode, diagnostics: &mut Vec<Diagnostic>) {
    if trigger.trigger_type.is_none() {
        diagnostics.push(error(
            "missing_trigger_type",
            "Trigger is missing required type.",
            trigger.source.line,
        ));
    }
}
