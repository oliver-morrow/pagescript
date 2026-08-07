mod adapters;
mod evidence;
mod explainer;
mod ir;
mod parser;
mod render;
mod resolver;
mod stats;
mod types;
mod validator;

pub use adapters::{to_intro_config, to_shepherd_config};
pub use evidence::{
    Callout, Citation, Confidence, Entity, EntityKind, EvidenceBundle, ExplainerCalloutIr,
    ExplainerEntityIr, ExplainerIr, ExplainerRelationshipIr, ExplainerSpec, ExplainerViewIr,
    Provenance, Relationship, RelationshipKind, Source, Subject, SubjectKind, TokenValue, View,
    ViewKind, bundle_digest, parse_evidence_bundle, parse_explainer_spec, project_explainer_ir,
    validate_evidence_bundle, validate_explainer_spec,
};
pub use explainer::render_explainer_to_html;
pub use ir::{
    ComponentIr, EffectIr, EventIr, GraphEdgeIr, GraphIr, GraphNodeIr, IrNode, LayoutIr,
    MarkdownIr, PageIr, StateIr, compile_page_ir,
};
pub use parser::{parse_page_script, parse_tour_script};
pub use render::render_to_html;
pub use resolver::Resolver;
pub use stats::{ArtifactTokenMeasure, TokenSavingsReport, measure_token_savings};
pub use types::{
    AttributeValue, ComponentNode, Diagnostic, DocumentNode, IntroStep, IntroTourConfig,
    MarkdownNode, Node, PageNode, Severity, ShepherdStep, ShepherdTourConfig, SourcePosition,
    StepNode, TourNode, TriggerNode,
};
pub use validator::validate_document;
