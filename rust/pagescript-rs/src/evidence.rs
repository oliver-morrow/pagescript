use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::types::{Diagnostic, error};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    Repository,
    Dbt,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    pub kind: SubjectKind,
    pub name: String,
    pub root: String,
    pub revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub path: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_pointer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Extracted,
    Inferred,
    Declared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub evidence: Vec<Citation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Module,
    Package,
    Service,
    Database,
    Dataset,
    Model,
    Source,
    Test,
    Exposure,
    File,
    Symbol,
    External,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub id: String,
    pub kind: EntityKind,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    Contains,
    Imports,
    DependsOn,
    Calls,
    Reads,
    Writes,
    Produces,
    Consumes,
    Tests,
    Documents,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub id: String,
    pub kind: RelationshipKind,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBundle {
    pub schema_version: String,
    pub subject: Subject,
    pub sources: Vec<Source>,
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    String(String),
    Number(serde_json::Number),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewKind {
    Overview,
    Architecture,
    Lineage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Callout {
    pub id: String,
    pub title: String,
    pub body: String,
    pub entities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub id: String,
    pub kind: ViewKind,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub entities: Vec<String>,
    pub relationships: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub callouts: Vec<Callout>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplainerSpec {
    pub schema_version: String,
    pub bundle_digest: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tokens: BTreeMap<String, TokenValue>,
    pub views: Vec<View>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplainerEntityIr {
    pub id: String,
    pub kind: EntityKind,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainerRelationshipIr {
    pub id: String,
    pub kind: RelationshipKind,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainerCalloutIr {
    pub id: String,
    pub title: String,
    pub body: String,
    pub entities: Vec<String>,
    pub relationships: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplainerViewIr {
    pub id: String,
    pub kind: ViewKind,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub entities: Vec<ExplainerEntityIr>,
    pub relationships: Vec<ExplainerRelationshipIr>,
    pub callouts: Vec<ExplainerCalloutIr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplainerIr {
    pub bundle_digest: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub tokens: BTreeMap<String, TokenValue>,
    pub sources: Vec<Source>,
    pub views: Vec<ExplainerViewIr>,
}

pub fn parse_evidence_bundle(source: &str) -> Result<EvidenceBundle, String> {
    serde_json::from_str(source).map_err(|error| format!("Invalid evidence bundle: {error}"))
}

pub fn parse_explainer_spec(source: &str) -> Result<ExplainerSpec, String> {
    serde_json::from_str(source)
        .map_err(|error| format!("Invalid explainer specification: {error}"))
}

pub fn validate_evidence_bundle(bundle: &EvidenceBundle) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if bundle.schema_version != "1.0" {
        diagnostics.push(domain_error(
            "unsupported_evidence_schema",
            "Evidence bundles must use schema_version \"1.0\".",
        ));
    }
    if bundle.subject.name.trim().is_empty()
        || bundle.subject.root.trim().is_empty()
        || bundle.subject.revision.trim().is_empty()
    {
        diagnostics.push(domain_error(
            "invalid_evidence_subject",
            "Evidence subjects require non-empty name, root, and revision values.",
        ));
    }

    let source_ids = validate_sources(bundle, &mut diagnostics);
    let entity_ids = validate_entities(bundle, &source_ids, &mut diagnostics);
    validate_relationships(bundle, &source_ids, &entity_ids, &mut diagnostics);
    diagnostics
}

pub fn validate_explainer_spec(spec: &ExplainerSpec, bundle: &EvidenceBundle) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if spec.schema_version != "1.0" {
        diagnostics.push(domain_error(
            "unsupported_explainer_schema",
            "Explainer specifications must use schema_version \"1.0\".",
        ));
    }
    if spec.title.trim().is_empty() {
        diagnostics.push(domain_error(
            "invalid_explainer_title",
            "Explainer specifications require a non-empty title.",
        ));
    }
    if !is_digest(&spec.bundle_digest) {
        diagnostics.push(domain_error(
            "invalid_bundle_digest",
            "Explainer bundle_digest must be a lowercase sha256 digest.",
        ));
    } else if let Ok(actual_digest) = bundle_digest(bundle)
        && spec.bundle_digest != actual_digest
    {
        diagnostics.push(domain_error(
            "bundle_digest_mismatch",
            format!(
                "Explainer specification references {}, but the bundle digest is {actual_digest}.",
                spec.bundle_digest
            ),
        ));
    }

    let entity_ids = bundle
        .entities
        .iter()
        .map(|entity| entity.id.as_str())
        .collect::<BTreeSet<_>>();
    let relationship_ids = bundle
        .relationships
        .iter()
        .map(|relationship| relationship.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut view_ids = BTreeSet::new();
    let mut callout_ids = BTreeSet::new();
    if spec.views.is_empty() {
        diagnostics.push(domain_error(
            "missing_explainer_views",
            "Explainer specifications require at least one view.",
        ));
    }

    for view in &spec.views {
        validate_id(&view.id, "view", &mut diagnostics);
        if !view_ids.insert(&view.id) {
            diagnostics.push(domain_error(
                "duplicate_view_id",
                format!("View id \"{}\" is duplicated.", view.id),
            ));
        }
        if view.title.trim().is_empty() {
            diagnostics.push(domain_error(
                "invalid_view_title",
                format!("View \"{}\" requires a non-empty title.", view.id),
            ));
        }
        validate_unique_references(
            &view.entities,
            &entity_ids,
            "entity",
            &view.id,
            &mut diagnostics,
        );
        validate_unique_references(
            &view.relationships,
            &relationship_ids,
            "relationship",
            &view.id,
            &mut diagnostics,
        );
        for callout in &view.callouts {
            validate_id(&callout.id, "callout", &mut diagnostics);
            if !callout_ids.insert(&callout.id) {
                diagnostics.push(domain_error(
                    "duplicate_callout_id",
                    format!("Callout id \"{}\" is duplicated.", callout.id),
                ));
            }
            if callout.title.trim().is_empty() || callout.body.trim().is_empty() {
                diagnostics.push(domain_error(
                    "invalid_callout_content",
                    format!(
                        "Callout \"{}\" requires non-empty title and body.",
                        callout.id
                    ),
                ));
            }
            if callout.entities.is_empty() {
                diagnostics.push(domain_error(
                    "missing_callout_entities",
                    format!("Callout \"{}\" must reference an entity.", callout.id),
                ));
            }
            validate_unique_references(
                &callout.entities,
                &entity_ids,
                "entity",
                &callout.id,
                &mut diagnostics,
            );
            validate_unique_references(
                &callout.relationships,
                &relationship_ids,
                "relationship",
                &callout.id,
                &mut diagnostics,
            );
        }
    }

    diagnostics
}

pub fn bundle_digest(bundle: &EvidenceBundle) -> Result<String, String> {
    let canonical = canonical_bundle(bundle);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| format!("Could not serialize evidence bundle: {error}"))?;
    let digest = Sha256::digest(bytes);
    Ok(format!("sha256:{digest:x}"))
}

pub fn project_explainer_ir(
    bundle: &EvidenceBundle,
    spec: &ExplainerSpec,
) -> Result<ExplainerIr, String> {
    let mut diagnostics = validate_evidence_bundle(bundle);
    diagnostics.extend(validate_explainer_spec(spec, bundle));
    if !diagnostics.is_empty() {
        return Err(format_diagnostics(&diagnostics));
    }

    let entities = bundle
        .entities
        .iter()
        .map(|entity| (entity.id.as_str(), entity))
        .collect::<BTreeMap<_, _>>();
    let relationships = bundle
        .relationships
        .iter()
        .map(|relationship| (relationship.id.as_str(), relationship))
        .collect::<BTreeMap<_, _>>();
    let mut views = spec.views.clone();
    views.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(ExplainerIr {
        bundle_digest: bundle_digest(bundle)?,
        title: spec.title.clone(),
        summary: spec.summary.clone(),
        tokens: spec.tokens.clone(),
        sources: canonical_bundle(bundle).sources,
        views: views
            .iter()
            .map(|view| project_view(view, &entities, &relationships))
            .collect(),
    })
}

fn validate_sources(
    bundle: &EvidenceBundle,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for source in &bundle.sources {
        validate_id(&source.id, "source", diagnostics);
        if !ids.insert(source.id.clone()) {
            diagnostics.push(domain_error(
                "duplicate_source_id",
                format!("Source id \"{}\" is duplicated.", source.id),
            ));
        }
        if !is_safe_source_path(&source.path) {
            diagnostics.push(domain_error(
                "invalid_source_path",
                format!(
                    "Source path \"{}\" is not a normalized relative path.",
                    source.path
                ),
            ));
        }
        if !is_digest(&source.digest) {
            diagnostics.push(domain_error(
                "invalid_source_digest",
                format!("Source \"{}\" has an invalid digest.", source.id),
            ));
        }
    }
    ids
}

fn validate_entities(
    bundle: &EvidenceBundle,
    source_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for entity in &bundle.entities {
        validate_id(&entity.id, "entity", diagnostics);
        if !ids.insert(entity.id.clone()) {
            diagnostics.push(domain_error(
                "duplicate_entity_id",
                format!("Entity id \"{}\" is duplicated.", entity.id),
            ));
        }
        if entity.label.trim().is_empty() {
            diagnostics.push(domain_error(
                "invalid_entity_label",
                format!("Entity \"{}\" requires a non-empty label.", entity.id),
            ));
        }
        validate_provenance(&entity.provenance, &entity.id, source_ids, diagnostics);
    }
    for entity in &bundle.entities {
        if let Some(group) = &entity.group
            && !ids.contains(group)
        {
            diagnostics.push(domain_error(
                "unknown_entity_group",
                format!(
                    "Entity \"{}\" references unknown group \"{group}\".",
                    entity.id
                ),
            ));
        }
    }
    ids
}

fn validate_relationships(
    bundle: &EvidenceBundle,
    source_ids: &BTreeSet<String>,
    entity_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut ids = BTreeSet::new();
    for relationship in &bundle.relationships {
        validate_id(&relationship.id, "relationship", diagnostics);
        if !ids.insert(relationship.id.clone()) {
            diagnostics.push(domain_error(
                "duplicate_relationship_id",
                format!("Relationship id \"{}\" is duplicated.", relationship.id),
            ));
        }
        for (role, entity_id) in [("from", &relationship.from), ("to", &relationship.to)] {
            if !entity_ids.contains(entity_id) {
                diagnostics.push(domain_error(
                    "unknown_relationship_endpoint",
                    format!(
                        "Relationship \"{}\" has unknown {role} entity \"{entity_id}\".",
                        relationship.id
                    ),
                ));
            }
        }
        validate_provenance(
            &relationship.provenance,
            &relationship.id,
            source_ids,
            diagnostics,
        );
    }
}

fn validate_provenance(
    provenance: &Provenance,
    owner_id: &str,
    source_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if provenance.evidence.is_empty() {
        diagnostics.push(domain_error(
            "missing_provenance_evidence",
            format!("{owner_id} requires at least one citation."),
        ));
    }
    if matches!(provenance.confidence, Confidence::Inferred)
        && provenance
            .rationale
            .as_deref()
            .is_none_or(|rationale| rationale.trim().is_empty())
    {
        diagnostics.push(domain_error(
            "missing_inference_rationale",
            format!("Inferred {owner_id} requires a rationale."),
        ));
    }
    for citation in &provenance.evidence {
        if !source_ids.contains(&citation.source) {
            diagnostics.push(domain_error(
                "unknown_citation_source",
                format!(
                    "Citation for {owner_id} references unknown source \"{}\".",
                    citation.source
                ),
            ));
        }
        if citation.start_line.is_some_and(|line| line == 0)
            || citation.end_line.is_some_and(|line| line == 0)
        {
            diagnostics.push(domain_error(
                "invalid_citation_line",
                format!("Citation for {owner_id} must use line numbers greater than zero."),
            ));
        }
        if let (Some(start), Some(end)) = (citation.start_line, citation.end_line)
            && start > end
        {
            diagnostics.push(domain_error(
                "invalid_citation_range",
                format!("Citation for {owner_id} has an inverted line range."),
            ));
        }
        if citation.start_line.is_some() != citation.end_line.is_some() {
            diagnostics.push(domain_error(
                "incomplete_citation_range",
                format!("Citation for {owner_id} must include both start_line and end_line."),
            ));
        }
        if citation
            .symbol
            .as_deref()
            .is_some_and(|symbol| symbol.trim().is_empty())
            || citation
                .json_pointer
                .as_deref()
                .is_some_and(|pointer| !pointer.starts_with('/'))
            || (citation.start_line.is_none()
                && citation.symbol.is_none()
                && citation.json_pointer.is_none())
        {
            diagnostics.push(domain_error(
                "invalid_citation_locator",
                format!("Citation for {owner_id} requires a valid locator."),
            ));
        }
    }
}

fn validate_unique_references(
    references: &[String],
    known_ids: &BTreeSet<&str>,
    kind: &str,
    owner_id: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut seen = BTreeSet::new();
    for reference in references {
        if !seen.insert(reference) {
            diagnostics.push(domain_error(
                "duplicate_explainer_reference",
                format!("{owner_id} references {kind} \"{reference}\" more than once."),
            ));
        }
        if !known_ids.contains(reference.as_str()) {
            diagnostics.push(domain_error(
                "unknown_explainer_reference",
                format!("{owner_id} references unknown {kind} \"{reference}\"."),
            ));
        }
    }
}

fn validate_id(id: &str, kind: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !is_id(id) {
        diagnostics.push(domain_error(
            "invalid_evidence_id",
            format!("{kind} id \"{id}\" must match [a-z][a-z0-9:_-]*."),
        ));
    }
}

fn project_view(
    view: &View,
    entities: &BTreeMap<&str, &Entity>,
    relationships: &BTreeMap<&str, &Relationship>,
) -> ExplainerViewIr {
    let mut view_entities = view
        .entities
        .iter()
        .filter_map(|id| entities.get(id.as_str()))
        .map(|entity| ExplainerEntityIr {
            id: entity.id.clone(),
            kind: entity.kind.clone(),
            label: entity.label.clone(),
            description: entity.description.clone(),
            group: entity.group.clone(),
            provenance: entity.provenance.clone(),
        })
        .collect::<Vec<_>>();
    view_entities.sort_by(|left, right| left.id.cmp(&right.id));

    let mut view_relationships = view
        .relationships
        .iter()
        .filter_map(|id| relationships.get(id.as_str()))
        .map(|relationship| ExplainerRelationshipIr {
            id: relationship.id.clone(),
            kind: relationship.kind.clone(),
            from: relationship.from.clone(),
            to: relationship.to.clone(),
            label: relationship.label.clone(),
            provenance: relationship.provenance.clone(),
        })
        .collect::<Vec<_>>();
    view_relationships.sort_by(|left, right| left.id.cmp(&right.id));

    let mut callouts = view
        .callouts
        .iter()
        .map(|callout| ExplainerCalloutIr {
            id: callout.id.clone(),
            title: callout.title.clone(),
            body: callout.body.clone(),
            entities: sorted_ids(&callout.entities),
            relationships: sorted_ids(&callout.relationships),
        })
        .collect::<Vec<_>>();
    callouts.sort_by(|left, right| left.id.cmp(&right.id));

    ExplainerViewIr {
        id: view.id.clone(),
        kind: view.kind.clone(),
        title: view.title.clone(),
        summary: view.summary.clone(),
        entities: view_entities,
        relationships: view_relationships,
        callouts,
    }
}

fn canonical_bundle(bundle: &EvidenceBundle) -> EvidenceBundle {
    let mut canonical = bundle.clone();
    canonical
        .sources
        .sort_by(|left, right| left.id.cmp(&right.id));
    canonical
        .entities
        .sort_by(|left, right| left.id.cmp(&right.id));
    canonical
        .relationships
        .sort_by(|left, right| left.id.cmp(&right.id));
    for entity in &mut canonical.entities {
        canonicalize_provenance(&mut entity.provenance);
    }
    for relationship in &mut canonical.relationships {
        canonicalize_provenance(&mut relationship.provenance);
    }
    canonical
}

fn canonicalize_provenance(provenance: &mut Provenance) {
    provenance.evidence.sort_by(|left, right| {
        (
            &left.source,
            left.start_line,
            left.end_line,
            &left.symbol,
            &left.json_pointer,
        )
            .cmp(&(
                &right.source,
                right.start_line,
                right.end_line,
                &right.symbol,
                &right.json_pointer,
            ))
    });
}

fn sorted_ids(ids: &[String]) -> Vec<String> {
    let mut sorted = ids.to_vec();
    sorted.sort();
    sorted
}

fn is_id(value: &str) -> bool {
    let mut characters = value.bytes();
    matches!(characters.next(), Some(b'a'..=b'z'))
        && characters
            .all(|character| matches!(character, b'a'..=b'z' | b'0'..=b'9' | b':' | b'_' | b'-'))
}

fn is_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|character| matches!(character, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_safe_source_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

fn format_diagnostics(diagnostics: &[Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn domain_error(code: &str, message: impl Into<String>) -> Diagnostic {
    error(code, message, 1)
}
