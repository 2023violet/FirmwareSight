//! The portable projection of a Core `GateEvaluation`, and of the review acceptances beside it.
//!
//! These are the shapes behind `urn:firmwaresight:schema:gate-results:1` and
//! `urn:firmwaresight:schema:accepted-reviews:1` (`schemas/`), both frozen before P3 and used at major
//! version 1. The consequence that matters here: the top-level property lists are closed
//! (`additionalProperties: false`), so anything P3 needs beyond them travels in `extensions`, which is
//! the slot the schemas reserved for exactly that. Nothing renumbers a schema and no state word is
//! invented (`04_TECH/26`, prompt §36).
//!
//! Unlike the diff DTO, these two serialize with **snake_case** field names, because that is what the
//! published v1 schemas spell. Field declaration order is emitted key order, no map is used, and no
//! host path or wall-clock timestamp appears anywhere: `accepted_at` is a stored record of when a
//! decision was made, which is a fact rather than a rendering artifact.
//!
//! Nothing is recomputed here. The five states, the severities and the aggregate are Core's answers
//! (`AGENTS.md` 3); a projection that re-derived them would be a second rule book that could disagree
//! with the first.

use serde::Serialize;

use firmwaresight_core::domain::gate::{
    EffectiveSeverity, GateAttachmentFact, GateContext, GateEvaluation, GateFinding,
};

/// The identity of a gate-results document, so a reader can find the schema without a side channel.
pub const GATE_SCHEMA_ID: &str = "urn:firmwaresight:schema:gate-results:1";

/// The identity of an accepted-reviews document.
pub const ACCEPTED_REVIEWS_SCHEMA_ID: &str = "urn:firmwaresight:schema:accepted-reviews:1";

/// Both documents are at major 1; a semantics change must bump it (`04_TECH/26:29-30`).
pub const GATE_SCHEMA_VERSION: u32 = 1;

/// Counted separately from the Gate document, because each is its own published contract.
pub const ACCEPTED_REVIEWS_SCHEMA_VERSION: u32 = 1;

/// The only state an acceptance may record as accepted (ADR-0023, prompt §34).
pub const ACCEPTANCE_ORIGINAL_STATE: &str = "REVIEW";

/// One Gate run as a portable document.
#[derive(Debug, Clone, Serialize)]
pub struct GateResultsDto {
    pub schema_version: u32,
    pub run_id: String,
    pub snapshot_id: String,
    pub findings: Vec<GateFindingDto>,
    pub extensions: GateExtensionsDto,
}

/// One rule's answer. `state` and `effective_severity` are separate fields because ADR-0023 keeps them
/// separate: an `UNKNOWN` finding can carry a `BLOCK` severity, and a reader must see both.
#[derive(Debug, Clone, Serialize)]
pub struct GateFindingDto {
    pub id: String,
    pub rule_id: String,
    pub state: &'static str,
    pub effective_severity: &'static str,
    pub summary: String,
    pub evidence_refs: Vec<String>,
    pub remediation: Option<String>,
}

/// The facts v1's closed property list has no room for.
#[derive(Debug, Clone, Serialize)]
pub struct GateExtensionsDto {
    /// The semantic policy fingerprint: same values, same hash, whatever the file's formatting was.
    pub policy_sha256: String,
    /// `schema_version` of the `firmwaresight.toml` that supplied the policy.
    pub project_config_schema_version: i64,
    /// The build this run was compared against, `null` when the Gate ran without a baseline.
    pub baseline_snapshot_id: Option<String>,
    /// The aggregate Core computed for this run with no acceptances in play. Carried rather than
    /// re-derived, so no consumer implements Gate precedence.
    pub overall_effective_severity: &'static str,
    /// The canonical grammar the run id was taken over — present only for a run that bound attached bytes,
    /// because a run that bound none is documented exactly the way P4 documented one (`ADR-0030` D-4).
    /// `04_TECH/28` §7.3: the two entries arrive as a pair or not at all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_input_label: Option<&'static str>,
    /// The attached bytes this verdict was judged over, in the identity's own order. Omitted entirely when
    /// there are none, so no pre-C1 document's bytes move.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<GateAttachmentDto>,
}

/// One bound attachment, disclosed as the identity binds it: the kind its owner declared and the bytes that
/// were read. No file name and no size-derived claim travels here, because the digest fixes the length
/// (`04_TECH/28` §5 rule 1) and a name is not part of a run id.
#[derive(Debug, Clone, Serialize)]
pub struct GateAttachmentDto {
    pub kind: &'static str,
    pub sha256: String,
    pub byte_size: u64,
}

impl GateAttachmentDto {
    /// Project one canonical row. The kind word and the digest are Core's, not re-derived here.
    ///
    /// # Panics
    ///
    /// Never in a run this engine produced: an attachment whose bytes could not be hashed was refused at
    /// observation (`04_TECH/28` §5 rule 3), so every row that reached a context names a digest. The
    /// `expect` states that boundary rather than inventing a digest for a row that has none.
    #[must_use]
    pub fn of(row: &GateAttachmentFact) -> Self {
        Self {
            kind: row.kind.word(),
            sha256: row
                .sha256
                .value()
                .expect("an attached file that could not be hashed never reached a Gate context")
                .clone(),
            byte_size: row.byte_size,
        }
    }
}

/// The acceptances a release owner recorded against one run.
#[derive(Debug, Clone, Serialize)]
pub struct AcceptedReviewsDto {
    pub schema_version: u32,
    pub run_id: String,
    pub acceptances: Vec<AcceptanceDto>,
    pub extensions: AcceptedReviewsExtensionsDto,
}

/// One accepted review. The finding it accepts keeps its `REVIEW` state forever; this record is the
/// evidence that a person answered it (`04_TECH/27:43-48`).
#[derive(Debug, Clone, Serialize)]
pub struct AcceptanceDto {
    pub finding_id: String,
    pub actor: String,
    pub accepted_at: String,
    pub reason: String,
    /// Required by ADR-0023 and optional in schema v1, so an old minimal document still validates
    /// while every P3-emitted record names what was accepted (prompt §35).
    pub original_state: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct AcceptedReviewsExtensionsDto {
    /// The aggregate once these acceptances are counted. Core derives it; the document carries it so a
    /// reader never has to.
    pub overall_effective_severity: &'static str,
}

impl GateResultsDto {
    /// Project a run's verdict and the bytes that verdict was judged over.
    ///
    /// The context is a parameter rather than an option, and this is the reason: `04_TECH/28` §7.3 makes the
    /// canonical-input label and the attachment rows a pair that arrives together or not at all, and a
    /// projection built from an evaluation alone could describe a `/2` run as though it had judged nothing
    /// extra. Taking the context means the one predicate Core owns — [`GateContext::canonical_input_label`] —
    /// decides both the grammar the id was taken over and what this document says about it.
    /// `policy_sha256` and `project_config_schema_version` come from the project adapter, which owns hashing
    /// and reads the config's own version — this crate names neither.
    #[must_use]
    pub fn from_evaluation(
        context: &GateContext,
        evaluation: &GateEvaluation,
        policy_sha256: &str,
        project_config_schema_version: i64,
    ) -> Self {
        let attachments = context.canonical_attachments();
        let bound = !attachments.is_empty();
        Self {
            schema_version: GATE_SCHEMA_VERSION,
            run_id: evaluation.run_id.clone(),
            snapshot_id: evaluation.snapshot_id.clone(),
            findings: evaluation
                .findings
                .iter()
                .map(GateFindingDto::from_finding)
                .collect(),
            extensions: GateExtensionsDto {
                policy_sha256: policy_sha256.to_owned(),
                project_config_schema_version,
                baseline_snapshot_id: evaluation.baseline_snapshot_id.clone(),
                overall_effective_severity: evaluation.overall_effective_severity.as_str(),
                canonical_input_label: bound.then(|| context.canonical_input_label()),
                attachments: attachments.iter().map(GateAttachmentDto::of).collect(),
            },
        }
    }
}

impl GateFindingDto {
    #[must_use]
    pub fn from_finding(finding: &GateFinding) -> Self {
        Self {
            id: finding.id.clone(),
            rule_id: finding.rule_id.as_str().to_owned(),
            state: finding.state.as_str(),
            effective_severity: finding.effective_severity.as_str(),
            summary: finding.summary.clone(),
            evidence_refs: finding.evidence_refs.clone(),
            remediation: finding.remediation.clone(),
        }
    }
}

impl AcceptedReviewsDto {
    /// Project the acceptances of one run together with the aggregate they produce.
    #[must_use]
    pub fn new(
        run_id: &str,
        acceptances: &[AcceptanceDto],
        overall_effective_severity: EffectiveSeverity,
    ) -> Self {
        Self {
            schema_version: ACCEPTED_REVIEWS_SCHEMA_VERSION,
            run_id: run_id.to_owned(),
            acceptances: acceptances.to_vec(),
            extensions: AcceptedReviewsExtensionsDto {
                overall_effective_severity: overall_effective_severity.as_str(),
            },
        }
    }
}
