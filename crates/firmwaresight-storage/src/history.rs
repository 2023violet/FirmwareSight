//! Read-only storage support for the P5 History surface.
//!
//! P5 adds no table for History: a build, a Gate run and a release record are already persisted, by
//! Analyze, by the Gate and by a Bundle export (`P5_VALIDATION/P5_MIGRATION_DECISION.md` records the
//! decision and §18 of the P5 prompt sets the rule). What this module owns is the *reading* of that
//! history for a person who did not create it, possibly days later, possibly with the firmware files
//! already gone.
//!
//! Three rules decide every shape below, and all three are enforced here rather than trusted:
//!
//! - **Rows only.** Nothing in this module opens a file. A stored build stays a stored build after
//!   its ELF and MAP have moved or been deleted, so History needs no source artifact to be readable.
//! - **Complete imports only.** A build in `IMPORTING`, or one whose import failed, is not a
//!   snapshot and is never presented as one (`04_TECH/15` §6).
//! - **No host path.** `artifacts.path` is reduced to its file name inside this crate, per
//!   `AGENTS.md` 7 — and it is never a *filter* column either. Matching a path would turn the
//!   History search box into an oracle for directory names even though every displayed field already
//!   shows a bare name (prompt §16, §44). The searchable columns are the identities this crate
//!   stores: build, snapshot, digest, architecture, run, version, disposition.
//!
//! Ordering is a rule, not a coincidence: newest stored time first, then the row's own identity, so
//! two rows stored in the same second cannot swap places between two reads of the same database.
//! Gate dispositions and finding counts come back through the same closed parsers the Gate history
//! reader uses, so a stored word this build does not recognize fails the read instead of being
//! filed under `UNKNOWN` — the state ADR-0023 reserves for evidence that never arrived.

use std::collections::BTreeMap;

use rusqlite::types::ToSql;

use firmwaresight_core::domain::diff::display_file_name;
use firmwaresight_core::domain::gate::{EffectiveSeverity, FindingState, GateStateCounts};

use crate::Database;
use crate::compare::{
    CandidateQuery, CompareCandidate, DEFAULT_CANDIDATE_LIMIT, MAX_CANDIDATE_LIMIT,
    in_placeholders, read_err,
};
use crate::error::StorageError;
use crate::query::{Page, clamp_page_with, like_pattern, page, search_clause};

/// The page size a History table uses when the caller does not say.
///
/// Every History table shares one pair of bounds, so one screen never shows three tables that page
/// by different amounts. They are the candidate list's numbers, because History's build table *is*
/// the candidate read: the same rows, the same order, the same ceiling.
pub const DEFAULT_HISTORY_LIMIT: i64 = DEFAULT_CANDIDATE_LIMIT;
/// The largest page Rust will produce for any History table, whatever the caller asked for.
pub const MAX_HISTORY_LIMIT: i64 = MAX_CANDIDATE_LIMIT;

/// Which projects are in scope, how far to read, and what to narrow the read to.
///
/// `project_ids` is required rather than optional and an empty list means "nothing is in scope",
/// exactly as in the candidate query: a caller that forgets to name its project must not get
/// someone else's history (prompt §17).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HistoryQuery {
    pub project_ids: Vec<String>,
    pub offset: i64,
    pub limit: i64,
    /// Text to narrow the read to. LIKE syntax is escaped, so `%` and `_` match themselves.
    pub filter: Option<String>,
}

impl HistoryQuery {
    #[must_use]
    pub fn for_project(project_id: impl Into<String>) -> Self {
        Self {
            project_ids: vec![project_id.into()],
            offset: 0,
            limit: DEFAULT_HISTORY_LIMIT,
            filter: None,
        }
    }
}

/// One stored Gate run, with the verdict as it was recorded and the findings counted by state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryGateRun {
    pub run_id: String,
    pub build_id: String,
    /// File name only. Never a host path.
    pub file_name: String,
    pub baseline_build_id: Option<String>,
    /// The baseline's file name, when the run named a baseline at all.
    pub baseline_file_name: Option<String>,
    /// `PASS`, `REVIEW` or `BLOCK` — the stored disposition, never a recomputation.
    pub disposition: String,
    pub counts: GateStateCounts,
    /// `gate_runs.created_at`: when this application recorded the verdict.
    pub stored_at: String,
}

/// One published release, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryReleaseRecord {
    pub release_id: String,
    pub release_version: String,
    pub build_id: String,
    /// File name only. Never a host path.
    pub file_name: String,
    /// The run whose disposition qualified this release.
    pub gate_run_id: String,
    pub manifest_sha256: String,
    /// `release_records.created_at`. An audit time, and deliberately not part of the release id.
    pub stored_at: String,
}

/// The `FROM` a Gate run read uses. The artifact joins exist in the row query only: a count needs
/// the build, not its files.
const GATE_FROM: &str = "gate_runs g JOIN builds b ON b.id = g.build_id";
const RELEASE_FROM: &str = "release_records r JOIN builds b ON b.id = r.build_id";

/// Only a complete import is history, and only the named projects are in scope.
const SCOPE: &str = "b.state = 'COMPLETE' AND b.project_id IN";

/// The identity columns a History filter may search, for a Gate run joined to its build.
const GATE_SEARCHABLE: &[&str] = &[
    "g.id",
    "g.policy_sha256",
    "g.overall_effective_severity",
    "b.id",
    "b.snapshot_id",
];

/// The identity columns a History filter may search, for a release record joined to its build.
const RELEASE_SEARCHABLE: &[&str] = &[
    "r.id",
    "r.gate_run_id",
    "r.manifest_sha256",
    "r.release_version",
    "b.id",
    "b.snapshot_id",
];

impl Database {
    /// The builds this application stored, newest first, one bounded page at a time.
    ///
    /// This is the candidate read the Compare selectors already use — same rows, same scope rule,
    /// same reduction of a stored path to a file name. History and Compare are two views of one
    /// stored population, so they share the SELECT instead of keeping two copies of it.
    ///
    /// # Errors
    ///
    /// [`StorageError::Write`] when a read fails, or [`StorageError::Invariant`] when the scope names
    /// more projects than SQLite can bind.
    pub fn list_history_builds(
        &self,
        query: &HistoryQuery,
    ) -> Result<Page<CompareCandidate>, StorageError> {
        self.list_compare_candidates(&CandidateQuery {
            project_ids: query.project_ids.clone(),
            offset: query.offset,
            limit: query.limit,
            filter: query.filter.clone(),
        })
    }

    /// The Gate runs stored against builds in `query`'s scope, newest first, with each run's findings
    /// counted by factual state.
    ///
    /// # Errors
    ///
    /// [`StorageError::Write`] when a read fails; [`StorageError::Invariant`] for a stored
    /// disposition or finding state this build does not recognize, or for a scope SQLite cannot bind.
    pub fn list_history_gate_runs(
        &self,
        query: &HistoryQuery,
    ) -> Result<Page<HistoryGateRun>, StorageError> {
        let (offset, limit) = clamp_page_with(
            query.offset,
            query.limit,
            DEFAULT_HISTORY_LIMIT,
            MAX_HISTORY_LIMIT,
        );
        if query.project_ids.is_empty() {
            return Ok(page(Vec::new(), 0, offset, limit));
        }

        let pattern = like_pattern(query.filter.as_deref());
        let mut scope: Vec<&dyn ToSql> = scoped_values(&query.project_ids);
        let filter = search_clause(GATE_SEARCHABLE, pattern.as_ref(), &mut scope);
        let where_clause = format!(
            "{SCOPE} ({}){filter}",
            in_placeholders(query.project_ids.len())?
        );
        let total = self.count(GATE_FROM, &where_clause, &scope)?;

        let mut bound = scope;
        bound.push(&limit);
        bound.push(&offset);
        let read = {
            let mut stmt = self
                .connection()
                .prepare(&format!(
                    "SELECT g.id, g.build_id, a.path, g.baseline_build_id, ba.path,
                            g.overall_effective_severity, g.created_at
                       FROM {GATE_FROM}
                       JOIN artifacts a ON a.id = b.id || '#0'
                       LEFT JOIN artifacts ba ON ba.id = g.baseline_build_id || '#0'
                      WHERE {where_clause}
                      ORDER BY g.created_at DESC, g.id ASC
                      LIMIT ? OFFSET ?"
                ))
                .map_err(read_err)?;
            stmt.query_map(rusqlite::params_from_iter(bound.iter()), |row| {
                let baseline_path: Option<String> = row.get(4)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    display_file_name(&row.get::<_, String>(2)?),
                    row.get::<_, Option<String>>(3)?,
                    baseline_path.as_deref().map(display_file_name),
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?
        };

        let ids: Vec<String> = read.iter().map(|row| row.0.clone()).collect();
        let counts = self.finding_counts(&ids)?;
        let mut rows: Vec<HistoryGateRun> = Vec::with_capacity(read.len());
        for (
            run_id,
            build_id,
            file_name,
            baseline_build_id,
            baseline_file_name,
            severity,
            stored_at,
        ) in read
        {
            let disposition =
                EffectiveSeverity::parse(&severity).ok_or_else(|| StorageError::Invariant {
                    detail: format!("stored run `{run_id}` has the severity `{severity}`"),
                })?;
            rows.push(HistoryGateRun {
                run_id: run_id.clone(),
                build_id,
                file_name,
                baseline_build_id,
                baseline_file_name,
                disposition: disposition.as_str().to_owned(),
                counts: counts.get(&run_id).copied().unwrap_or_default(),
                stored_at,
            });
        }

        Ok(page(rows, total, offset, limit))
    }

    /// The release records stored against builds in `query`'s scope, newest first.
    ///
    /// # Errors
    ///
    /// [`StorageError::Write`] when a read fails, or [`StorageError::Invariant`] when the scope names
    /// more projects than SQLite can bind.
    pub fn list_history_releases(
        &self,
        query: &HistoryQuery,
    ) -> Result<Page<HistoryReleaseRecord>, StorageError> {
        let (offset, limit) = clamp_page_with(
            query.offset,
            query.limit,
            DEFAULT_HISTORY_LIMIT,
            MAX_HISTORY_LIMIT,
        );
        if query.project_ids.is_empty() {
            return Ok(page(Vec::new(), 0, offset, limit));
        }

        let pattern = like_pattern(query.filter.as_deref());
        let mut scope: Vec<&dyn ToSql> = scoped_values(&query.project_ids);
        let filter = search_clause(RELEASE_SEARCHABLE, pattern.as_ref(), &mut scope);
        let where_clause = format!(
            "{SCOPE} ({}){filter}",
            in_placeholders(query.project_ids.len())?
        );
        let total = self.count(RELEASE_FROM, &where_clause, &scope)?;

        let mut bound = scope;
        bound.push(&limit);
        bound.push(&offset);
        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT r.id, r.release_version, r.build_id, a.path, r.gate_run_id,
                        r.manifest_sha256, r.created_at
                   FROM {RELEASE_FROM}
                   JOIN artifacts a ON a.id = b.id || '#0'
                  WHERE {where_clause}
                  ORDER BY r.created_at DESC, r.id ASC
                  LIMIT ? OFFSET ?"
            ))
            .map_err(read_err)?;

        let rows = stmt
            .query_map(rusqlite::params_from_iter(bound.iter()), |row| {
                Ok(HistoryReleaseRecord {
                    release_id: row.get(0)?,
                    release_version: row.get(1)?,
                    build_id: row.get(2)?,
                    file_name: display_file_name(&row.get::<_, String>(3)?),
                    gate_run_id: row.get(4)?,
                    manifest_sha256: row.get(5)?,
                    stored_at: row.get(6)?,
                })
            })
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;

        Ok(page(rows, total, offset, limit))
    }

    /// How many findings each of these runs stored, counted by factual state.
    ///
    /// One grouped read for the whole page, so a page of a hundred runs costs one statement instead
    /// of a hundred. The ids come from an already bounded page, so the `IN (...)` list is bounded too.
    fn finding_counts(
        &self,
        run_ids: &[String],
    ) -> Result<BTreeMap<String, GateStateCounts>, StorageError> {
        if run_ids.is_empty() {
            return Ok(BTreeMap::new());
        }
        let placeholders = in_placeholders(run_ids.len())?;
        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT run_id, state, COUNT(*) FROM gate_findings
                  WHERE run_id IN ({placeholders})
                  GROUP BY run_id, state"
            ))
            .map_err(read_err)?;
        let grouped = stmt
            .query_map(
                rusqlite::params_from_iter(run_ids.iter().map(|id| id as &dyn ToSql)),
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;

        let mut counts: BTreeMap<String, GateStateCounts> = BTreeMap::new();
        for (run_id, state, seen) in grouped {
            let Some(parsed) = FindingState::parse(&state) else {
                return Err(StorageError::Invariant {
                    detail: format!("stored run `{run_id}` has a finding in state `{state}`"),
                });
            };
            let counted = usize::try_from(seen).unwrap_or(usize::MAX);
            let entry = counts.entry(run_id).or_default();
            match parsed {
                FindingState::Pass => entry.pass += counted,
                FindingState::Review => entry.review += counted,
                FindingState::Block => entry.block += counted,
                FindingState::Unknown => entry.unknown += counted,
                FindingState::NotApplicable => entry.not_applicable += counted,
            }
        }
        Ok(counts)
    }

    /// `SELECT COUNT(*)` over one joined `FROM` and the caller's WHERE clause.
    fn count(
        &self,
        from: &str,
        where_clause: &str,
        bound: &[&dyn ToSql],
    ) -> Result<i64, StorageError> {
        self.connection()
            .query_row(
                &format!("SELECT COUNT(*) FROM {from} WHERE {where_clause}"),
                rusqlite::params_from_iter(bound.iter()),
                |row| row.get::<_, i64>(0),
            )
            .map_err(read_err)
    }
}

/// The project ids as bound values, in the order the `IN (...)` list expects them.
fn scoped_values(project_ids: &[String]) -> Vec<&dyn ToSql> {
    project_ids.iter().map(|id| id as &dyn ToSql).collect()
}
