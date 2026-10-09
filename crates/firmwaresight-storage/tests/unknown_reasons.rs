//! The two nullable numeric columns that lost their Unknown reason on the way in (L6, L7).
//!
//! `0001_initial.sql:42` states the rule the schema has followed everywhere except here: *"A fact we
//! could not determine is recorded as a reason, never silently as zero."* Five of the seven nullable
//! numeric columns pair their value with a `*_unknown` text column; `sections.file_offset` (`:65`) and
//! `symbols.address` (`:79`) do not, and `optional_fact_u64` (`db.rs:567-569`) discards the reason the
//! caller handed it. The decision and its limits are in
//! `P5_VALIDATION/P5_MIGRATION_DECISION.md`; these tests are the evidence that the loss is real, and
//! then the proof it stopped.
//!
//! One asymmetry is recorded rather than smoothed over: the ELF parser really does emit an Unknown
//! file offset (`elf.rs:240-244`, a `SHT_NOBITS` section such as `.bss`), so the section case is proven
//! from a committed fixture. It never emits an Unknown symbol address (`elf.rs:264` is
//! `Fact::known(…)`), so the symbol case is proven from a snapshot that reports one — which pins the
//! storage contract without claiming a symptom the shipped parser does not produce.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::{BuildSnapshot, SnapshotBuilder};
use firmwaresight_core::domain::capability::Capabilities;
use firmwaresight_core::domain::identity::Fact;
use firmwaresight_core::domain::symbol::{Symbol, SymbolBinding, SymbolKind, SymbolSectionRef};
use firmwaresight_storage::{Database, SCHEMA_VERSION, SectionQuery};
use rusqlite::params;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/<name>")
        .to_path_buf()
}

/// The ELF + GNU ld MAP fixture. Its `.bss` has no file range, so the parser reports that offset as
/// Unknown with a reason — the fact this file is about.
fn dual_region_snapshot() -> BuildSnapshot {
    let root = repo_root();
    pipeline::analyze(
        &AnalysisRequest::new(root.join("fixtures/elf/p0-dual-region/firmware.elf"))
            .with_map(root.join("fixtures/elf/p0-dual-region/firmware.map")),
    )
    .expect("the committed fixture must analyze")
    .snapshot
    .clone()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "fwsight-unknown-reasons-{label}-{}-{:?}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        path.push(unique);
        let _ = std::fs::remove_file(&path);
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

fn open(file: &TempDb) -> Database {
    Database::open(file.path()).expect("open and migrate")
}

fn schema_version(db: &Database) -> i64 {
    db.connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .expect("version")
}

// ------------------------------------------------------------------- L6, from a committed fixture

#[test]
fn a_section_offset_reported_unknown_keeps_the_reason_the_parser_gave_it() {
    let snapshot = dual_region_snapshot();
    let (index, reason) = snapshot
        .sections()
        .iter()
        .filter_map(|section| {
            section
                .file_offset
                .reason_if_unknown()
                .map(|reason| (section.index, reason.to_owned()))
        })
        .next()
        .expect("the fixture must contain a section whose file offset is Unknown, such as .bss");
    assert!(
        !reason.is_empty(),
        "the parser's Unknown must come with a reason to preserve"
    );

    let file = TempDb::new("section-reason");
    let mut db = open(&file);
    let build = db
        .import_snapshot("proj-1", "P0 project", &snapshot)
        .expect("import");

    let (stored_value, stored_reason): (Option<i64>, Option<String>) = db
        .connection()
        .query_row(
            "SELECT file_offset, file_offset_unknown FROM sections
                  WHERE build_id = ?1 AND section_index = ?2",
            params![build, index as i64],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("an unknown numeric fact must be stored with its reason, not as a bare NULL");

    assert_eq!(
        stored_value, None,
        "an Unknown offset is never written as a number"
    );
    assert_eq!(
        stored_reason.as_deref(),
        Some(reason.as_str()),
        "the reason the parser recorded must survive the write"
    );
}

// ------------------------------------------------------------------ L7, from a reported Unknown

/// A snapshot that reports one symbol address as Unknown. The ELF parser does not do this today, so the
/// case is constructed rather than claimed: storage must keep a reason it is handed, whatever produces it.
fn snapshot_with_unknown_symbol_address(base: &BuildSnapshot) -> BuildSnapshot {
    let mut symbols = base.symbols().to_vec();
    symbols.insert(
        0,
        Symbol {
            name: Fact::known("no_value_entry".to_owned()),
            address: Fact::unknown("the symbol entry is undefined and carries no value"),
            size: Fact::known(4),
            kind: SymbolKind::Object,
            binding: SymbolBinding::Global,
            section: SymbolSectionRef::Undefined,
        },
    );

    SnapshotBuilder::new(base.created_by_fwsight_version())
        .project_id("proj-1")
        .artifact(base.primary_artifact().expect("artifact").clone())
        .memory(base.memory().expect("memory").clone())
        .sections(base.sections().to_vec())
        .symbols(symbols)
        .evidence(base.evidence().to_vec())
        .capabilities(Capabilities::default())
        .seal()
        .expect("seal")
}

#[test]
fn a_symbol_address_reported_unknown_keeps_its_reason_and_not_a_bare_null() {
    let base = dual_region_snapshot();
    let reason = {
        let snapshot = snapshot_with_unknown_symbol_address(&base);
        snapshot
            .symbols()
            .first()
            .and_then(|symbol| symbol.address.reason_if_unknown())
            .expect("the constructed symbol must report an Unknown address")
            .to_owned()
    };

    let file = TempDb::new("symbol-reason");
    let mut db = open(&file);
    let snapshot = snapshot_with_unknown_symbol_address(&base);
    let build = db
        .import_snapshot("proj-1", "P0 project", &snapshot)
        .expect("import");

    let (stored_value, stored_reason): (Option<i64>, Option<String>) = db
        .connection()
        .query_row(
            "SELECT address, address_unknown FROM symbols
                  WHERE build_id = ?1 AND ordinal = 0",
            params![build],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("an unknown numeric fact must be stored with its reason, not as a bare NULL");

    assert_eq!(
        stored_value, None,
        "an Unknown address is never written as 0"
    );
    assert_eq!(
        stored_reason.as_deref(),
        Some(reason.as_str()),
        "the reason must survive the write"
    );
}

// ------------------------------------------------------------------------ schema shape and upgrade

#[test]
fn the_unknown_offset_reason_reaches_a_reader_through_the_query_layer() {
    // L6 is a claim about what a reader can say, not about what a column can hold, so it is proven on
    // the paging path the desktop actually calls. The fixture has 17 sections, well inside the default
    // page, which keeps this about the reason rather than about pagination.
    let snapshot = dual_region_snapshot();
    let (index, reason) = snapshot
        .sections()
        .iter()
        .find_map(|section| {
            section
                .file_offset
                .reason_if_unknown()
                .map(|reason| (section.index, reason.to_owned()))
        })
        .expect("the fixture must contain a section whose file offset is Unknown");

    let file = TempDb::new("section-reason-read");
    let mut db = open(&file);
    db.import_snapshot("proj-1", "P0 project", &snapshot)
        .expect("import");

    let page = db
        .query_sections(&SectionQuery {
            snapshot_id: snapshot.id().as_str().to_owned(),
            ..SectionQuery::default()
        })
        .expect("sections page");

    let row = page
        .rows
        .iter()
        .find(|row| row.index == i64::try_from(index).expect("a section index that fits"))
        .expect("the unknown-offset section is on the page");

    assert_eq!(
        row.file_offset,
        Fact::unknown(reason),
        "an Unknown offset must come back as Unknown with its reason, not as an unexplained absence"
    );
    assert_eq!(
        row.file_offset.value(),
        None,
        "and still never as a zero the artifact never reported"
    );
}

#[test]
fn the_schema_pairs_every_nullable_numeric_column_with_a_reason() {
    let file = TempDb::new("shape");
    let db = open(&file);

    assert_eq!(
        SCHEMA_VERSION, 6,
        "P5 raised the schema to version 5; C1-U1's additive attachment table raised it to 6, and \
         neither of the two reason columns moved"
    );
    assert_eq!(schema_version(&db), SCHEMA_VERSION);

    let columns = |table: &str, column: &str| -> i64 {
        db.connection()
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
                params![table, column],
                |r| r.get(0),
            )
            .expect("table info")
    };
    for (table, column) in [
        ("sections", "file_offset_unknown"),
        ("symbols", "address_unknown"),
    ] {
        assert_eq!(
            columns(table, column),
            1,
            "{table}.{column} must exist so a reason can be stored at all"
        );
    }
}

#[test]
fn a_version_four_store_upgrades_without_inventing_reasons_for_its_own_rows() {
    // Build the version-4 shape by hand, exactly as the migration tests before this one do, then
    // require `open` to bring it forward. The row written under version 4 has no reason to recover, so
    // the upgraded row must say so by holding NULL - which the read model renders as "the version that
    // wrote this row did not record why", never as a reason this round made up.
    let file = TempDb::new("from-v4");
    {
        let conn = rusqlite::Connection::open(file.path()).expect("raw create");
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .expect("version 1 schema");
        conn.execute_batch(include_str!(
            "../migrations/0002_evidence_keyed_by_build.sql"
        ))
        .expect("version 2 schema");
        conn.execute_batch(include_str!("../migrations/0003_gate_history.sql"))
            .expect("version 3 schema");
        conn.execute_batch(include_str!("../migrations/0004_release_records.sql"))
            .expect("version 4 schema");
        conn.execute_batch(
            "CREATE TABLE schema_migrations (
                 version    INTEGER PRIMARY KEY NOT NULL,
                 name       TEXT NOT NULL,
                 applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
             );
             INSERT INTO schema_migrations (version, name) VALUES (1, '0001_initial');
             INSERT INTO schema_migrations (version, name) VALUES (2, '0002_evidence_keyed_by_build');
             INSERT INTO schema_migrations (version, name) VALUES (3, '0003_gate_history');
             INSERT INTO schema_migrations (version, name) VALUES (4, '0004_release_records');
             INSERT INTO projects (id, name) VALUES ('proj-1', 'P0 project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
                  VALUES ('build-v4', 'proj-1', 'snap-v4', 'p0-normalize-1', '0.1.0', 'COMPLETE');
             INSERT INTO sections (build_id, section_index, name, role, is_alloc, is_write,
                                   is_execute, file_offset, file_size)
                  VALUES ('build-v4', 7, '.bss', 'nobits', 1, 1, 0, NULL, 0);",
        )
        .expect("version 4 database with one unknown-offset row");
        drop(conn);
    }

    let db = open(&file);
    assert_eq!(
        schema_version(&db),
        SCHEMA_VERSION,
        "a version 4 store is brought forward on open"
    );

    let (name, value, reason): (String, Option<i64>, Option<String>) = db
        .connection()
        .query_row(
            "SELECT name, file_offset, file_offset_unknown FROM sections
                  WHERE build_id = 'build-v4' AND section_index = 7",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("the version 4 row survives the upgrade");

    assert_eq!(name, ".bss", "the row is the one that was written");
    assert_eq!(value, None, "its Unknown offset is still not a number");
    assert_eq!(
        reason, None,
        "a reason this round never had must stay unrecorded rather than be invented"
    );
}
