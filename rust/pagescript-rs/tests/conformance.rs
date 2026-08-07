use std::{collections::BTreeSet, fs, process::Command};

use jsonschema::JSONSchema;
use pagescript_rs::{
    Resolver, bundle_digest, compile_page_ir, measure_token_savings, parse_evidence_bundle,
    parse_explainer_spec, parse_page_script, project_explainer_ir, render_explainer_to_html,
    render_to_html, to_intro_config, to_shepherd_config, validate_document,
    validate_evidence_bundle, validate_explainer_spec,
};
use serde_json::Value;

fn resolver() -> Resolver {
    Resolver::new(None)
}

fn resolver_with_path(path: &str) -> Resolver {
    Resolver::new(Some(std::path::PathBuf::from(path)))
}

#[test]
fn valid_fixture_matches_expected_ast() {
    let source = fs::read_to_string("../../conformance/valid/basic.page").unwrap();
    let expected: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/valid/basic.ast.json").unwrap(),
    )
    .unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document, &resolver()), Vec::new());
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
    let diagnostics = validate_document(&document, &resolver());

    assert_eq!(serde_json::to_value(&diagnostics).unwrap(), expected);
}

#[test]
fn release_hardening_fixture_matches_expected_ast_and_ir() {
    let source = fs::read_to_string("../../conformance/valid/release-hardening.page").unwrap();
    let expected_ast: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/valid/release-hardening.ast.json").unwrap(),
    )
    .unwrap();
    let expected_ir: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/valid/release-hardening.ir.json").unwrap(),
    )
    .unwrap();
    let resolver = resolver_with_path("../../conformance/valid");
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document, &resolver), Vec::new());
    assert_eq!(serde_json::to_value(&document).unwrap(), expected_ast);
    assert_eq!(
        serde_json::to_value(
            compile_page_ir(&document, Some("release-hardening"), &resolver).unwrap()
        )
        .unwrap(),
        expected_ir
    );
}

#[test]
fn release_hardening_invalid_fixture_matches_expected_diagnostics() {
    let source = fs::read_to_string("../../conformance/invalid/release-hardening.page").unwrap();
    let expected: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/invalid/release-hardening.diagnostics.json")
            .unwrap(),
    )
    .unwrap();
    let document = parse_page_script(&source);
    let diagnostics =
        validate_document(&document, &resolver_with_path("../../conformance/invalid"));

    assert_eq!(serde_json::to_value(&diagnostics).unwrap(), expected);
}

#[test]
fn real_ast_and_ir_outputs_validate_against_public_schemas() {
    let ast_schema: Value =
        serde_json::from_str(&fs::read_to_string("../../schemas/ast.schema.json").unwrap())
            .unwrap();
    let diagnostics_schema: Value =
        serde_json::from_str(&fs::read_to_string("../../schemas/diagnostics.schema.json").unwrap())
            .unwrap();
    let ir_schema: Value =
        serde_json::from_str(&fs::read_to_string("../../schemas/page-ir.schema.json").unwrap())
            .unwrap();
    let ast_validator = JSONSchema::options()
        .with_document(
            "https://pagescript.org/schemas/diagnostics.schema.json".to_string(),
            diagnostics_schema,
        )
        .compile(&ast_schema)
        .unwrap();
    let ir_validator = JSONSchema::compile(&ir_schema).unwrap();
    let source = fs::read_to_string("../../conformance/valid/release-hardening.page").unwrap();
    let resolver = resolver_with_path("../../conformance/valid");
    let document = parse_page_script(&source);
    let ast_json = serde_json::to_value(&document).unwrap();
    let ir_json = serde_json::to_value(
        compile_page_ir(&document, Some("release-hardening"), &resolver).unwrap(),
    )
    .unwrap();

    assert_schema_valid(&ast_validator, &ast_json);
    assert_schema_valid(&ir_validator, &ir_json);
}

#[test]
fn evidence_and_explainer_fixtures_validate_against_public_schemas() {
    let evidence_schema: Value = serde_json::from_str(
        &fs::read_to_string("../../schemas/evidence-bundle.schema.json").unwrap(),
    )
    .unwrap();
    let explainer_schema: Value = serde_json::from_str(
        &fs::read_to_string("../../schemas/explainer-spec.schema.json").unwrap(),
    )
    .unwrap();
    let evidence_validator = JSONSchema::compile(&evidence_schema).unwrap();
    let explainer_validator = JSONSchema::compile(&explainer_schema).unwrap();
    let evidence: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/evidence/valid/minimal.evidence.json").unwrap(),
    )
    .unwrap();
    let invalid_evidence: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/evidence/invalid/missing-provenance.evidence.json")
            .unwrap(),
    )
    .unwrap();
    let explainer: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/explainer/valid/minimal.explainer.json").unwrap(),
    )
    .unwrap();

    assert_schema_valid(&evidence_validator, &evidence);
    assert!(evidence_validator.validate(&invalid_evidence).is_err());
    assert_schema_valid(&explainer_validator, &explainer);
}

#[test]
fn projects_a_validated_evidence_bundle_into_deterministic_explainer_ir() {
    let bundle = parse_evidence_bundle(
        &fs::read_to_string("../../conformance/evidence/valid/minimal.evidence.json").unwrap(),
    )
    .unwrap();
    let spec = parse_explainer_spec(
        &fs::read_to_string("../../conformance/explainer/valid/minimal.explainer.json").unwrap(),
    )
    .unwrap();

    assert!(validate_evidence_bundle(&bundle).is_empty());
    assert_eq!(spec.bundle_digest, bundle_digest(&bundle).unwrap());
    assert!(validate_explainer_spec(&spec, &bundle).is_empty());

    let ir = project_explainer_ir(&bundle, &spec).unwrap();

    assert_eq!(ir.views.len(), 1);
    assert_eq!(ir.views[0].entities.len(), 2);
    assert_eq!(ir.views[0].relationships.len(), 1);
    assert_eq!(
        serde_json::to_vec(&ir).unwrap(),
        serde_json::to_vec(&ir).unwrap()
    );
}

#[test]
fn explainer_renderer_emits_source_cited_standalone_html_without_network_assets() {
    let bundle = parse_evidence_bundle(
        &fs::read_to_string("../../conformance/evidence/valid/minimal.evidence.json").unwrap(),
    )
    .unwrap();
    let spec = parse_explainer_spec(
        &fs::read_to_string("../../conformance/explainer/valid/minimal.explainer.json").unwrap(),
    )
    .unwrap();
    let ir = project_explainer_ir(&bundle, &spec).unwrap();
    let html = render_explainer_to_html(&ir);

    assert!(html.contains("Fixture architecture"));
    assert!(html.contains("src/main.rs:1-4"));
    assert!(html.contains("Rendered locally with no external requests."));
    assert!(!html.contains("<script"));
    assert!(!html.contains("https://"));
}

#[test]
fn evidence_validation_rejects_invalid_citation_ranges_and_stale_spec_digests() {
    let mut bundle = parse_evidence_bundle(
        &fs::read_to_string("../../conformance/evidence/valid/minimal.evidence.json").unwrap(),
    )
    .unwrap();
    let mut spec = parse_explainer_spec(
        &fs::read_to_string("../../conformance/explainer/valid/minimal.explainer.json").unwrap(),
    )
    .unwrap();
    bundle.entities[0].provenance.evidence[0].start_line = Some(0);
    spec.bundle_digest =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string();

    assert!(
        validate_evidence_bundle(&bundle)
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_citation_line")
    );
    assert!(
        validate_explainer_spec(&spec, &bundle)
            .iter()
            .any(|diagnostic| diagnostic.code == "bundle_digest_mismatch")
    );
}

#[test]
fn reports_reproducible_o200k_token_savings_for_authored_pagescript_source() {
    let source = fs::read_to_string("../../examples/revenue-map-demo.page").unwrap();
    let resolver = resolver_with_path("../../examples");
    let document = parse_page_script(&source);
    let html = render_to_html(&document, Some("revenue-map"), &resolver).unwrap();

    let report = measure_token_savings(&source, &html).unwrap();
    let expected: Value = serde_json::from_str(
        &fs::read_to_string("../../conformance/stats/revenue-map.o200k.json").unwrap(),
    )
    .unwrap();
    let schema: Value = serde_json::from_str(
        &fs::read_to_string("../../schemas/token-savings.schema.json").unwrap(),
    )
    .unwrap();

    assert_eq!(report.tokenizer, "o200k_base");
    assert_eq!(
        report.comparison,
        "authored PageScript source vs generated standalone HTML"
    );
    assert!(report.authored_source.tokens < report.generated_html.tokens);
    assert!(report.authored_source_token_reduction_percent > 0.0);
    assert!(report.methodology.contains("excludes prompts"));
    assert_eq!(serde_json::to_value(&report).unwrap(), expected);
    assert_schema_valid(&JSONSchema::compile(&schema).unwrap(), &expected);
}

fn assert_schema_valid(schema: &JSONSchema, value: &Value) {
    if let Err(errors) = schema.validate(value) {
        panic!(
            "schema validation failed: {}",
            errors
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        );
    }
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
    let diagnostics = validate_document(&document, &resolver());

    assert_eq!(diagnostics[0].code, "malformed_directive");
}

#[test]
fn unknown_directive_reports_repair_suggestion() {
    let document = parse_page_script(
        r##"::page id=typo
  ::sectoin
  ::/sectoin
::/page
"##,
    );

    assert_eq!(document.diagnostics[0].code, "unknown_directive");
    assert!(
        document.diagnostics[0]
            .message
            .contains("Did you mean \"section\"?")
    );
}

#[test]
fn renders_semantic_ui_primitives_without_raw_html() {
    let document = parse_page_script(
        r##"::page id=semantic title="Semantic UI"
  ::nav label="Primary"
    ::nav-item label="Overview" href="#overview"
    ::/nav-item
  ::/nav
  ::section id=overview heading="Accounts"
    ::filter id=account-search label="Search accounts" placeholder="Company or owner"
    ::/filter
    ::table id=accounts
      ::column label="Account"
      ::/column
      ::column label="Status"
      ::/column
      ::row
        ::cell value="Acme"
        ::/cell
        ::cell value="Healthy"
        ::/cell
      ::/row
    ::/table
    ::empty-state title="No accounts" body="Adjust filters to widen the result set."
    ::/empty-state
  ::/section
::/page
"##,
    );

    assert_eq!(validate_document(&document, &resolver()), Vec::new());
    let html = render_to_html(&document, Some("semantic"), &resolver()).unwrap();

    assert!(html.contains("ps-nav"));
    assert!(html.contains("Search accounts"));
    assert!(html.contains("<th scope=\"col\">Account</th>"));
    assert!(html.contains("<td>Healthy</td>"));
    assert!(html.contains("ps-empty-state"));
}

#[test]
fn parses_and_renders_interactive_page_components() {
    let source = fs::read_to_string("../../examples/interactive-doc.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document, &resolver()), Vec::new());
    let html = render_to_html(&document, Some("tool-docs"), &resolver()).unwrap();

    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("Interactive docs your code generation tools can write"));
    assert!(html.contains("data-action=\"open-modal\""));
    assert!(html.contains("<dialog class=\"ps-modal\" id=\"workflow\">"));
}

#[test]
fn parses_and_renders_data_lineage_demo() {
    let source = fs::read_to_string("../../examples/data-lineage-demo.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document, &resolver()), Vec::new());
    let html = render_to_html(&document, Some("lineage-demo"), &resolver()).unwrap();

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
fn renders_revenue_map_without_svg_animation_drift() {
    let source = fs::read_to_string("../../examples/revenue-map-demo.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document, &resolver()), Vec::new());
    let html = render_to_html(&document, Some("revenue-map"), &resolver()).unwrap();

    assert!(html.contains("data-ps-node=\"identity\""));
    assert!(html.contains("ps-effect-pulse"));
    assert!(html.contains("width=\"148\" height=\"58\""));
    assert!(html.contains("M 179 90 C"));
    assert!(!html.contains("transform:scale"));
    assert!(!source.contains("<script"));
}

#[test]
fn compiles_revenue_map_to_generic_page_ir() {
    let source = fs::read_to_string("../../examples/revenue-map-demo.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document, &resolver()), Vec::new());
    let ir = compile_page_ir(&document, Some("revenue-map"), &resolver()).unwrap();
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

    assert_eq!(ir.title, "Revenue Map Demo");
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

    assert_eq!(validate_document(&document, &resolver()), Vec::new());
    let ir = compile_page_ir(&document, Some("tokens"), &resolver()).unwrap();
    let html = render_to_html(&document, Some("tokens"), &resolver()).unwrap();

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
    let source = fs::read_to_string("../../examples/revenue-map-demo.page").unwrap();
    let document = parse_page_script(&source);
    let html = render_to_html(&document, Some("revenue-map"), &resolver()).unwrap();

    assert!(html.contains("transform=\"translate(265, 190)\""));
    assert!(html.contains("d=\"M 179 90 C 227 90, 143 190, 191 190\""));
    assert!(html.contains("ps-graph-node"));
    assert!(html.contains("ps-effect-pulse"));
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
    let diagnostics = validate_document(&parse_page_script(source), &resolver());
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

#[test]
fn rejects_style_content_that_can_terminate_the_output_style_tag() {
    let source = r##"::page id=unsafe-style
  ::style scope=page
    body { color: red; }
    </StYlE><script>globalThis.pagescriptXss = true</script><style>
  ::/style
::/page"##;
    let document = parse_page_script(source);
    let diagnostics = validate_document(&document, &resolver());

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unsafe_style_content")
    );
    assert!(render_to_html(&document, Some("unsafe-style"), &resolver()).is_err());
}

#[test]
fn emits_byte_identical_ir_across_fresh_processes() {
    let executable = env!("CARGO_BIN_EXE_pagescript");
    let mut outputs = BTreeSet::new();

    for _ in 0..10 {
        let output = Command::new(executable)
            .args([
                "ir",
                "../../examples/product-metrics-demo.page",
                "--page",
                "product-metrics-demo",
            ])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        outputs.insert(output.stdout);
    }

    assert_eq!(outputs.len(), 1, "IR output changed across fresh processes");
}

#[test]
fn rejects_executable_url_schemes_in_page_and_web_core_attributes() {
    let source = r##"::page id=unsafe-urls
  ::nav-item label="Unsafe link" href="javascript:alert(1)"
  ::/nav-item
  ::form action="javascript:alert(2)"
  ::/form
  ::el tag=a href="javascript:alert(3)"
    ::attr name=href value="javascript:alert(4)"
    ::/attr
  ::/el
::/page"##;
    let document = parse_page_script(source);
    let diagnostics = validate_document(&document, &resolver());

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unsafe_url_scheme")
    );
    assert!(render_to_html(&document, Some("unsafe-urls"), &resolver()).is_err());
}

#[test]
fn renders_web_core_kernel_recipe_expansion() {
    let source = fs::read_to_string("../../examples/web-core-kernel.page").unwrap();
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document, &resolver()), Vec::new());
    let ir = compile_page_ir(&document, Some("web-core-kernel"), &resolver()).unwrap();
    let html = render_to_html(&document, Some("web-core-kernel"), &resolver()).unwrap();

    assert!(ir.recipes.contains_key("kernel-card"));
    assert!(html.contains("<main class=\"kernel-shell\">"));
    assert!(
        html.contains(
            "<a class=\"kernel-card\" href=\"#semantic\" aria-label=\"Semantic Source\">"
        )
    );
    assert!(html.contains("Semantic components stay efficient for code generators"));
    assert!(html.contains(".kernel-card:hover"));
    assert!(html.contains("Draft 0.5 Web Core Kernel"));
    assert!(!html.contains("<use"));
    assert!(!html.contains("<script src="));
    assert!(!source.contains("<script"));
}

#[test]
fn renders_product_metrics_demo_with_stdlib_imports() {
    let source = fs::read_to_string("../../examples/product-metrics-demo.page").unwrap();
    let document = parse_page_script(&source);

    // Note: The embedded resolver will find stdlib/product.page even if physical files are missing
    let resolver = resolver_with_path("../../examples");

    assert_eq!(validate_document(&document, &resolver), Vec::new());
    let html = render_to_html(&document, Some("product-metrics-demo"), &resolver).unwrap();

    assert!(html.contains("Product Metrics"));
    assert!(html.contains("Active Users"));
    assert!(html.contains("Data Pipeline Health"));
    assert!(html.contains("Source Map"));
    assert!(html.contains("Ready to build the page?"));
    assert!(html.contains("data-ps-node=\"ingest\""));
}

#[test]
fn validates_web_core_kernel_safety_rules() {
    let source = r##"::page id=unsafe
  ::el tag=script
    ::text value="bad"
    ::/text
  ::/el
  ::attr name=onclick value="bad()"
  ::/attr
  ::use recipe=missing
  ::/use
::/page"##;
    let diagnostics = validate_document(&parse_page_script(source), &resolver());
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();

    assert!(codes.contains(&"unsafe_element_tag"));
    assert!(codes.contains(&"unsafe_attribute_name"));
    assert!(codes.contains(&"unknown_recipe"));
}

#[test]
fn validates_and_renders_recursive_imports_and_local_recipe_overrides() {
    let source = fs::read_to_string("../../conformance/valid/release-hardening.page").unwrap();
    let resolver = resolver_with_path("../../conformance/valid");
    let document = parse_page_script(&source);

    assert_eq!(validate_document(&document, &resolver), Vec::new());
    let html = render_to_html(&document, Some("release-hardening"), &resolver).unwrap();

    assert!(html.contains("Draft 0.6"));
    assert!(html.contains("Open details"));
    assert!(html.contains("Local Override"));
    assert!(html.contains("Base recipe resolved"));
    assert!(html.contains("Nested recipe resolved"));
    assert!(!html.contains("<script src="));
}

#[test]
fn rejects_invalid_import_paths_at_resolver_boundary() {
    let resolver = resolver_with_path("../../conformance/valid");

    assert!(resolver.resolve("../Cargo.toml").is_err());
    assert!(resolver.resolve("/tmp/secret.page").is_err());
}

#[test]
fn resolver_requires_an_explicit_root_for_nonstdlib_imports() {
    assert!(resolver().resolve("Cargo.toml").is_err());
}

#[test]
fn rejects_unresolvable_imports_before_ir_or_html_generation() {
    let source = r##"::page id=missing-import
  ::import from="does-not-exist.page"
  ::/import
::/page"##;
    let document = parse_page_script(source);
    let diagnostics = validate_document(&document, &resolver());

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unresolved_import")
    );
    assert!(compile_page_ir(&document, Some("missing-import"), &resolver()).is_err());
    assert!(render_to_html(&document, Some("missing-import"), &resolver()).is_err());
}

#[test]
fn rejects_recursive_recipe_expansion_before_ir_or_html_generation() {
    let source = r##"::page id=recursive-recipe
  ::recipe name=loop
    ::template
      ::use recipe=loop
      ::/use
    ::/template
  ::/recipe
  ::use recipe=loop
  ::/use
::/page"##;
    let document = parse_page_script(source);
    let diagnostics = validate_document(&document, &resolver());

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "recursive_recipe")
    );
    assert!(compile_page_ir(&document, Some("recursive-recipe"), &resolver()).is_err());
    assert!(render_to_html(&document, Some("recursive-recipe"), &resolver()).is_err());
}

#[test]
fn preserves_typed_state_and_event_values_in_ir_and_runtime_config() {
    let source = r##"::page id=typed-runtime
  ::state id=retry_count default=3
  ::/state
  ::event on=button.click set=enabled value=true
  ::/event
::/page"##;
    let document = parse_page_script(source);

    let ir = compile_page_ir(&document, Some("typed-runtime"), &resolver()).unwrap();
    let html = render_to_html(&document, Some("typed-runtime"), &resolver()).unwrap();
    let ir_schema: Value =
        serde_json::from_str(&fs::read_to_string("../../schemas/page-ir.schema.json").unwrap())
            .unwrap();

    assert_eq!(ir.states[0].default_value, Value::from(3));
    assert_eq!(ir.events[0].value, Value::Bool(true));
    assert_schema_valid(
        &JSONSchema::compile(&ir_schema).unwrap(),
        &serde_json::to_value(&ir).unwrap(),
    );
    assert!(html.contains(r#""default":3"#));
    assert!(html.contains(r#""value":true"#));
}
