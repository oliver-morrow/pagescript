use serde_json::{Map, Value};

use crate::types::{
    AttachTo, DocumentNode, IntroStep, IntroTourConfig, Node, ShepherdStep, ShepherdTourConfig,
    StepNode, TourNode, TriggerNode,
};

pub fn to_shepherd_config(
    document: &DocumentNode,
    tour_id: Option<&str>,
) -> Result<ShepherdTourConfig, String> {
    let tour = select_tour(document, tour_id)?;
    let triggers = tour
        .children
        .iter()
        .filter_map(|child| match child {
            Node::Trigger(trigger) => Some(trigger.clone()),
            _ => None,
        })
        .collect();
    let steps = ordered_steps(tour)
        .into_iter()
        .map(|step| {
            let mut meta = Map::new();
            if let Some(order) = &step.order {
                meta.insert("order".to_string(), order.clone());
            }
            ShepherdStep {
                id: step.id.clone(),
                title: step.title.clone(),
                text: step
                    .body
                    .clone()
                    .or_else(|| step.markdown.clone())
                    .unwrap_or_default(),
                attach_to: AttachTo {
                    element: step.target.clone().unwrap_or_default(),
                    on: step.position.clone(),
                },
                when: step.when.clone(),
                options: step.options.clone(),
                meta,
            }
        })
        .collect();

    Ok(ShepherdTourConfig {
        id: tour.id.clone(),
        title: tour.title.clone(),
        description: tour.description.clone(),
        options: tour.options.clone(),
        triggers,
        steps,
    })
}

pub fn to_intro_config(
    document: &DocumentNode,
    tour_id: Option<&str>,
) -> Result<IntroTourConfig, String> {
    let tour = select_tour(document, tour_id)?;
    let triggers = tour
        .children
        .iter()
        .filter_map(|child| match child {
            Node::Trigger(trigger) => Some(trigger.clone()),
            _ => None,
        })
        .collect::<Vec<TriggerNode>>();
    let steps = ordered_steps(tour)
        .into_iter()
        .map(|step| {
            let mut meta = Map::new();
            if let Some(id) = &step.id {
                meta.insert("id".to_string(), Value::String(id.clone()));
            }
            if let Some(order) = &step.order {
                meta.insert("order".to_string(), order.clone());
            }
            if let Some(when) = &step.when {
                meta.insert("when".to_string(), Value::String(when.clone()));
            }
            IntroStep {
                element: step.target.clone().unwrap_or_default(),
                title: step.title.clone(),
                intro: step
                    .body
                    .clone()
                    .or_else(|| step.markdown.clone())
                    .unwrap_or_default(),
                position: step.position.clone(),
                options: step.options.clone(),
                meta,
            }
        })
        .collect();

    Ok(IntroTourConfig {
        id: tour.id.clone(),
        title: tour.title.clone(),
        description: tour.description.clone(),
        options: tour.options.clone(),
        triggers,
        steps,
    })
}

fn select_tour<'a>(
    document: &'a DocumentNode,
    tour_id: Option<&str>,
) -> Result<&'a TourNode, String> {
    let tours = document.children.iter().flat_map(|child| match child {
        Node::Tour(tour) => vec![tour],
        Node::Page(page) => page
            .children
            .iter()
            .filter_map(|node| match node {
                Node::Tour(tour) => Some(tour),
                _ => None,
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    });

    if let Some(tour_id) = tour_id {
        tours
            .into_iter()
            .find(|tour| tour.id.as_deref() == Some(tour_id))
            .ok_or_else(|| format!("Tour \"{tour_id}\" was not found."))
    } else {
        tours
            .into_iter()
            .next()
            .ok_or_else(|| "No tour was found.".to_string())
    }
}

fn ordered_steps(tour: &TourNode) -> Vec<&StepNode> {
    let mut steps = tour
        .children
        .iter()
        .filter_map(|child| match child {
            Node::Step(step) => Some(step),
            _ => None,
        })
        .collect::<Vec<_>>();
    steps.sort_by(|left, right| {
        match (
            left.order.as_ref().and_then(Value::as_f64),
            right.order.as_ref().and_then(Value::as_f64),
        ) {
            (Some(left), Some(right)) => left
                .partial_cmp(&right)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    steps
}
