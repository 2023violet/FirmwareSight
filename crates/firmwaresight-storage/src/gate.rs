//! Persisting an immutable Gate run, and the review acceptances that dispose of a finding.
//!
//! The module has one read half as well: [`Database::load_gate_facts`] hydrates the five fact groups a
//! Gate run needs out of the rows Analyze already wrote, so the desktop can gate a build it stored
//! earlier without the original files being present. It returns rows, never verdicts.
//!
//! Storage keeps no Gate semantics: it writes the states Core produced, refuses a run whose recorded
//! answer differs from the one already stored under that id, and hands back what it stored. The
//! invariants that a UI or a CLI must not be able to violate — `UNKNOWN` is never a PASS, an accepted
//! review keeps its finding at `REVIEW`, an audit row cannot be edited — are CHECK constraints and
//! triggers in migration `0003`, so they hold for a hand-written `INSERT` too.
//!
//! Four rules from prompt §33 and §54 govern this module:
//!
//! - one transaction per run: the run row, its findings and every evidence ref commit together, or the
//!   run does not exist at all;
//! - a run id is content-addressed, so re-running the same input dedupes instead of duplicating;
//! - the same run id with different semantics is an invariant violation, not an update;
//! - acceptance never touches a finding. It adds a row, and the finding keeps saying `REVIEW`.

use rusqlite::{OptionalExtension, params};

use crate::Database;
use crate::compare::{StoredBudget, read_err};
use crate::db::write_err;
use crate::error::StorageError;
use firmwaresight_core::domain::gate::{
    EffectiveSeverity, FindingState, GateEvaluation, GateRuleId,
};

/// How many `Unknown` evidence ids a read returns as a sample. The count below is always complete;
/// the list is a bounded excerpt, so a snapshot with ten thousand gaps does not turn one Gate run into
/// a ten-thousand-row read (`AGENTS.md` 10, prompt §25). Eight is the number a Gate finding quotes.
const MAX_UNKNOWN_EVIDENCE_SAMPLE: usize = 8;

/// One artifact row of a stored build, reduced to what the Gate reads. The stored `path` is dropped
/// here, not redacted later: no caller of this API can receive a host path (`AGENTS.md` 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateArtifactRow {
    /// The stored kind spelling, `Elf`/`Map`/`Bin`/`IntelHex`/`Unknown`.
    pub kind: String,
    pub sha256: String,
    pub byte_size: u64,
}

/// A stored footprint, with the two totals and the evidence strength they were recorded at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFootprintRow {
    pub layout_source: String,
    pub weakest_basis: Option<String>,
    /// Whether the accounting that produced these totals may support a hard verdict.
    pub admissible_hard_block: bool,
    pub nonvolatile: StoredBudget,
    pub runtime_ram: StoredBudget,
}

/// How much of a stored snapshot's own evidence is classified `unknown`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateEvidenceGaps {
    /// Every `unknown` row for this build, which is why the count and the sample are separate.
    pub count: usize,
    /// Evidence ids, ordered by id, capped at eight.
    pub sample_ids: Vec<String>,
}

/// Everything the Gate reads about one persisted build.
///
/// This is the stored counterpart of a fresh analysis: the same five fact groups, assembled from rows
/// instead of from files. Nothing is decided here — no state, severity or threshold appears in this
/// shape, because `firmwaresight-core` owns that and the desktop must get the same answer the CLI
/// would (`AGENTS.md` 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateSnapshotFacts {
    pub build_id: String,
    pub snapshot_id: String,
    pub artifacts: Vec<GateArtifactRow>,
    pub footprint: GateFootprintRow,
    pub gaps: GateEvidenceGaps,
}

/// How a run arrived: written now, or already stored with exactly these semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateRunWrite {
    Inserted,
    AlreadyStored,
}

/// One Gate run as the caller means to store it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateRunDraft<'a> {
    /// `gate-<sha256>`, from `firmwaresight-project`'s fingerprint of the Gate input.
    pub run_id: &'a str,
    /// The persisted build this verdict is about. A run is always tied to a stored build, because
    /// that is what makes the record reviewable after the files are gone.
    pub build_id: &'a str,
    pub baseline_build_id: Option<&'a str>,
    pub policy_sha256: &'a str,
    pub evaluation: &'a GateEvaluation,
}

/// A finding as it was stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredGateFinding {
    pub id: String,
    /// The stable rule identity. Parsing to [`GateRuleId`] is the caller's business; the column's CHECK
    /// already refuses any other text.
    pub rule_id: String,
    pub state: FindingState,
    pub effective_severity: EffectiveSeverity,
    pub summary: String,
    pub remediation: Option<String>,
    pub evidence_refs: Vec<String>,
}

impl StoredGateFinding {
    /// The finding's rule identity, when it is one this build knows.
    #[must_use]
    pub fn rule(&self) -> Option<GateRuleId> {
        GateRuleId::parse(&self.rule_id)
    }
}

/// A whole stored run, findings in canonical order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredGateRun {
    pub run_id: String,
    pub build_id: String,
    pub baseline_build_id: Option<String>,
    pub policy_sha256: String,
    pub overall_effective_severity: EffectiveSeverity,
    /// `gate_runs.created_at`: when FirmwareSight stored this run. Not a build time.
    pub created_at: String,
    pub findings: Vec<StoredGateFinding>,
}

impl StoredGateRun {
    /// The findings that are acceptable as reviews, in canonical order.
    #[must_use]
    pub fn acceptable_findings(&self) -> Vec<&StoredGateFinding> {
        self.findings
            .iter()
            .filter(|finding| finding.state == FindingState::Review)
            .collect()
    }
}

/// An accepted review, exactly as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedReview {
    pub run_id: String,
    pub finding_id: String,
    pub actor: String,
    /// `accepted_reviews.accepted_at`, written by SQLite at insertion.
    pub accepted_at: String,
    pub reason: String,
    /// Always `REVIEW`: ADR-0023 requires the record to name what was accepted, and only a review is
    /// acceptable.
    pub original_state: FindingState,
}

/// Why an acceptance was refused. Each variant is a different answer for the release owner, so they are
/// not collapsed into one storage failure. `Clone` and `PartialEq` are deliberately absent: the
/// `Storage` branch carries a `rusqlite::Error`, which is neither, so callers match on the variant
/// instead of comparing values.
#[derive(Debug, thiserror::Error)]
pub enum AcceptReviewError {
    #[error("Gate run `{run_id}` has no finding `{finding_id}`")]
    FindingNotFound { run_id: String, finding_id: String },
    #[error(
        "only a REVIEW finding can be accepted; `{finding_id}` is {state}. A BLOCK needs the condition \
         fixed, an UNKNOWN needs evidence, and a PASS needs nothing"
    )]
    NotReview {
        run_id: String,
        finding_id: String,
        state: String,
    },
    #[error("finding `{finding_id}` in run `{run_id}` was already accepted by {actor}")]
    AlreadyAccepted {
        run_id: String,
        finding_id: String,
        actor: String,
    },
    #[error("a review acceptance has to name the person who accepted it")]
    EmptyActor,
    #[error("a review acceptance has to state the reason it was accepted")]
    EmptyReason,
    #[error(transparent)]
    Storage(#[from] StorageError),
}

impl AcceptReviewError {
    #[must_use]
    pub fn stable_code(&self) -> &'static str {
        match self {
            Self::FindingNotFound { .. } => "ERR-STORAGE-4007",
            Self::NotReview { .. } => "ERR-STORAGE-4008",
            Self::AlreadyAccepted { .. } => "ERR-STORAGE-4009",
            Self::EmptyActor | Self::EmptyReason => "ERR-STORAGE-4010",
            Self::Storage(source) => source.stable_code(),
        }
    }
}

impl Database {
    /// Store one Gate run atomically, or recognize that it is already stored.
    ///
    /// # Errors
    ///
    /// [`StorageError::Write`] when the build the run names is not persisted, or a constraint the
    /// caller's data violated; [`StorageError::Invariant`] when the same run id already holds a
    /// different answer, which means the fingerprint collided with different semantics and no write
    /// happens.
    pub fn persist_gate_run(
        &mut self,
        draft: &GateRunDraft<'_>,
    ) -> Result<GateRunWrite, StorageError> {
        if draft.run_id != draft.evaluation.run_id {
            return Err(StorageError::Invariant {
                detail: format!(
                    "the run being stored is identified as `{}` but was handed the id `{}`",
                    draft.evaluation.run_id, draft.run_id
                ),
            });
        }
        if let Some(existing) = self.gate_run_by_id(draft.run_id)? {
            return if matches_existing(&existing, draft) {
                Ok(GateRunWrite::AlreadyStored)
            } else {
                Err(StorageError::Invariant {
                    detail: format!(
                        "Gate run `{}` is already stored with a different result; a changed policy, \
                         workspace or baseline produces a new run id rather than rewriting this one",
                        draft.run_id
                    ),
                })
            };
        }

        let tx = self.conn.transaction().map_err(write_err)?;
        tx.execute(
            "INSERT INTO gate_runs
                 (id, build_id, baseline_build_id, policy_sha256, overall_effective_severity)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                draft.run_id,
                draft.build_id,
                draft.baseline_build_id,
                draft.policy_sha256,
                draft.evaluation.overall_effective_severity.as_str(),
            ],
        )
        .map_err(|source| describe_write(source, draft.run_id))?;

        for (ordinal, finding) in draft.evaluation.findings.iter().enumerate() {
            tx.execute(
                "INSERT INTO gate_findings
                     (run_id, id, rule_id, state, effective_severity, summary, remediation, ordinal)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    draft.run_id,
                    finding.id,
                    finding.rule_id.as_str(),
                    finding.state.as_str(),
                    finding.effective_severity.as_str(),
                    finding.summary,
                    finding.remediation,
                    ordinal as i64,
                ],
            )
            .map_err(|source| describe_write(source, &finding.id))?;

            for (position, reference) in finding.evidence_refs.iter().enumerate() {
                tx.execute(
                    "INSERT INTO gate_finding_evidence (run_id, finding_id, ordinal, evidence_ref)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![draft.run_id, finding.id, position as i64, reference],
                )
                .map_err(|source| describe_write(source, &finding.id))?;
            }
        }

        tx.commit().map_err(write_err)?;
        Ok(GateRunWrite::Inserted)
    }

    /// Read one stored run with its findings and evidence refs, in the order the run recorded them.
    ///
    /// # Errors
    ///
    /// Any SQLite failure is reported as [`StorageError::Write`].
    pub fn gate_run_by_id(&self, run_id: &str) -> Result<Option<StoredGateRun>, StorageError> {
        let head = self
            .conn
            .query_row(
                "SELECT build_id, baseline_build_id, policy_sha256, overall_effective_severity, created_at
                   FROM gate_runs WHERE id = ?1",
                params![run_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(write_err)?;
        let Some((build_id, baseline_build_id, policy_sha256, severity, created_at)) = head else {
            return Ok(None);
        };
        let overall_effective_severity =
            EffectiveSeverity::parse(&severity).ok_or_else(|| StorageError::Invariant {
                detail: format!("stored run `{run_id}` carries the severity `{severity}`"),
            })?;
        let findings = self.gate_findings(run_id)?;
        Ok(Some(StoredGateRun {
            run_id: run_id.to_owned(),
            build_id,
            baseline_build_id,
            policy_sha256,
            overall_effective_severity,
            created_at,
            findings,
        }))
    }

    /// Record that a person accepted one `REVIEW` finding of one run.
    ///
    /// The finding row is not touched: it keeps its `REVIEW` state and its `REVIEW` effective severity,
    /// and the aggregate a caller computes from the run plus these rows is what moves (prompt §34, §48).
    pub fn accept_review(
        &mut self,
        run_id: &str,
        finding_id: &str,
        actor: &str,
        reason: &str,
    ) -> Result<AcceptedReview, AcceptReviewError> {
        if actor.trim().is_empty() {
            return Err(AcceptReviewError::EmptyActor);
        }
        if reason.trim().is_empty() {
            return Err(AcceptReviewError::EmptyReason);
        }
        let state: Option<String> = self
            .conn
            .query_row(
                "SELECT state FROM gate_findings WHERE run_id = ?1 AND id = ?2",
                params![run_id, finding_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(accept_err)?;
        let Some(state) = state else {
            return Err(AcceptReviewError::FindingNotFound {
                run_id: run_id.to_owned(),
                finding_id: finding_id.to_owned(),
            });
        };
        if state != FindingState::Review.as_str() {
            return Err(AcceptReviewError::NotReview {
                run_id: run_id.to_owned(),
                finding_id: finding_id.to_owned(),
                state,
            });
        }
        if let Some(existing) = self.accepted_review_for(run_id, finding_id)? {
            return Err(AcceptReviewError::AlreadyAccepted {
                run_id: run_id.to_owned(),
                finding_id: finding_id.to_owned(),
                actor: existing.actor,
            });
        }
        let tx = self.conn.transaction().map_err(accept_err)?;
        tx.execute(
            "INSERT INTO accepted_reviews (run_id, finding_id, actor, reason, original_state)
             VALUES (?1, ?2, ?3, ?4, 'REVIEW')",
            params![run_id, finding_id, actor.trim(), reason.trim()],
        )
        .map_err(|source| describe_write(source, finding_id))?;
        // The timestamp is SQLite's insertion moment, read back rather than guessed at here, so the
        // audit row states when the database recorded the decision.
        let stored = read_acceptance(&tx, run_id, finding_id)?;
        tx.commit().map_err(accept_err)?;
        stored.ok_or_else(|| {
            AcceptReviewError::Storage(StorageError::Invariant {
                detail: format!("the acceptance for `{finding_id}` was written and then not found"),
            })
        })
    }

    /// Every acceptance recorded for one run, oldest first: the audit trail a reviewer reads.
    pub fn accepted_reviews_for_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<AcceptedReview>, StorageError> {
        let mut statement = self
            .conn
            .prepare(&format!(
                "{ACCEPTANCE_SELECT} ORDER BY a.accepted_at, a.finding_id"
            ))
            .map_err(|source| describe_write(source, run_id))?;
        let rows = statement
            .query_map(params![run_id], acceptance_columns)
            .map_err(|source| describe_write(source, run_id))?;
        let mut acceptances = Vec::new();
        for row in rows {
            let columns = row.map_err(|source| describe_write(source, run_id))?;
            acceptances.push(acceptance_from_columns(columns, run_id)?);
        }
        Ok(acceptances)
    }

    /// Hydrate the facts one persisted build offers the Gate.
    ///
    /// Rows only: the artifact files are never opened, so a run stays possible after the ELF and MAP
    /// have moved or been deleted, and the verdict a reviewer accepted yesterday is re-readable today.
    /// The stored `path` of each artifact is dropped at the row boundary.
    ///
    /// # Errors
    ///
    /// - [`StorageError::NotFound`] when no build carries this snapshot id;
    /// - [`StorageError::Write`] when a read fails;
    /// - [`StorageError::Invariant`] when a build is missing its footprint row, which no supported
    ///   import path can produce.
    pub fn load_gate_facts(&self, snapshot_id: &str) -> Result<GateSnapshotFacts, StorageError> {
        let build_id =
            self.build_id_for_snapshot(snapshot_id)?
                .ok_or_else(|| StorageError::NotFound {
                    id: snapshot_id.to_owned(),
                })?;

        let mut statement = self
            .conn
            .prepare(
                "SELECT kind, sha256, byte_size FROM artifacts WHERE build_id = ?1 ORDER BY id",
            )
            .map_err(read_err)?;
        let artifacts = statement
            .query_map(params![build_id], |row| {
                Ok(GateArtifactRow {
                    kind: row.get(0)?,
                    sha256: row.get(1)?,
                    byte_size: row.get::<_, i64>(2)? as u64,
                })
            })
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;

        let footprint = self
            .conn
            .query_row(
                "SELECT layout_source, weakest_basis, admissible_hard_block,
                        nonvolatile_state, nonvolatile_bytes, runtime_state, runtime_bytes
                   FROM memory_footprints WHERE build_id = ?1",
                params![build_id],
                |row| {
                    Ok(GateFootprintRow {
                        layout_source: row.get(0)?,
                        weakest_basis: row.get(1)?,
                        admissible_hard_block: row.get::<_, i64>(2)? == 1,
                        nonvolatile: StoredBudget::from_parts(row.get(3)?, row.get(4)?),
                        runtime_ram: StoredBudget::from_parts(row.get(5)?, row.get(6)?),
                    })
                },
            )
            .optional()
            .map_err(read_err)?
            .ok_or_else(|| StorageError::Invariant {
                detail: format!("build {build_id} has no stored footprint row"),
            })?;

        let count = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM evidence WHERE build_id = ?1 AND classification = 'unknown'",
                params![build_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(read_err)? as usize;
        let mut statement = self
            .conn
            .prepare(
                "SELECT id FROM evidence WHERE build_id = ?1 AND classification = 'unknown'
                   ORDER BY id LIMIT ?2",
            )
            .map_err(read_err)?;
        let sample_ids = statement
            .query_map(
                params![build_id, MAX_UNKNOWN_EVIDENCE_SAMPLE as i64],
                |row| row.get::<_, String>(0),
            )
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;

        Ok(GateSnapshotFacts {
            snapshot_id: snapshot_id.to_owned(),
            build_id,
            artifacts,
            footprint,
            gaps: GateEvidenceGaps { count, sample_ids },
        })
    }

    fn accepted_review_for(
        &self,
        run_id: &str,
        finding_id: &str,
    ) -> Result<Option<AcceptedReview>, StorageError> {
        let columns = self
            .conn
            .query_row(
                &format!("{ACCEPTANCE_SELECT} AND a.finding_id = ?2"),
                params![run_id, finding_id],
                acceptance_columns,
            )
            .optional()
            .map_err(|source| describe_write(source, finding_id))?;
        columns
            .map(|columns| acceptance_from_columns(columns, finding_id))
            .transpose()
    }

    fn gate_findings(&self, run_id: &str) -> Result<Vec<StoredGateFinding>, StorageError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, rule_id, state, effective_severity, summary, remediation
                   FROM gate_findings WHERE run_id = ?1 ORDER BY ordinal",
            )
            .map_err(|source| describe_write(source, run_id))?;
        let rows = statement
            .query_map(params![run_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|source| describe_write(source, run_id))?;
        let mut findings = Vec::new();
        for row in rows {
            let (id, rule_id, state, severity, summary, remediation) =
                row.map_err(|source| describe_write(source, run_id))?;
            let Some(state) = FindingState::parse(&state) else {
                return Err(StorageError::Invariant {
                    detail: format!("stored finding `{id}` carries the state `{state}`"),
                });
            };
            let Some(effective_severity) = EffectiveSeverity::parse(&severity) else {
                return Err(StorageError::Invariant {
                    detail: format!("stored finding `{id}` carries the severity `{severity}`"),
                });
            };
            let refs = self.evidence_refs(run_id, &id)?;
            findings.push(StoredGateFinding {
                id,
                rule_id,
                state,
                effective_severity,
                summary,
                remediation,
                evidence_refs: refs,
            });
        }
        Ok(findings)
    }

    fn evidence_refs(&self, run_id: &str, finding_id: &str) -> Result<Vec<String>, StorageError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT evidence_ref FROM gate_finding_evidence
                  WHERE run_id = ?1 AND finding_id = ?2 ORDER BY ordinal",
            )
            .map_err(|source| describe_write(source, finding_id))?;
        let rows = statement
            .query_map(params![run_id, finding_id], |row| row.get::<_, String>(0))
            .map_err(|source| describe_write(source, finding_id))?;
        let mut references = Vec::new();
        for row in rows {
            references.push(row.map_err(|source| describe_write(source, finding_id))?);
        }
        Ok(references)
    }
}

/// Findings and refs are compared to the draft so a dedupe means "identical", not "same id".
fn matches_existing(existing: &StoredGateRun, draft: &GateRunDraft<'_>) -> bool {
    if existing.build_id != draft.build_id
        || existing.baseline_build_id.as_deref() != draft.baseline_build_id
        || existing.policy_sha256 != draft.policy_sha256
        || existing.overall_effective_severity != draft.evaluation.overall_effective_severity
        || existing.findings.len() != draft.evaluation.findings.len()
    {
        return false;
    }
    existing
        .findings
        .iter()
        .zip(&draft.evaluation.findings)
        .all(|(stored, fresh)| {
            stored.id == fresh.id
                && stored.rule_id == fresh.rule_id.as_str()
                && stored.state == fresh.state
                && stored.effective_severity == fresh.effective_severity
                && stored.summary == fresh.summary
                && stored.remediation == fresh.remediation
                && stored.evidence_refs == fresh.evidence_refs
        })
}

const ACCEPTANCE_SELECT: &str = "SELECT a.run_id, a.finding_id, a.actor, a.accepted_at, a.reason, \
                                 a.original_state \
                                    FROM accepted_reviews a WHERE a.run_id = ?1";

/// The six acceptance columns, read as text. The state is parsed by [`acceptance_from_columns`],
/// which is where a corrupt value can become a typed storage error instead of a panic.
fn acceptance_columns(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<(String, String, String, String, String, String)> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
    ))
}

fn acceptance_from_columns(
    (run_id, finding_id, actor, accepted_at, reason, original_state): (
        String,
        String,
        String,
        String,
        String,
        String,
    ),
    subject: &str,
) -> Result<AcceptedReview, StorageError> {
    let Some(state) = FindingState::parse(&original_state) else {
        return Err(StorageError::Invariant {
            detail: format!("acceptance `{subject}` records the original state `{original_state}`"),
        });
    };
    Ok(AcceptedReview {
        run_id,
        finding_id,
        actor,
        accepted_at,
        reason,
        original_state: state,
    })
}

fn read_acceptance(
    tx: &rusqlite::Transaction<'_>,
    run_id: &str,
    finding_id: &str,
) -> Result<Option<AcceptedReview>, StorageError> {
    let columns = tx
        .query_row(
            &format!("{ACCEPTANCE_SELECT} AND a.finding_id = ?2"),
            params![run_id, finding_id],
            acceptance_columns,
        )
        .optional()
        .map_err(|source| describe_write(source, finding_id))?;
    columns
        .map(|columns| acceptance_from_columns(columns, finding_id))
        .transpose()
}

/// A write failure, named by the record it concerned. A constraint text from SQLite is kept in the
/// detail, because "which invariant" is the question an inspectable error state has to answer.
fn describe_write(source: rusqlite::Error, subject: &str) -> StorageError {
    StorageError::Write {
        detail: format!("{subject}: {source}"),
    }
}

/// A SQLite failure on the acceptance path, where the caller's error type is the acceptance's own.
fn accept_err(source: rusqlite::Error) -> AcceptReviewError {
    AcceptReviewError::Storage(write_err(source))
}
