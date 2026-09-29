//! Connection setup, migrations and the snapshot import path.

use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, params};

use crate::error::StorageError;
use firmwaresight_core::domain::build_snapshot::{BuildSnapshot, NORMALIZATION_VERSION};
use firmwaresight_core::domain::identity::Fact;
use firmwaresight_core::domain::memory::ByteTotal;

/// The schema version this build writes.
pub const SCHEMA_VERSION: i64 = 2;

const MIGRATION_0001: &str = include_str!("../migrations/0001_initial.sql");
const MIGRATION_0002: &str = include_str!("../migrations/0002_evidence_keyed_by_build.sql");

/// Applied in version order, each in its own transaction, so a failed upgrade leaves the previous
/// schema and every row in it exactly as they were.
const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "0001_initial", MIGRATION_0001),
    (2, "0002_evidence_keyed_by_build", MIGRATION_0002),
];

/// Everything the P0 round-trip test compares against the in-memory snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSummary {
    pub build_id: String,
    pub snapshot_id: String,
    pub state: String,
    pub normalization_version: String,
    pub sha256: String,
    pub byte_size: u64,
    pub architecture: String,
    pub bitness: String,
    pub endianness: String,
    pub section_count: i64,
    pub symbol_count: i64,
    pub evidence_count: i64,
    pub nonvolatile_state: String,
    pub nonvolatile_bytes: Option<i64>,
    pub runtime_state: String,
    pub runtime_bytes: Option<i64>,
    pub admissible_hard_block: bool,
    pub layout_source: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(path).map_err(|source| StorageError::Open {
            path: path.display().to_string(),
            source,
        })?;
        Self::establish(conn).map_err(|err| match err {
            StorageError::Open { .. } => err,
            other => other,
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory().map_err(|source| StorageError::Open {
            path: ":memory:".to_owned(),
            source,
        })?;
        Self::establish(conn)
    }

    /// Apply the baseline connection settings, then migrate.
    fn establish(conn: Connection) -> Result<Self, StorageError> {
        // `foreign_keys` is per-connection in SQLite, so it is set on every open rather than
        // once at creation time.
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|err| StorageError::Configure {
                detail: format!("foreign_keys: {err}"),
            })?;
        conn.busy_timeout(Duration::from_secs(5))
            .map_err(|err| StorageError::Configure {
                detail: format!("busy_timeout: {err}"),
            })?;

        // WAL is attempted, not assumed: it is unavailable on some filesystems and modes, and a
        // journal-mode fallback is a performance property, not a correctness one.
        let journal_mode = conn
            .pragma_update(None, "journal_mode", "WAL")
            .and_then(|_| conn.query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0)));
        match journal_mode {
            Ok(mode) if mode.eq_ignore_ascii_case("wal") => {}
            Ok(mode) => {
                tracing::warn!(
                    journal_mode = mode,
                    "write-ahead logging unavailable here; continuing with the journal mode \
                     SQLite selected"
                );
            }
            Err(err) => {
                tracing::warn!(error = %err, "could not confirm journal mode; continuing");
            }
        }

        let mut db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Create the schema bookkeeping and apply any unapplied migrations, in order.
    pub fn migrate(&mut self) -> Result<(), StorageError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                     version    INTEGER PRIMARY KEY NOT NULL,
                     name       TEXT NOT NULL,
                     applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
                 );",
            )
            .map_err(|source| StorageError::Migration { version: 0, source })?;

        let current = self.current_version()?;

        if current > SCHEMA_VERSION {
            // Never delete and recreate: that is how history gets destroyed by an older binary.
            return Err(StorageError::UnsupportedSchemaVersion {
                found: current,
                supported: SCHEMA_VERSION,
            });
        }

        for (version, name, sql) in MIGRATIONS {
            if current >= *version {
                continue;
            }
            let tx = self
                .conn
                .transaction()
                .map_err(|source| StorageError::Migration {
                    version: *version,
                    source,
                })?;
            tx.execute_batch(sql)
                .map_err(|source| StorageError::Migration {
                    version: *version,
                    source,
                })?;
            tx.execute(
                "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
                params![*version, *name],
            )
            .map_err(|source| StorageError::Migration {
                version: *version,
                source,
            })?;
            tx.commit().map_err(|source| StorageError::Migration {
                version: *version,
                source,
            })?;
        }

        Ok(())
    }

    fn current_version(&self) -> Result<i64, StorageError> {
        let version: Option<i64> = self
            .conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap_or(None);
        Ok(version.unwrap_or(0))
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Persist a snapshot as one atomic import.
    ///
    /// The build row starts at IMPORTING and is only flipped to COMPLETE inside the same
    /// transaction that writes its contents, so a rolled-back import cannot be observed as a
    /// finished build.
    pub fn import_snapshot(
        &mut self,
        project_id: &str,
        project_name: &str,
        snapshot: &BuildSnapshot,
    ) -> Result<String, StorageError> {
        let build_id = format!("build-{}", snapshot.id().as_str());

        let tx = self
            .conn
            .transaction()
            .map_err(|source| StorageError::Write {
                detail: format!("begin: {source}"),
            })?;

        tx.execute(
            "INSERT OR IGNORE INTO projects (id, name) VALUES (?1, ?2)",
            params![project_id, project_name],
        )
        .map_err(write_err)?;

        tx.execute(
            "INSERT INTO builds
                 (id, project_id, snapshot_id, normalization_version, created_by_fwsight, state)
             VALUES (?1, ?2, ?3, ?4, ?5, 'IMPORTING')",
            params![
                build_id,
                project_id,
                snapshot.id().as_str(),
                NORMALIZATION_VERSION,
                snapshot.created_by_fwsight_version(),
            ],
        )
        .map_err(write_err)?;

        write_artifacts(&tx, &build_id, snapshot)?;
        write_sections(&tx, &build_id, snapshot)?;
        write_symbols(&tx, &build_id, snapshot)?;
        write_evidence(&tx, &build_id, snapshot)?;
        write_memory(&tx, &build_id, snapshot)?;

        // Last write inside the transaction: nothing can observe COMPLETE with partial content.
        tx.execute(
            "UPDATE builds SET state = 'COMPLETE' WHERE id = ?1",
            params![build_id],
        )
        .map_err(write_err)?;

        tx.commit().map_err(|source| StorageError::Write {
            detail: format!("commit: {source}"),
        })?;

        Ok(build_id)
    }

    pub fn summary(&self, build_id: &str) -> Result<BuildSummary, StorageError> {
        // The primary artifact is index 0 of the build, the row `write_artifacts` stores it under.
        // Naming it is what keeps this deterministic now that a build can carry a companion MAP
        // artifact: the bare `JOIN artifacts a ON a.build_id = b.id` returned whichever row SQLite
        // happened to visit first, and `query_row` keeps only that one.
        let primary_artifact_id = artifact_row_id(build_id, 0);

        self.conn
            .query_row(
                "SELECT b.id, b.snapshot_id, b.state, b.normalization_version,
                        a.sha256, a.byte_size, a.architecture, a.bitness, a.endianness,
                        (SELECT COUNT(*) FROM sections s WHERE s.build_id = b.id),
                        (SELECT COUNT(*) FROM symbols y WHERE y.build_id = b.id),
                        (SELECT COUNT(*) FROM evidence e WHERE e.build_id = b.id),
                        m.nonvolatile_state, m.nonvolatile_bytes,
                        m.runtime_state, m.runtime_bytes,
                        m.admissible_hard_block, m.layout_source
                   FROM builds b
                   JOIN artifacts a ON a.build_id = b.id AND a.id = ?2
                   JOIN memory_footprints m ON m.build_id = b.id
                  WHERE b.id = ?1",
                params![build_id, primary_artifact_id],
                |row| {
                    Ok(BuildSummary {
                        build_id: row.get(0)?,
                        snapshot_id: row.get(1)?,
                        state: row.get(2)?,
                        normalization_version: row.get(3)?,
                        sha256: row.get(4)?,
                        byte_size: row.get::<_, i64>(5)? as u64,
                        architecture: row.get(6)?,
                        bitness: row.get(7)?,
                        endianness: row.get(8)?,
                        section_count: row.get(9)?,
                        symbol_count: row.get(10)?,
                        evidence_count: row.get(11)?,
                        nonvolatile_state: row.get(12)?,
                        nonvolatile_bytes: row.get(13)?,
                        runtime_state: row.get(14)?,
                        runtime_bytes: row.get(15)?,
                        admissible_hard_block: row.get::<_, i64>(16)? == 1,
                        layout_source: row.get(17)?,
                    })
                },
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => StorageError::NotFound {
                    id: build_id.to_owned(),
                },
                other => StorageError::Write {
                    detail: format!("read summary: {other}"),
                },
            })
    }

    /// The build already stored for a content-addressed snapshot, if any.
    ///
    /// A snapshot id is derived from artifact bytes, so finding one here means the identical
    /// import is already in history. Callers use this to avoid writing the same build twice.
    pub fn build_id_for_snapshot(&self, snapshot_id: &str) -> Result<Option<String>, StorageError> {
        let found: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM builds WHERE snapshot_id = ?1 LIMIT 1",
                params![snapshot_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(write_err)?;
        Ok(found)
    }
}

fn write_err(source: rusqlite::Error) -> StorageError {
    StorageError::Write {
        detail: source.to_string(),
    }
}

/// The artifact row for one position of one build.
///
/// The writer and the primary-artifact reader both derive the id from the build id, so they cannot
/// disagree about which row is index 0.
fn artifact_row_id(build_id: &str, index: usize) -> String {
    format!("{build_id}#{index}")
}

fn write_artifacts(
    tx: &rusqlite::Transaction<'_>,
    build_id: &str,
    snapshot: &BuildSnapshot,
) -> Result<(), StorageError> {
    for (index, artifact) in snapshot.artifacts().iter().enumerate() {
        let (entry, entry_unknown) = match &artifact.entry_point {
            Fact::Known(value) => (Some(*value as i64), None),
            Fact::Unknown { reason } => (None, Some(reason.clone())),
        };
        let (note, note_unknown) = match &artifact.build_id {
            Fact::Known(value) => (Some(value.clone()), None),
            Fact::Unknown { reason } => (None, Some(reason.clone())),
        };
        tx.execute(
            "INSERT INTO artifacts
                 (id, build_id, path, kind, sha256, byte_size, parser_id, architecture,
                  bitness, endianness, entry_point, entry_unknown, build_id_note,
                  build_id_unknown)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                artifact_row_id(build_id, index),
                build_id,
                artifact.path,
                format!("{:?}", artifact.kind),
                artifact.sha256.hex(),
                artifact.byte_size as i64,
                artifact.parser_id.0,
                format!("{:?}", artifact.architecture),
                format!("{:?}", artifact.bitness),
                format!("{:?}", artifact.endianness),
                entry,
                entry_unknown,
                note,
                note_unknown,
            ],
        )
        .map_err(write_err)?;
    }
    Ok(())
}

fn write_sections(
    tx: &rusqlite::Transaction<'_>,
    build_id: &str,
    snapshot: &BuildSnapshot,
) -> Result<(), StorageError> {
    let mut stmt = tx
        .prepare(
            "INSERT INTO sections
                 (build_id, section_index, name, name_unknown, role, is_alloc, is_write,
                  is_execute, virt_addr, virt_unknown, load_addr, load_unknown, file_offset,
                  file_size, mem_size, mem_unknown, region, region_unknown)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        )
        .map_err(write_err)?;

    for section in snapshot.sections() {
        let (virt, virt_unknown) = optional_fact(&section.virtual_address);
        let (load, load_unknown) = optional_fact(&section.load_address);
        let (mem, mem_unknown) = optional_fact(&section.memory_size);
        let (name, name_unknown) = optional_string(&section.name);
        let (region, region_unknown) = optional_string(&section.region);
        let file_offset = optional_fact_u64(&section.file_offset);

        stmt.execute(params![
            build_id,
            section.index as i64,
            name,
            name_unknown,
            format!("{:?}", section.role),
            section.flags.alloc as i64,
            section.flags.write as i64,
            section.flags.execute as i64,
            virt,
            virt_unknown,
            load,
            load_unknown,
            file_offset,
            section.file_size as i64,
            mem,
            mem_unknown,
            region,
            region_unknown,
        ])
        .map_err(write_err)?;
    }
    Ok(())
}

fn write_symbols(
    tx: &rusqlite::Transaction<'_>,
    build_id: &str,
    snapshot: &BuildSnapshot,
) -> Result<(), StorageError> {
    let mut stmt = tx
        .prepare(
            "INSERT INTO symbols
                 (build_id, ordinal, name, name_unknown, address, size, size_unknown, kind,
                  binding, section_ref)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        )
        .map_err(write_err)?;

    for (ordinal, symbol) in snapshot.symbols().iter().enumerate() {
        let (name, name_unknown) = optional_string(&symbol.name);
        let address = optional_fact_u64(&symbol.address);
        let (size, size_unknown) = optional_fact(&symbol.size);
        stmt.execute(params![
            build_id,
            ordinal as i64,
            name,
            name_unknown,
            address,
            size,
            size_unknown,
            format!("{:?}", symbol.kind),
            format!("{:?}", symbol.binding),
            format!("{:?}", symbol.section),
        ])
        .map_err(write_err)?;
    }
    Ok(())
}

fn write_evidence(
    tx: &rusqlite::Transaction<'_>,
    build_id: &str,
    snapshot: &BuildSnapshot,
) -> Result<(), StorageError> {
    let mut stmt = tx
        .prepare(
            "INSERT INTO evidence
                 (id, build_id, field, classification, source_type, source_locator, raw_value,
                  rule, confidence)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        )
        .map_err(write_err)?;

    for item in snapshot.evidence() {
        stmt.execute(params![
            item.id,
            build_id,
            item.field,
            db_class(item.classification),
            format!("{:?}", item.source_type),
            item.source_locator,
            item.raw_value,
            item.rule,
            item.confidence.map(|c| format!("{c:?}")),
        ])
        .map_err(write_err)?;
    }
    Ok(())
}

fn write_memory(
    tx: &rusqlite::Transaction<'_>,
    build_id: &str,
    snapshot: &BuildSnapshot,
) -> Result<(), StorageError> {
    let memory = snapshot.memory().ok_or_else(|| StorageError::Invariant {
        detail: "a P0 snapshot must carry a memory footprint".to_owned(),
    })?;

    let (nv_state, nv_bytes, nv_class) = budget_parts(&memory.nonvolatile);
    let (rt_state, rt_bytes, rt_class) = budget_parts(&memory.runtime_ram);

    tx.execute(
        "INSERT INTO memory_footprints
             (build_id, layout_source, weakest_basis, admissible_hard_block,
              nonvolatile_state, nonvolatile_bytes, nonvolatile_class,
              runtime_state, runtime_bytes, runtime_class, excluded_metadata)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            build_id,
            db_layout(memory.layout_source),
            memory.weakest_basis.map(|b| format!("{b:?}")),
            memory.admissible_for_hard_block() as i64,
            nv_state,
            nv_bytes,
            nv_class,
            rt_state,
            rt_bytes,
            rt_class,
            memory.excluded_metadata_bytes as i64,
        ],
    )
    .map_err(write_err)?;
    Ok(())
}

fn budget_parts(total: &ByteTotal) -> (&'static str, Option<i64>, &'static str) {
    let class = match total {
        ByteTotal::Exact { .. } => "observed",
        ByteTotal::Partial { .. } => "derived",
        ByteTotal::Unknown { .. } => "unknown",
    };
    match total {
        ByteTotal::Exact { bytes } => ("exact", Some(*bytes as i64), class),
        ByteTotal::Partial { bytes, .. } => ("partial", Some(*bytes as i64), class),
        ByteTotal::Unknown { .. } => ("unknown", None, class),
    }
}

const fn db_layout(source: firmwaresight_core::domain::memory::LayoutSource) -> &'static str {
    match source {
        firmwaresight_core::domain::memory::LayoutSource::ProjectConfig => "project-config",
        firmwaresight_core::domain::memory::LayoutSource::MapMemoryConfiguration => "map",
        firmwaresight_core::domain::memory::LayoutSource::None => "none",
    }
}

const fn db_class(class: firmwaresight_core::domain::evidence::EvidenceClass) -> &'static str {
    use firmwaresight_core::domain::evidence::EvidenceClass as E;
    match class {
        E::Observed => "observed",
        E::Derived => "derived",
        E::Declared => "declared",
        E::Unknown => "unknown",
    }
}

fn optional_fact(fact: &Fact<u64>) -> (Option<i64>, Option<String>) {
    match fact {
        Fact::Known(value) => (Some(*value as i64), None),
        Fact::Unknown { reason } => (None, Some(reason.clone())),
    }
}

fn optional_fact_u64(fact: &Fact<u64>) -> Option<i64> {
    fact.value().copied().map(|value| value as i64)
}

fn optional_string(fact: &Fact<String>) -> (Option<String>, Option<String>) {
    match fact {
        Fact::Known(value) => (Some(value.clone()), None),
        Fact::Unknown { reason } => (None, Some(reason.clone())),
    }
}
