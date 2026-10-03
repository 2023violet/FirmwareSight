//! Read-only storage support for Compare.
//!
//! P2 adds no table and no migration: every fact a build-to-build diff needs is already stored by
//! Analyze (`04_TECH/15` §3 and §4). This module therefore has two jobs and no others —
//!
//! 1. list the persisted builds a caller may pick from, bounded and scoped to the projects the
//!    caller names, so the P0 demo build never appears as someone's history;
//! 2. hydrate [`DiffSnapshotInput`] for one snapshot out of SQLite, so `firmwaresight-core` decides
//!    the diff and storage keeps no diff semantics of its own.
//!
//! Three rules carried over from the P1 query layer and enforced here rather than assumed:
//!
//! - **Nothing is re-read from disk.** A stored snapshot stays comparable after the original ELF
//!   and MAP have moved or been deleted, so both paths read rows only and never open an artifact.
//! - **Unknown stays unknown.** `nonvolatile_state` and `runtime_state` decide a budget's class,
//!   and a NULL byte count stays a NULL. A row that stores a fact as unknown comes back unknown.
//! - **No host path leaves this crate.** A stored artifact `path` is reduced to its file name
//!   before it reaches a candidate row or a diff input, per `AGENTS.md` 7.
//!
//! SQL assembled at runtime varies only in the *number* of `?` placeholders, one per project id.
//! Project ids, limits and offsets travel as bound parameters.

use rusqlite::{OptionalExtension, params};

use crate::Database;
use crate::error::StorageError;
use crate::query::{
    Page, bytes_or_unknown, clamp_page_with, known_or_unknown, like_pattern, page, search_clause,
};
use firmwaresight_core::domain::diff::{
    BudgetState, DiffArtifact, DiffBudget, DiffMemory, DiffSection, DiffSnapshotInput, DiffSymbol,
    SideEvidence, display_file_name,
};

/// How many candidates a selector asks for when it does not say.
pub const DEFAULT_CANDIDATE_LIMIT: i64 = 25;
/// The largest candidate page Rust will produce, whatever the caller asked for.
pub const MAX_CANDIDATE_LIMIT: i64 = 100;

/// Which builds may be listed, and how many.
///
/// `project_ids` is required rather than optional: an empty list means "nothing is in scope", not
/// "everything is". A caller that forgets to name its project gets an empty page instead of the P0
/// demo build.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CandidateQuery {
    pub project_ids: Vec<String>,
    pub offset: i64,
    pub limit: i64,
    /// Text to narrow the read to, over the identity columns this crate stores. LIKE syntax is
    /// escaped, so `%` and `_` typed by a person match themselves. A stored path is deliberately
    /// not searchable: see `history`.
    pub filter: Option<String>,
}

impl CandidateQuery {
    #[must_use]
    pub fn for_project(project_id: impl Into<String>) -> Self {
        Self {
            project_ids: vec![project_id.into()],
            offset: 0,
            limit: DEFAULT_CANDIDATE_LIMIT,
            filter: None,
        }
    }
}

/// The identity columns a candidate filter may search.
const CANDIDATE_SEARCHABLE: &[&str] = &["b.id", "b.snapshot_id", "a.sha256", "a.architecture"];

/// `?, ?, ...` for one `IN (...)` list, refusing a list SQLite cannot bind rather than truncating it.
///
/// # Errors
///
/// [`StorageError::Invariant`] when `count` exceeds [`SQLITE_MAX_VARIABLES`].
pub(crate) fn in_placeholders(count: usize) -> Result<String, StorageError> {
    if count > SQLITE_MAX_VARIABLES {
        return Err(StorageError::Invariant {
            detail: format!(
                "a bounded read may name at most {SQLITE_MAX_VARIABLES} values at once, not {count}"
            ),
        });
    }
    Ok(vec!["?"; count].join(","))
}

/// A persisted budget: its recorded state and, when one was recorded, its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredBudget {
    pub state: String,
    pub bytes: Option<u64>,
}

impl StoredBudget {
    pub(crate) fn from_parts(state: String, bytes: Option<i64>) -> Self {
        Self {
            state,
            bytes: bytes.map(|value| value as u64),
        }
    }
}

/// One selectable, already-persisted build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareCandidate {
    pub build_id: String,
    pub snapshot_id: String,
    /// File name only. Never a host path.
    pub file_name: String,
    pub sha256: String,
    pub byte_size: u64,
    pub architecture: String,
    /// `builds.created_at`, which is when FirmwareSight imported this build. It is not a build
    /// time and must never be labelled as one (`04_TECH/24` Time).
    pub imported_at: String,
    pub nonvolatile: StoredBudget,
    pub runtime_ram: StoredBudget,
}

/// The footprint row a diff input needs, read once per side.
#[derive(Debug)]
struct FootprintRow {
    layout_source: String,
    weakest_basis: Option<String>,
    nonvolatile: StoredBudget,
    runtime_ram: StoredBudget,
    excluded_metadata: u64,
}

/// The primary artifact row a diff input needs.
#[derive(Debug)]
struct PrimaryArtifactRow {
    path: String,
    sha256: String,
    byte_size: u64,
}

impl Database {
    /// List the persisted builds a caller may compare, newest import first.
    ///
    /// Only a build whose import transaction completed is offered: a half-written import is not a
    /// snapshot, and `04_TECH/15` §6 says it must never be presented as one.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Write`] when either read fails.
    ///
    /// # Panics
    ///
    /// Never in a supported path. A project-id list long enough to exceed SQLite's variable limit
    /// is refused as a typed error rather than truncated.
    pub fn list_compare_candidates(
        &self,
        query: &CandidateQuery,
    ) -> Result<Page<CompareCandidate>, StorageError> {
        let (offset, limit) = clamp_page_with(
            query.offset,
            query.limit,
            DEFAULT_CANDIDATE_LIMIT,
            MAX_CANDIDATE_LIMIT,
        );

        if query.project_ids.is_empty() {
            return Ok(page(Vec::new(), 0, offset, limit));
        }
        let placeholders = in_placeholders(query.project_ids.len())?;
        let pattern = like_pattern(query.filter.as_deref());

        let mut scope: Vec<&dyn rusqlite::types::ToSql> = query
            .project_ids
            .iter()
            .map(|id| id as &dyn rusqlite::types::ToSql)
            .collect();
        let filter = search_clause(CANDIDATE_SEARCHABLE, pattern.as_ref(), &mut scope);
        let scoped = format!("b.state = 'COMPLETE' AND b.project_id IN ({placeholders}){filter}");

        let total = self
            .connection()
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM builds b
                       JOIN artifacts a ON a.id = b.id || '#0'
                       JOIN memory_footprints m ON m.build_id = b.id
                      WHERE {scoped}"
                ),
                rusqlite::params_from_iter(scope.iter()),
                |row| row.get::<_, i64>(0),
            )
            .map_err(read_err)?;

        let mut bound = scope;
        bound.push(&limit);
        bound.push(&offset);

        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT b.id, b.snapshot_id, a.path, a.sha256, a.byte_size, a.architecture,
                        b.created_at, m.nonvolatile_state, m.nonvolatile_bytes,
                        m.runtime_state, m.runtime_bytes
                   FROM builds b
                   JOIN artifacts a ON a.id = b.id || '#0'
                   JOIN memory_footprints m ON m.build_id = b.id
                  WHERE {scoped}
                  ORDER BY b.created_at DESC, b.snapshot_id ASC
                  LIMIT ? OFFSET ?"
            ))
            .map_err(read_err)?;

        let rows = stmt
            .query_map(rusqlite::params_from_iter(bound.iter()), |row| {
                Ok(CompareCandidate {
                    build_id: row.get(0)?,
                    snapshot_id: row.get(1)?,
                    file_name: display_file_name(&row.get::<_, String>(2)?),
                    sha256: row.get(3)?,
                    byte_size: row.get::<_, i64>(4)? as u64,
                    architecture: row.get(5)?,
                    imported_at: row.get(6)?,
                    nonvolatile: StoredBudget::from_parts(row.get(7)?, row.get(8)?),
                    runtime_ram: StoredBudget::from_parts(row.get(9)?, row.get(10)?),
                })
            })
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;

        Ok(page(rows, total, offset, limit))
    }

    /// Hydrate one persisted snapshot into the input the Core diff consumes.
    ///
    /// Every row set is read for the build this snapshot resolves to and ordered by its own key, so
    /// the hydrated input does not depend on SQLite's visit order. The artifact file is never
    /// opened: the stored bytes are the whole basis for the comparison.
    ///
    /// # Errors
    ///
    /// - [`StorageError::NotFound`] when no build carries this snapshot id;
    /// - [`StorageError::Write`] when a read fails;
    /// - [`StorageError::Invariant`] when a build has no primary artifact row or no footprint row,
    ///   which no supported application path can produce.
    pub fn load_diff_input(&self, snapshot_id: &str) -> Result<DiffSnapshotInput, StorageError> {
        let build_id =
            self.build_id_for_snapshot(snapshot_id)?
                .ok_or_else(|| StorageError::NotFound {
                    id: snapshot_id.to_owned(),
                })?;

        let artifact = self
            .connection()
            .query_row(
                "SELECT path, sha256, byte_size FROM artifacts WHERE id = ?1",
                params![crate::db::artifact_row_id(&build_id, 0)],
                |row| {
                    Ok(PrimaryArtifactRow {
                        path: row.get(0)?,
                        sha256: row.get(1)?,
                        byte_size: row.get::<_, i64>(2)? as u64,
                    })
                },
            )
            .optional()
            .map_err(read_err)?
            .ok_or_else(|| StorageError::Invariant {
                detail: format!("build {build_id} has no primary artifact row"),
            })?;

        let map_backed: bool = self
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM artifacts WHERE build_id = ?1 AND kind = 'Map'",
                params![build_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(read_err)?
            > 0;

        let footprint = self
            .connection()
            .query_row(
                "SELECT layout_source, weakest_basis, nonvolatile_state, nonvolatile_bytes,
                        runtime_state, runtime_bytes, excluded_metadata
                   FROM memory_footprints WHERE build_id = ?1",
                params![build_id],
                |row| {
                    Ok(FootprintRow {
                        layout_source: row.get(0)?,
                        weakest_basis: row.get(1)?,
                        nonvolatile: StoredBudget::from_parts(row.get(2)?, row.get(3)?),
                        runtime_ram: StoredBudget::from_parts(row.get(4)?, row.get(5)?),
                        excluded_metadata: row.get::<_, i64>(6)? as u64,
                    })
                },
            )
            .optional()
            .map_err(read_err)?
            .ok_or_else(|| StorageError::Invariant {
                detail: format!("build {build_id} has no stored footprint row"),
            })?;

        Ok(DiffSnapshotInput {
            snapshot_id: snapshot_id.to_owned(),
            artifact: DiffArtifact {
                file_name: display_file_name(&artifact.path),
                sha256: artifact.sha256,
                byte_size: artifact.byte_size,
            },
            memory: Some(DiffMemory {
                nonvolatile: budget(&footprint.nonvolatile),
                runtime_ram: budget(&footprint.runtime_ram),
                excluded_metadata_bytes: footprint.excluded_metadata,
                evidence: SideEvidence {
                    map_backed,
                    layout_source: footprint.layout_source,
                    weakest_basis: footprint.weakest_basis,
                },
            }),
            sections: self.read_diff_sections(&build_id)?,
            symbols: self.read_diff_symbols(&build_id)?,
        })
    }

    fn read_diff_sections(&self, build_id: &str) -> Result<Vec<DiffSection>, StorageError> {
        let mut stmt = self
            .connection()
            .prepare(
                "SELECT section_index, name, name_unknown, role, is_alloc, is_write, is_execute,
                        virt_addr, virt_unknown, load_addr, load_unknown, file_offset, file_size,
                        mem_size, mem_unknown, region, region_unknown
                   FROM sections WHERE build_id = ?1 ORDER BY section_index ASC",
            )
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params![build_id], |row| {
                let name: Option<String> = row.get(1)?;
                let name_reason: Option<String> = row.get(2)?;
                Ok(DiffSection {
                    index: row.get(0)?,
                    name: known_or_unknown(name, name_reason),
                    role: row.get(3)?,
                    alloc: row.get::<_, i64>(4)? == 1,
                    write: row.get::<_, i64>(5)? == 1,
                    execute: row.get::<_, i64>(6)? == 1,
                    virtual_address: bytes_or_unknown(row.get(7)?, row.get(8)?),
                    load_address: bytes_or_unknown(row.get(9)?, row.get(10)?),
                    // No file_offset_unknown column exists, so an absent offset can only be
                    // reported as absent. It is never reported as 0.
                    file_offset: row.get::<_, Option<i64>>(11)?.map(|value| value as u64),
                    file_size: row.get::<_, i64>(12)? as u64,
                    memory_size: bytes_or_unknown(row.get(13)?, row.get(14)?),
                    region: known_or_unknown(row.get(15)?, row.get(16)?),
                })
            })
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;
        Ok(rows)
    }

    fn read_diff_symbols(&self, build_id: &str) -> Result<Vec<DiffSymbol>, StorageError> {
        let mut stmt = self
            .connection()
            .prepare(
                "SELECT ordinal, name, name_unknown, address, size, size_unknown, kind, binding,
                        section_ref
                   FROM symbols WHERE build_id = ?1 ORDER BY ordinal ASC",
            )
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params![build_id], |row| {
                let name: Option<String> = row.get(1)?;
                let name_reason: Option<String> = row.get(2)?;
                Ok(DiffSymbol {
                    ordinal: row.get(0)?,
                    name: known_or_unknown(name, name_reason),
                    // No address_unknown column either, so an unknown address stays an Option.
                    address: row.get::<_, Option<i64>>(3)?.map(|value| value as u64),
                    size: bytes_or_unknown(row.get(4)?, row.get(5)?),
                    kind: row.get(6)?,
                    binding: row.get(7)?,
                    section_ref: row.get(8)?,
                })
            })
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;
        Ok(rows)
    }
}

/// SQLite's compile-time variable limit on the pinned `rusqlite + bundled` build.
const SQLITE_MAX_VARIABLES: usize = 999;

/// Map a stored budget back onto the diff's three-state model. A state string this build does not
/// recognize is treated as unknown rather than promoted to exact.
fn budget(stored: &StoredBudget) -> DiffBudget {
    let state = BudgetState::parse(&stored.state);
    DiffBudget {
        state,
        bytes: match state {
            BudgetState::Unknown => None,
            _ => stored.bytes,
        },
    }
}

pub(crate) fn read_err(source: rusqlite::Error) -> StorageError {
    StorageError::Write {
        detail: format!("read: {source}"),
    }
}
