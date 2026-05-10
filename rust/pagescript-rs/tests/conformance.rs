use std::fs;

use pagescript_rs::{
    compile_page_ir, parse_page_script, render_to_html, to_intro_config, to_shepherd_config,
    validate_document,
};
use serde_json::Value;

#[test]
fn valid_fixture_matches_expected_ast() {
    let source = fs::read_to_string("../../conformance/valid/basic.page").unwrap();
    let expected: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/valid/basic.ast.json").unwrap(),
    )
    .unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document), Vec::new());
    assert_eq!(serde_json::to_value(&document).unwrap(), expected);
}

#[test]
fn invalid_fixture_matches_expected_diagnostics() {
    let source = fs::read_to_string("../../conformance/invalid/required-fields.page").unwrap();
    let expected: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/invalid/required-fields.diagnostics.json").unwrap(),
    )
    .unwrap();
    let document = parse_page_script(&source);
    let diagnostics = validate_document(&document);

    assert_eq!(serde_json::to_value(&diagnostics).unwrap(), expected);
}

#[test]
fn converts_nested_page_tours_to_runtime_configs() {
    let source = fs::read_to_string("../../examples/dashboard.page").unwrap();
    let document = parse_page_script(&source);
    let shepherd = to_shepherd_config(&document, Some("dashboard-onboarding")).unwrap();
    let intro = to_intro_config(&document, Some("dashboard-onboarding")).unwrap();

    assert_eq!(shepherd.id.as_deref(), Some("dashboard-onboarding"));
    assert_eq!(shepherd.steps[0].attach_to.element, ".hero");
    assert_eq!(shepherd.steps[0].attach_to.on.as_deref(), Some("bottom"));
    assert_eq!(intro.steps[0].element, ".hero");
    assert_eq!(intro.steps[0].position.as_deref(), Some("bottom"));
}

#[test]
fn number_literals_match_draft_grammar() {
    let document = parse_page_script(
        r##"::tour id=numbers
  ::step id=valid target="#valid" order=-12.5
  ::/step
  ::step id=invalid target="#invalid" order=1.
  ::/step
::/tour"##,
    );
    let tour = document
        .children
        .iter()
        .find_map(|node| match node {
            pagescript_rs::Node::Tour(tour) => Some(tour),
            _ => None,
        })
        .unwrap();
    let steps = tour
        .children
        .iter()
        .filter_map(|node| match node {
            pagescript_rs::Node::Step(step) => Some(step),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(steps[0].order, Some(Value::from(-12.5)));
    assert_eq!(steps[1].order, None);
    assert_eq!(steps[1].attributes["order"], Value::String("1.".into()));
}

#[test]
fn malformed_directive_reports_diagnostic() {
    let document = parse_page_script("::tour!\n");
    let diagnostics = validate_document(&document);

    assert_eq!(diagnostics[0].code, "malformed_directive");
}

#[test]
fn parses_and_renders_interactive_page_components() {
    let source = fs::read_to_string("../../examples/interactive-doc.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document), Vec::new());
    let html = render_to_html(&document, Some("agent-docs")).unwrap();

    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("Interactive docs your AI tools can write"));
    assert!(html.contains("data-action=\"open-modal\""));
    assert!(html.contains("<dialog class=\"ps-modal\" id=\"workflow\">"));
}

#[test]
fn parses_and_renders_data_lineage_demo() {
    let source = fs::read_to_string("../../examples/data-lineage-demo.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document), Vec::new());
    let html = render_to_html(&document, Some("lineage-demo")).unwrap();

    assert!(html.contains("<svg class=\"ps-graph\""));
    assert!(html.contains("data-ps-node=\"warehouse\""));
    assert!(html.contains("ps-effect-flow"));
    assert!(html.contains("ps-runtime-config"));
    assert!(html.contains("node.click"));
    assert!(html.contains(".lineage-hero"));
    assert!(html.contains("viewBox=\"0 0 760 380\""));
    assert!(!html.contains("transform:scale"));
    assert!(!source.contains("<script"));
}

#[test]
fn renders_revenue_command_center_without_svg_animation_drift() {
    let source =
        fs::read_to_string("../../examples/autonomous-revenue-command-center.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document), Vec::new());
    let html = render_to_html(&document, Some("revenue-command-center")).unwrap();

    assert!(html.contains("data-ps-node=\"identity\""));
    assert!(html.contains("ps-pulse-svg"));
    assert!(html.contains("width=\"148\" height=\"58\""));
    assert!(html.contains("M 179 90 C"));
    assert!(!html.contains("transform:scale"));
    assert!(!source.contains("<script"));
}

#[test]
fn compiles_revenue_command_center_to_generic_page_ir() {
    let source =
        fs::read_to_string("../../examples/autonomous-revenue-command-center.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document), Vec::new());
    let ir = compile_page_ir(&document, Some("revenue-command-center")).unwrap();
    let first_scene = ir
        .body
        .iter()
        .filter_map(|node| match node {
            pagescript_rs::IrNode::Component(component) if component.name == "scene" => {
                Some(component)
            }
            _ => None,
        })
        .next()
        .unwrap();
    let graph = first_scene
        .children
        .iter()
        .filter_map(|node| match node {
            pagescript_rs::IrNode::Component(component) => component.graph.as_ref(),
            _ => None,
        })
        .next()
        .unwrap();

    assert_eq!(ir.title, "Autonomous Revenue Command Center");
    assert_eq!(ir.tokens["color.accent"], Value::String("#4dd6a0".into()));
    assert_eq!(ir.states[0].id, "selectedNode");
    let first_panel = match &first_scene.children[0] {
        pagescript_rs::IrNode::Component(component) => component,
        _ => panic!("expected panel component"),
    };
    assert_eq!(
        first_panel.layout.as_ref().unwrap().density.as_deref(),
        Some("spacious")
    );
    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.edges.len(), 5);
    assert_eq!(graph.nodes[0].id, "signals");
}

#[test]
fn renders_tokens_and_generic_layout_metadata() {
    let source = r##"::page id=tokens title="Tokens"
  ::tokens
    color.accent="#ff0055"
    color.bg="#101010"
    radius.panel=20
  ::/tokens

  ::grid columns=4 gap=lg density=compact
    ::card title="One" body="Tokenized card"
    ::/card
  ::/grid
::/page"##;
    let document = parse_page_script(source);

    assert_eq!(validate_document(&document), Vec::new());
    let ir = compile_page_ir(&document, Some("tokens")).unwrap();
    let html = render_to_html(&document, Some("tokens")).unwrap();

    let component = ir
        .body
        .iter()
        .find_map(|node| match node {
            pagescript_rs::IrNode::Component(component) => Some(component),
            _ => None,
        })
        .unwrap();

    assert_eq!(ir.tokens["color.accent"], Value::String("#ff0055".into()));
    assert_eq!(component.layout.as_ref().unwrap().columns, Some(4));
    assert_eq!(
        component.layout.as_ref().unwrap().density.as_deref(),
        Some("compact")
    );
    assert!(html.contains("--ps-token-color-accent:#ff0055;"));
    assert!(html.contains("--ps-accent:#ff0055;"));
    assert!(html.contains("--ps-radius:20px;"));
    assert!(html.contains("ps-density-compact"));
}

#[test]
fn visual_sanity_checks_guard_graph_alignment_regressions() {
    let source =
        fs::read_to_string("../../examples/autonomous-revenue-command-center.page").unwrap();
    let document = parse_page_script(&source);
    let html = render_to_html(&document, Some("revenue-command-center")).unwrap();

    assert!(html.contains("transform=\"translate(265, 190)\""));
    assert!(html.contains("d=\"M 179 90 C 227 90, 143 190, 191 190\""));
    assert!(html.contains(".ps-graph-node.ps-effect-pulse"));
    assert!(!html.contains("transform:scale"));
    assert!(!html.contains("data-ps-node=\"\""));
}

#[test]
fn validates_draft_04_primitive_rules() {
    let source = r##"::page id=bad
  ::scene title="Missing id"
  ::/scene
  ::node id=dup label="One"
  ::/node
  ::node id=dup label="Two"
  ::/node
  ::edge from=dup
  ::/edge
  ::effect id=spin type=spin
  ::/effect
  ::style scope=global
  ::/style
  ::tokens options={"nested":true}
  ::/tokens
::/page"##;
    let diagnostics = validate_document(&parse_page_script(source));
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();

    assert!(codes.contains(&"missing_component_attribute"));
    assert!(codes.contains(&"duplicate_node_id"));
    assert!(codes.contains(&"invalid_effect_type"));
    assert!(codes.contains(&"invalid_style_scope"));
    assert!(codes.contains(&"invalid_token_value"));
}
