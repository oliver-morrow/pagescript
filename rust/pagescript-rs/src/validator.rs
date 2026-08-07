use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::parser::parse_page_script;
use crate::resolver::{Resolver, is_valid_import_path};
use crate::types::{
    ComponentNode, Diagnostic, DocumentNode, Node, PageNode, StepNode, TourNode, TriggerNode, error,
};

pub fn validate_document(document: &DocumentNode, resolver: &Resolver) -> Vec<Diagnostic> {
    let mut diagnostics = document.diagnostics.clone();
    let mut page_ids = HashSet::new();
    let mut tour_ids = HashSet::new();

    for child in &document.children {
        match child {
            Node::Page(page) => validate_page(
                page,
                &mut diagnostics,
                &mut page_ids,
                &mut tour_ids,
                resolver,
            ),
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
    resolver: &Resolver,
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
        if let Node::Component(component) = child {
            collect_recipe_names(component, &mut ids.known_recipe_names, resolver);
        }
    }
    validate_recipe_expansion_cycles(page, diagnostics, resolver);

    for child in &page.children {
        match child {
            Node::Tour(tour) => validate_tour(tour, diagnostics, tour_ids),
            Node::Component(component) => {
                validate_component(component, diagnostics, &mut ids, resolver)
            }
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
    known_recipe_names: HashSet<String>,
    seen_recipe_names: HashSet<String>,
}

#[derive(Clone)]
struct RecipeDefinition {
    source_line: usize,
    references: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RecipeVisitState {
    Visiting,
    Visited,
}

fn validate_recipe_expansion_cycles(
    page: &PageNode,
    diagnostics: &mut Vec<Diagnostic>,
    resolver: &Resolver,
) {
    let mut definitions = BTreeMap::new();
    let mut seen_imports = HashSet::new();
    for child in &page.children {
        collect_root_recipe_definitions(child, &mut definitions, resolver, &mut seen_imports);
    }

    let mut states = BTreeMap::new();
    let mut stack = Vec::new();
    let mut cycle_members = BTreeSet::new();
    for name in definitions.keys().cloned().collect::<Vec<_>>() {
        visit_recipe(
            &name,
            &definitions,
            &mut states,
            &mut stack,
            &mut cycle_members,
        );
    }

    for name in cycle_members {
        let definition = &definitions[&name];
        diagnostics.push(error(
            "recursive_recipe",
            format!("Recipe \"{name}\" participates in a recursive expansion."),
            definition.source_line,
        ));
    }
}

fn collect_root_recipe_definitions(
    node: &Node,
    definitions: &mut BTreeMap<String, RecipeDefinition>,
    resolver: &Resolver,
    seen_imports: &mut HashSet<String>,
) {
    let Node::Component(component) = node else {
        return;
    };

    if component.name == "import"
        && let Some(from) = component
            .attributes
            .get("from")
            .and_then(|value| value.as_str())
        && is_valid_import_path(from)
        && seen_imports.insert(from.to_string())
        && let Ok(source) = resolver.resolve(from)
    {
        let imported_document = parse_page_script(&source);
        for child in &imported_document.children {
            collect_imported_recipe_definitions(child, definitions, resolver, seen_imports);
        }
    }

    if component.name == "recipe"
        && let Some(name) = component
            .attributes
            .get("name")
            .and_then(|value| value.as_str())
    {
        definitions.insert(name.to_string(), recipe_definition(component));
    }

    for child in &component.children {
        collect_root_recipe_definitions(child, definitions, resolver, seen_imports);
    }
}

fn collect_imported_recipe_definitions(
    node: &Node,
    definitions: &mut BTreeMap<String, RecipeDefinition>,
    resolver: &Resolver,
    seen_imports: &mut HashSet<String>,
) {
    match node {
        Node::Page(page) => {
            for child in &page.children {
                collect_imported_recipe_definitions(child, definitions, resolver, seen_imports);
            }
        }
        Node::Component(component) => {
            if component.name == "import"
                && let Some(from) = component
                    .attributes
                    .get("from")
                    .and_then(|value| value.as_str())
                && is_valid_import_path(from)
                && seen_imports.insert(from.to_string())
                && let Ok(source) = resolver.resolve(from)
            {
                let imported_document = parse_page_script(&source);
                for child in &imported_document.children {
                    collect_imported_recipe_definitions(child, definitions, resolver, seen_imports);
                }
            }

            if component.name == "recipe"
                && let Some(name) = component
                    .attributes
                    .get("name")
                    .and_then(|value| value.as_str())
            {
                definitions
                    .entry(name.to_string())
                    .or_insert_with(|| recipe_definition(component));
            }

            for child in &component.children {
                collect_imported_recipe_definitions(child, definitions, resolver, seen_imports);
            }
        }
        _ => {}
    }
}

fn recipe_definition(component: &ComponentNode) -> RecipeDefinition {
    RecipeDefinition {
        source_line: component.source.line,
        references: recipe_references(component),
    }
}

fn recipe_references(component: &ComponentNode) -> Vec<String> {
    let mut references = BTreeSet::new();
    collect_recipe_references(component, &mut references);
    references.into_iter().collect()
}

fn collect_recipe_references(component: &ComponentNode, references: &mut BTreeSet<String>) {
    for child in &component.children {
        let Node::Component(child) = child else {
            continue;
        };
        if child.name == "recipe" {
            continue;
        }
        if child.name == "use"
            && let Some(recipe) = child
                .attributes
                .get("recipe")
                .and_then(|value| value.as_str())
        {
            references.insert(recipe.to_string());
        }
        collect_recipe_references(child, references);
    }
}

fn visit_recipe(
    name: &str,
    definitions: &BTreeMap<String, RecipeDefinition>,
    states: &mut BTreeMap<String, RecipeVisitState>,
    stack: &mut Vec<String>,
    cycle_members: &mut BTreeSet<String>,
) {
    match states.get(name) {
        Some(RecipeVisitState::Visited) => return,
        Some(RecipeVisitState::Visiting) => {
            if let Some(index) = stack.iter().position(|item| item == name) {
                cycle_members.extend(stack[index..].iter().cloned());
            }
            return;
        }
        None => {}
    }

    states.insert(name.to_string(), RecipeVisitState::Visiting);
    stack.push(name.to_string());
    let references = definitions[name].references.clone();
    for reference in references {
        if definitions.contains_key(&reference) {
            visit_recipe(&reference, definitions, states, stack, cycle_members);
        }
    }
    stack.pop();
    states.insert(name.to_string(), RecipeVisitState::Visited);
}

fn validate_component(
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
    ids: &mut PageIds,
    resolver: &Resolver,
) {
    let required = match component.name.as_str() {
        "nav" => &[][..],
        "nav-item" => &["label", "href"][..],
        "button" => &["label"][..],
        "image" => &["src", "alt"][..],
        "input" => &["name", "label"][..],
        "filter" => &["id", "label"][..],
        "table" => &[][..],
        "column" => &["label"][..],
        "row" => &[][..],
        "cell" => &[][..],
        "empty-state" => &["title"][..],
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
        "el" => &["tag"][..],
        "attr" => &["name", "value"][..],
        "style-rule" => &["selector"][..],
        "recipe" => &["name"][..],
        "use" => &["recipe"][..],
        "bind" => &["state"][..],
        "on" => &["event", "action"][..],
        "import" => &["from"][..],
        "raw" => &[][..],
        "script" => &[][..],
        _ => &[],
    };

    for key in required {
        if !component.attributes.contains_key(*key) {
            diagnostics.push(error(
                "missing_component_attribute",
                format!(
                    "Component \"{}\" is missing required attribute \"{}\". Add {}=... to the opening directive.",
                    component.name, key, key
                ),
                component.source.line,
            ));
        }
    }

    validate_component_semantics(component, diagnostics, ids, resolver);

    for child in &component.children {
        if let Node::Component(component) = child {
            validate_component(component, diagnostics, ids, resolver);
        }
    }
}

fn validate_component_semantics(
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
    ids: &mut PageIds,
    resolver: &Resolver,
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
            if contains_style_terminator(component) {
                diagnostics.push(error(
                    "unsafe_style_content",
                    "Style content must not contain \"</style\".",
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
        "nav-item" => validate_component_url_attribute(component, "href", diagnostics),
        "image" => validate_component_url_attribute(component, "src", diagnostics),
        "form" => validate_component_url_attribute(component, "action", diagnostics),
        "el" => {
            if let Some(tag) = component
                .attributes
                .get("tag")
                .and_then(|value| value.as_str())
            {
                validate_element_tag(tag, component, diagnostics);
            }
            for (name, value) in &component.attributes {
                validate_url_attribute(name, value.as_str(), component, diagnostics);
            }
        }
        "attr" => {
            if let Some(name) = component
                .attributes
                .get("name")
                .and_then(|value| value.as_str())
            {
                validate_attribute_name(name, component, diagnostics);
                validate_url_attribute(
                    name,
                    component
                        .attributes
                        .get("value")
                        .and_then(|value| value.as_str()),
                    component,
                    diagnostics,
                );
            }
        }
        "style-rule" if contains_style_terminator(component) => diagnostics.push(error(
            "unsafe_style_content",
            "Style content must not contain \"</style\".",
            component.source.line,
        )),
        "recipe" => record_unique_attr(
            &mut ids.seen_recipe_names,
            component,
            diagnostics,
            "name",
            "duplicate_recipe_name",
            "recipe",
        ),
        "use" => {
            if let Some(name) = component
                .attributes
                .get("recipe")
                .and_then(|value| value.as_str())
                && !ids.known_recipe_names.contains(name)
            {
                diagnostics.push(error(
                    "unknown_recipe",
                    format!("Recipe \"{name}\" was not found."),
                    component.source.line,
                ));
            }
        }
        "import" => validate_import(component, diagnostics, resolver),
        "raw" | "script" => diagnostics.push(error(
            "unsafe_escape_hatch",
            format!(
                "Component \"{}\" is outside the deterministic PageScript core.",
                component.name
            ),
            component.source.line,
        )),
        _ => {}
    }
}

fn validate_import(
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
    resolver: &Resolver,
) {
    let Some(from) = component
        .attributes
        .get("from")
        .and_then(|value| value.as_str())
    else {
        return;
    };

    if !is_valid_import_path(from) {
        diagnostics.push(error(
            "invalid_import_path",
            "Import paths must be relative and must not contain \"..\".",
            component.source.line,
        ));
    } else if let Err(message) = resolver.resolve(from) {
        diagnostics.push(error("unresolved_import", message, component.source.line));
    }
}

fn contains_style_terminator(component: &ComponentNode) -> bool {
    component
        .attributes
        .get("props")
        .and_then(|value| value.as_str())
        .into_iter()
        .chain(component.children.iter().filter_map(|child| match child {
            Node::Markdown(markdown) => Some(markdown.value.as_str()),
            _ => None,
        }))
        .any(|content| content.to_ascii_lowercase().contains("</style"))
}

fn validate_component_url_attribute(
    component: &ComponentNode,
    attribute: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_url_attribute(
        attribute,
        component
            .attributes
            .get(attribute)
            .and_then(|value| value.as_str()),
        component,
        diagnostics,
    );
}

fn validate_url_attribute(
    name: &str,
    value: Option<&str>,
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if is_url_attribute(name) && value.is_some_and(|value| !is_safe_url(value)) {
        diagnostics.push(error(
            "unsafe_url_scheme",
            format!("Attribute \"{name}\" has an unsafe URL scheme."),
            component.source.line,
        ));
    }
}

fn is_url_attribute(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "href" | "src" | "action"
    )
}

fn is_safe_url(value: &str) -> bool {
    if value.chars().any(char::is_control) {
        return false;
    }
    let value = value.trim();
    let Some((scheme, _)) = value.split_once(':') else {
        return true;
    };
    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "http" | "https" | "mailto" | "tel"
    )
}

fn collect_recipe_names(
    component: &ComponentNode,
    names: &mut HashSet<String>,
    resolver: &Resolver,
) {
    let mut seen_imports = HashSet::new();
    collect_recipe_names_inner(component, names, resolver, &mut seen_imports);
}

fn collect_recipe_names_inner(
    component: &ComponentNode,
    names: &mut HashSet<String>,
    resolver: &Resolver,
    seen_imports: &mut HashSet<String>,
) {
    if component.name == "recipe"
        && let Some(name) = component
            .attributes
            .get("name")
            .and_then(|value| value.as_str())
    {
        names.insert(name.to_string());
    }

    if component.name == "import"
        && let Some(from) = component
            .attributes
            .get("from")
            .and_then(|value| value.as_str())
        && is_valid_import_path(from)
        && seen_imports.insert(from.to_string())
        && let Ok(source) = resolver.resolve(from)
    {
        let imported_doc = parse_page_script(&source);
        for child in &imported_doc.children {
            collect_imported_recipe_names(child, names, resolver, seen_imports);
        }
    }

    for child in &component.children {
        if let Node::Component(component) = child {
            collect_recipe_names_inner(component, names, resolver, seen_imports);
        }
    }
}

fn collect_imported_recipe_names(
    node: &Node,
    names: &mut HashSet<String>,
    resolver: &Resolver,
    seen_imports: &mut HashSet<String>,
) {
    match node {
        Node::Page(page) => {
            for child in &page.children {
                collect_imported_recipe_names(child, names, resolver, seen_imports);
            }
        }
        Node::Component(component) => {
            if component.name == "import"
                && let Some(from) = component
                    .attributes
                    .get("from")
                    .and_then(|value| value.as_str())
                && is_valid_import_path(from)
                && seen_imports.insert(from.to_string())
                && let Ok(source) = resolver.resolve(from)
            {
                let imported_doc = parse_page_script(&source);
                for child in &imported_doc.children {
                    collect_imported_recipe_names(child, names, resolver, seen_imports);
                }
            }
            if component.name == "recipe"
                && let Some(name) = component
                    .attributes
                    .get("name")
                    .and_then(|value| value.as_str())
            {
                names.insert(name.to_string());
            }
            for child in &component.children {
                collect_imported_recipe_names(child, names, resolver, seen_imports);
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

fn record_unique_attr(
    set: &mut HashSet<String>,
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
    attr: &str,
    code: &str,
    label: &str,
) {
    let Some(id) = component
        .attributes
        .get(attr)
        .and_then(|value| value.as_str())
    else {
        return;
    };
    if !set.insert(id.to_string()) {
        diagnostics.push(error(
            code,
            format!("Duplicate {label} name \"{id}\"."),
            component.source.line,
        ));
    }
}

fn validate_element_tag(tag: &str, component: &ComponentNode, diagnostics: &mut Vec<Diagnostic>) {
    if !is_safe_tag_name(tag) || matches!(tag, "script" | "iframe" | "object" | "embed") {
        diagnostics.push(error(
            "unsafe_element_tag",
            format!("Element tag \"{tag}\" is not allowed."),
            component.source.line,
        ));
    }
}

fn validate_attribute_name(
    name: &str,
    component: &ComponentNode,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !is_safe_attribute_name(name)
        || name.eq_ignore_ascii_case("srcdoc")
        || name.to_ascii_lowercase().starts_with("on")
    {
        diagnostics.push(error(
            "unsafe_attribute_name",
            format!("Attribute \"{name}\" is not allowed."),
            component.source.line,
        ));
    }
}

fn is_safe_tag_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn is_safe_attribute_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':'))
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
