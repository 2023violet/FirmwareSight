//! Bounded Analyze queries over one stored snapshot.
//!
//! P1's inspect-Sections, inspect-Symbols and inspect-Evidence paths read through this module and
//! nothing else, which is where three guarantees are actually enforced:
//!
//! - **The payload is bounded.** A page defaults to [`DEFAULT_QUERY_LIMIT`] rows and cannot exceed
//!   [`MAX_QUERY_LIMIT`], whatever the caller asks for. `04_TECH/14` 3 forbids sending a whole
//!   symbol table across a boundary, and a boundary that only *offers* a limit is not a limit.
//! - **Unknown stays unknown.** A column written as NULL with a reason comes back as
//!   [`Fact::Unknown`] carrying that reason, never as a zero.
//! - **One snapshot names one build.** Every query resolves the snapshot id to a build before it
//!   reads, so no page can mix two builds' rows.
//!
//! The only SQL text assembled at runtime is an `ORDER BY` clause built from closed enums, and the
//! `FROM` target is a literal in each method. Filters travel as bound parameters.

use rusqlite::params;

use crate::Database;
use crate::error::StorageError;
use firmwaresight_core::domain::evidence::EvidenceClass;
use firmwaresight_core::domain::identity::Fact;

/// The page size a caller gets when it does not ask for one.
pub const DEFAULT_QUERY_LIMIT: i64 = 100;
/// The largest page Rust will produce. Enforced here, not in the UI.
pub const MAX_QUERY_LIMIT: i64 = 500;

/// Sort direction. `Asc` is the default so an untouched control shows the natural order of the
/// table rather than a reversed one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

impl SortDir {
    const fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SectionSort {
    /// The section header index, which is the stable locator inside one artifact.
    #[default]
    Index,
    Name,
    FileSize,
    MemorySize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SymbolSort {
    /// The storage position of the symbol within one build.
    #[default]
    Ordinal,
    Name,
    Address,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvidenceSort {
    #[default]
    Field,
    /// Evidence strength: observed, then derived, then declared, then unknown.
    Classification,
}

/// One page of rows, plus what a reader needs in order to ask for the next one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub rows: Vec<T>,
    /// The size of the whole filtered table, not of this page.
    pub total: i64,
    pub offset: i64,
    /// The limit actually applied, after clamping.
    pub limit: i64,
    /// `None` once the page has reached the end of the table.
    pub next_offset: Option<i64>,
}

/// One stored section, with every undetermined field still undetermined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionRow {
    pub index: i64,
    pub name: Fact<String>,
    pub role: String,
    pub alloc: bool,
    pub write: bool,
    pub execute: bool,
    pub virtual_address: Fact<u64>,
    pub load_address: Fact<u64>,
    /// An absent offset is `Unknown` with the reason the parser recorded. A row written before
    /// migration `0005` has no reason to report, and `bytes_or_unknown` says so in words rather than
    /// inventing one. It is never reported as `0`.
    pub file_offset: Fact<u64>,
    pub file_size: u64,
    pub memory_size: Fact<u64>,
    pub region: Fact<String>,
}

/// One stored symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRow {
    /// A row position, not an identity: one build's ordinal is stable only for that build.
    pub ordinal: i64,
    pub name: Fact<String>,
    /// The same shape as [`SectionRow::file_offset`].
    pub address: Fact<u64>,
    pub size: Fact<u64>,
    pub kind: String,
    pub binding: String,
    pub section_ref: String,
}

/// One stored evidence item: a fact and how it was obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRow {
    pub id: String,
    pub field: String,
    /// `observed`, `derived`, `declared` or `unknown`.
    pub classification: String,
    pub source_type: String,
    pub source_locator: String,
    pub raw_value: String,
    pub rule: String,
    pub confidence: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SectionQuery {
    pub snapshot_id: String,
    /// A literal substring of a section name. An unnamed section matches no filter.
    pub name_contains: Option<String>,
    pub sort_by: SectionSort,
    pub direction: SortDir,
    pub offset: i64,
    pub limit: i64,
}

impl Default for SectionQuery {
    fn default() -> Self {
        Self {
            snapshot_id: String::new(),
            name_contains: None,
            sort_by: SectionSort::default(),
            direction: SortDir::default(),
            offset: 0,
            limit: DEFAULT_QUERY_LIMIT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolQuery {
    pub snapshot_id: String,
    pub name_contains: Option<String>,
    pub sort_by: SymbolSort,
    pub direction: SortDir,
    pub offset: i64,
    pub limit: i64,
}

impl Default for SymbolQuery {
    fn default() -> Self {
        Self {
            snapshot_id: String::new(),
            name_contains: None,
            sort_by: SymbolSort::default(),
            direction: SortDir::default(),
            offset: 0,
            limit: DEFAULT_QUERY_LIMIT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceQuery {
    pub snapshot_id: String,
    pub field_contains: Option<String>,
    pub classification: Option<EvidenceClass>,
    pub sort_by: EvidenceSort,
    pub direction: SortDir,
    pub offset: i64,
    pub limit: i64,
}

impl Default for EvidenceQuery {
    fn default() -> Self {
        Self {
            snapshot_id: String::new(),
            field_contains: None,
            classification: None,
            sort_by: EvidenceSort::default(),
            direction: SortDir::default(),
            offset: 0,
            limit: DEFAULT_QUERY_LIMIT,
        }
    }
}

impl Database {
    /// Read one bounded page of sections for a snapshot.
    ///
    /// # Errors
    ///
    /// [`StorageError::NotFound`] when the snapshot was never stored; `ERR-STORAGE-4006` if the
    /// read itself fails.
    pub fn query_sections(&self, query: &SectionQuery) -> Result<Page<SectionRow>, StorageError> {
        let build_id = self.build_id_for_query(&query.snapshot_id)?;
        let (offset, limit) = clamp_page(query.offset, query.limit);
        let pattern = like_pattern(query.name_contains.as_deref());
        let filter = "build_id = ?1 AND (?2 IS NULL OR name LIKE '%' || ?2 || '%' ESCAPE '\\')";

        let order = match query.sort_by {
            SectionSort::Index => format!("section_index {}", query.direction.sql()),
            SectionSort::Name => format!(
                "name IS NULL, name {}, section_index ASC",
                query.direction.sql()
            ),
            SectionSort::FileSize => {
                format!("file_size {}, section_index ASC", query.direction.sql())
            }
            SectionSort::MemorySize => format!(
                "mem_size IS NULL, mem_size {}, section_index ASC",
                query.direction.sql()
            ),
        };

        let total = self
            .connection()
            .query_row(
                &format!("SELECT COUNT(*) FROM sections WHERE {filter}"),
                params![build_id, pattern],
                |row| row.get::<_, i64>(0),
            )
            .map_err(read_err)?;

        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT section_index, name, name_unknown, role, is_alloc, is_write, is_execute,
                        virt_addr, virt_unknown, load_addr, load_unknown, file_offset,
                        file_offset_unknown, file_size,
                        mem_size, mem_unknown, region, region_unknown
                   FROM sections
                  WHERE {filter}
                  ORDER BY {order}
                  LIMIT ?3 OFFSET ?4"
            ))
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params![build_id, pattern, limit, offset], section_row)
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;

        Ok(page(rows, total, offset, limit))
    }

    /// Read one bounded page of symbols for a snapshot.
    ///
    /// # Errors
    ///
    /// As [`Database::query_sections`].
    pub fn query_symbols(&self, query: &SymbolQuery) -> Result<Page<SymbolRow>, StorageError> {
        let build_id = self.build_id_for_query(&query.snapshot_id)?;
        let (offset, limit) = clamp_page(query.offset, query.limit);
        let pattern = like_pattern(query.name_contains.as_deref());
        let filter = "build_id = ?1 AND (?2 IS NULL OR name LIKE '%' || ?2 || '%' ESCAPE '\\')";

        // A NULL is not a value, so it cannot take part in a value ordering: unknown rows are put at
        // the tail in both directions rather than standing in for the smallest or largest one.
        let order = match query.sort_by {
            SymbolSort::Ordinal => format!("ordinal {}", query.direction.sql()),
            SymbolSort::Name => {
                format!("name IS NULL, name {}, ordinal ASC", query.direction.sql())
            }
            SymbolSort::Address => format!(
                "address IS NULL, address {}, ordinal ASC",
                query.direction.sql()
            ),
            SymbolSort::Size => {
                format!("size IS NULL, size {}, ordinal ASC", query.direction.sql())
            }
        };

        let total = self
            .connection()
            .query_row(
                &format!("SELECT COUNT(*) FROM symbols WHERE {filter}"),
                params![build_id, pattern],
                |row| row.get::<_, i64>(0),
            )
            .map_err(read_err)?;

        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT ordinal, name, name_unknown, address, address_unknown, size, size_unknown,
                        kind, binding,
                        section_ref
                   FROM symbols
                  WHERE {filter}
                  ORDER BY {order}
                  LIMIT ?3 OFFSET ?4"
            ))
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params![build_id, pattern, limit, offset], symbol_row)
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;

        Ok(page(rows, total, offset, limit))
    }

    /// Read one bounded page of evidence for a snapshot.
    ///
    /// # Errors
    ///
    /// As [`Database::query_sections`].
    pub fn query_evidence(&self, query: &EvidenceQuery) -> Result<Page<EvidenceRow>, StorageError> {
        let build_id = self.build_id_for_query(&query.snapshot_id)?;
        let (offset, limit) = clamp_page(query.offset, query.limit);
        let pattern = like_pattern(query.field_contains.as_deref());
        let class = query
            .classification
            .map(|value| class_label(value).to_owned());
        let filter = "build_id = ?1 AND (?2 IS NULL OR field LIKE '%' || ?2 || '%' ESCAPE '\\') \
                      AND (?3 IS NULL OR classification = ?3)";

        let order = match query.sort_by {
            EvidenceSort::Field => format!("field {}, id ASC", query.direction.sql()),
            EvidenceSort::Classification => format!(
                "CASE classification WHEN 'observed' THEN 0 WHEN 'derived' THEN 1 \
                  WHEN 'declared' THEN 2 ELSE 3 END {}, id ASC",
                query.direction.sql()
            ),
        };

        let total = self
            .connection()
            .query_row(
                &format!("SELECT COUNT(*) FROM evidence WHERE {filter}"),
                params![build_id, pattern, class],
                |row| row.get::<_, i64>(0),
            )
            .map_err(read_err)?;

        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT id, field, classification, source_type, source_locator, raw_value, rule,
                        confidence
                   FROM evidence
                  WHERE {filter}
                  ORDER BY {order}
                  LIMIT ?4 OFFSET ?5"
            ))
            .map_err(read_err)?;
        let rows = stmt
            .query_map(
                params![build_id, pattern, class, limit, offset],
                evidence_row,
            )
            .map_err(read_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(read_err)?;

        Ok(page(rows, total, offset, limit))
    }

    /// The build a snapshot id names, or `NotFound`.
    fn build_id_for_query(&self, snapshot_id: &str) -> Result<String, StorageError> {
        self.build_id_for_snapshot(snapshot_id)?
            .ok_or_else(|| StorageError::NotFound {
                id: snapshot_id.to_owned(),
            })
    }
}

/// Put the caller's numbers inside the boundary instead of rejecting them: a page that asked for
/// more than Rust will produce still gets a page, and the reported `limit` is the one that was used.
/// A limit that is not a page size at all - zero or negative - is treated as "not asked", which is
/// the default, rather than as a request for one row or for none.
fn clamp_page(offset: i64, limit: i64) -> (i64, i64) {
    clamp_page_with(offset, limit, DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT)
}

/// The same clamping rule with a different pair of bounds, so a bounded read that is not a detail
/// page (the candidate list, for instance) cannot reuse the detail ceiling by accident.
#[must_use]
pub(crate) fn clamp_page_with(offset: i64, limit: i64, default: i64, max: i64) -> (i64, i64) {
    let offset = offset.max(0);
    let limit = if limit <= 0 { default } else { limit.min(max) };
    (offset, limit)
}

/// `%` and `_` are LIKE syntax, not text. A filter is what the user typed, so both are escaped and
/// an empty filter is treated as no filter.
fn like_pattern(filter: Option<&str>) -> Option<String> {
    let text = filter?;
    if text.is_empty() {
        return None;
    }
    let mut escaped = String::with_capacity(text.len() + 8);
    for character in text.chars() {
        if matches!(character, '%' | '_' | '\\') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    Some(escaped)
}

pub(crate) fn page<T>(rows: Vec<T>, total: i64, offset: i64, limit: i64) -> Page<T> {
    let read = offset + rows.len() as i64;
    Page {
        rows,
        total,
        offset,
        limit,
        next_offset: if read < total { Some(read) } else { None },
    }
}

const fn class_label(class: EvidenceClass) -> &'static str {
    match class {
        EvidenceClass::Observed => "observed",
        EvidenceClass::Derived => "derived",
        EvidenceClass::Declared => "declared",
        EvidenceClass::Unknown => "unknown",
    }
}

fn section_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SectionRow> {
    Ok(SectionRow {
        index: row.get(0)?,
        name: known_or_unknown(row.get(1)?, row.get(2)?),
        role: row.get(3)?,
        alloc: row.get::<_, i64>(4)? == 1,
        write: row.get::<_, i64>(5)? == 1,
        execute: row.get::<_, i64>(6)? == 1,
        virtual_address: bytes_or_unknown(row.get(7)?, row.get(8)?),
        load_address: bytes_or_unknown(row.get(9)?, row.get(10)?),
        file_offset: bytes_or_unknown(row.get(11)?, row.get(12)?),
        file_size: row.get::<_, i64>(13)? as u64,
        memory_size: bytes_or_unknown(row.get(14)?, row.get(15)?),
        region: known_or_unknown(row.get(16)?, row.get(17)?),
    })
}

fn symbol_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SymbolRow> {
    Ok(SymbolRow {
        ordinal: row.get(0)?,
        name: known_or_unknown(row.get(1)?, row.get(2)?),
        address: bytes_or_unknown(row.get(3)?, row.get(4)?),
        size: bytes_or_unknown(row.get(5)?, row.get(6)?),
        kind: row.get(7)?,
        binding: row.get(8)?,
        section_ref: row.get(9)?,
    })
}

fn evidence_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<EvidenceRow> {
    Ok(EvidenceRow {
        id: row.get(0)?,
        field: row.get(1)?,
        classification: row.get(2)?,
        source_type: row.get(3)?,
        source_locator: row.get(4)?,
        raw_value: row.get(5)?,
        rule: row.get(6)?,
        confidence: row.get(7)?,
    })
}

/// A NULL with a recorded reason is `Unknown`; a NULL without one is still `Unknown`, because the
/// alternative is a zero the artifact never said.
pub(crate) fn known_or_unknown(value: Option<String>, reason: Option<String>) -> Fact<String> {
    match value {
        Some(value) => Fact::Known(value),
        None => Fact::Unknown {
            reason: reason.unwrap_or_else(|| "no reason was recorded".to_owned()),
        },
    }
}

pub(crate) fn bytes_or_unknown(value: Option<i64>, reason: Option<String>) -> Fact<u64> {
    match value {
        Some(value) => Fact::Known(value as u64),
        None => Fact::Unknown {
            reason: reason.unwrap_or_else(|| "no reason was recorded".to_owned()),
        },
    }
}

fn read_err(source: rusqlite::Error) -> StorageError {
    StorageError::Write {
        detail: format!("read: {source}"),
    }
}
