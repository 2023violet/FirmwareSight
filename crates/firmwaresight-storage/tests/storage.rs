//! Storage acceptance tests for P0: the boundary must hold, and a failed import must not be
//! observable as a finished build.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::{BuildSnapshot, SnapshotBuilder};
use firmwaresight_storage::{BuildSummary, Database, SCHEMA_VERSION};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/<name>")
        .to_path_buf()
}

fn real_snapshot() -> BuildSnapshot {
    let root = repo_root();
    let analysis = pipeline::analyze(
        &AnalysisRequest::new(root.join("fixtures/elf/p0-dual-region/firmware.elf"))
            .with_map(root.join("fixtures/elf/p0-dual-region/firmware.map")),
    )
    .expect("the committed fixture must analyze");
    analysis.snapshot.clone()
}

/// The other committed fixture: a different build, and therefore a different set of facts recorded
/// under the same evidence identifiers.
fn second_snapshot() -> BuildSnapshot {
    let root = repo_root();
    let analysis = pipeline::analyze(&AnalysisRequest::new(
        root.join("fixtures/elf/p0-basic/firmware.elf"),
    ))
    .expect("the second committed fixture must analyze");
    analysis.snapshot.clone()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "fwsight-p0-{label}-{}-{:?}.sqlite",
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

impl Drop for TempDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let mut candidate = self.0.as_os_str().to_os_string();
            candidate.push(suffix);
            let _ = std::fs::remove_file(Path::new(&candidate));
        }
    }
}

#[test]
fn a_fresh_database_migrates_to_the_supported_version() {
    let db_file = TempDb::new("fresh");
    let db = Database::open(db_file.path()).expect("open applies migrations");

    let version: i64 = db
        .connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .expect("a migration row exists");
    assert_eq!(version, SCHEMA_VERSION);

    let name: String = db
        .connection()
        .query_row(
            "SELECT name FROM schema_migrations WHERE version = 1",
            [],
            |r| r.get(0),
        )
        .expect("migration 1 is recorded");
    assert_eq!(name, "0001_initial");
}

#[test]
fn reopening_an_existing_database_is_idempotent() {
    let db_file = TempDb::new("reopen");

    {
        let mut db = Database::open(db_file.path()).expect("first open");
        db.migrate().expect("migrate is safe to call again");
        let snapshot = real_snapshot();
        db.import_snapshot("proj-1", "P0 project", &snapshot)
            .expect("import");
    }

    // Reopen: the schema must be recognised, not rebuilt, and history must survive.
    let db = Database::open(db_file.path()).expect("second open");
    let builds: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM builds", [], |r| r.get(0))
        .expect("builds table exists");
    assert_eq!(builds, 1, "reopening must not discard stored history");

    let migrations: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
        .expect("schema_migrations exists");
    assert_eq!(
        migrations, SCHEMA_VERSION,
        "every migration is present exactly once; reopening must not record one twice"
    );
}

#[test]
fn foreign_keys_are_enforced_on_the_connection() {
    let db_file = TempDb::new("fk");
    let db = Database::open(db_file.path()).expect("open");

    let result = db.connection().execute(
        "INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                             created_by_fwsight, state)
         VALUES ('b-orphan','no-such-project','snap','v','0.1.0','COMPLETE')",
        [],
    );

    assert!(
        result.is_err(),
        "a build referencing a nonexistent project must be rejected"
    );
}

#[test]
fn a_snapshot_survives_a_full_round_trip_with_identical_facts() {
    let db_file = TempDb::new("roundtrip");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = real_snapshot();

    let build_id = db
        .import_snapshot("proj-1", "P0 project", &snapshot)
        .expect("import a real snapshot");
    let summary: BuildSummary = db.summary(&build_id).expect("read it back");

    let artifact = snapshot.primary_artifact().expect("one artifact");
    let memory = snapshot.memory().expect("footprint");

    assert_eq!(summary.state, "COMPLETE");
    assert_eq!(summary.snapshot_id, snapshot.id().as_str());
    assert_eq!(summary.sha256, artifact.sha256.hex());
    assert_eq!(summary.byte_size, artifact.byte_size);
    assert_eq!(summary.architecture, format!("{:?}", artifact.architecture));
    assert_eq!(summary.bitness, "Bits32");
    assert_eq!(summary.endianness, "Little");
    assert_eq!(summary.section_count, snapshot.section_count() as i64);
    assert_eq!(summary.symbol_count, snapshot.symbol_count() as i64);
    assert_eq!(summary.evidence_count, snapshot.evidence().len() as i64);
    assert_eq!(summary.normalization_version, "p0-normalize-1");

    // The two budgets must come back as the two distinct numbers that were written.
    assert_eq!(summary.nonvolatile_state, "exact");
    assert_eq!(summary.runtime_state, "exact");
    assert_eq!(
        summary.nonvolatile_bytes,
        memory.nonvolatile.bytes().map(|b| b as i64)
    );
    assert_eq!(
        summary.runtime_bytes,
        memory.runtime_ram.bytes().map(|b| b as i64)
    );
    assert_ne!(
        summary.nonvolatile_bytes, summary.runtime_bytes,
        "the two budgets must not collapse into one stored number"
    );
    assert!(summary.admissible_hard_block);
    assert_eq!(summary.layout_source, "map");
}

#[test]
fn per_section_and_evidence_detail_is_retrievable() {
    let db_file = TempDb::new("detail");
    let mut db = Database::open(db_file.path()).expect("open");
    let snapshot = real_snapshot();
    let build_id = db
        .import_snapshot("proj-1", "P0 project", &snapshot)
        .expect("import");

    // `.data` is the section whose dual charge the whole model exists to prove.
    let row: (Option<i64>, Option<i64>, Option<String>, String) = db
        .connection()
        .query_row(
            "SELECT load_addr, file_size, region, role FROM sections
              WHERE build_id = ?1 AND name = '.data'",
            [&build_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .expect("the .data section was stored");

    assert_eq!(row.0, Some(0x0800_005c), "LMA must persist as ROM");
    assert_eq!(row.1, Some(4));
    assert_eq!(
        row.2.as_deref(),
        Some("RAM"),
        "runtime region must persist as RAM"
    );
    assert_eq!(row.3, "InitializedData");

    let unattributed: i64 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM sections WHERE build_id = ?1 AND load_addr IS NULL
               AND load_unknown IS NOT NULL",
            [&build_id],
            |r| r.get(0),
        )
        .expect("count");
    assert!(
        unattributed > 0,
        "sections without load evidence must be stored as unknown-with-a-reason, not as zero"
    );
}

#[test]
fn a_failed_import_leaves_no_build_visible_as_complete() {
    // A snapshot carrying a duplicated evidence id violates the evidence primary key halfway
    // through the write, which is the shape of a genuine partial-import failure.
    let db_file = TempDb::new("rollback");
    let mut db = Database::open(db_file.path()).expect("open");

    let base = real_snapshot();
    let mut evidence = base.evidence().to_vec();
    let duplicate = evidence[0].clone();
    evidence.push(duplicate);

    let artifact = base.primary_artifact().expect("artifact").clone();
    let tainted: BuildSnapshot = SnapshotBuilder::new("0.1.0")
        .project_id("proj-1")
        .artifact(artifact)
        .memory(base.memory().expect("memory").clone())
        .sections(base.sections().to_vec())
        .symbols(base.symbols().to_vec())
        .evidence(evidence)
        .capabilities(base.capabilities().clone())
        .seal()
        .expect("seal");

    let result = db.import_snapshot("proj-1", "P0 project", &tainted);
    assert!(
        result.is_err(),
        "a duplicated evidence id must fail the import, not be silently absorbed"
    );

    let builds: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM builds", [], |r| r.get(0))
        .expect("count builds");
    let complete: i64 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM builds WHERE state = 'COMPLETE'",
            [],
            |r| r.get(0),
        )
        .expect("count complete");
    let sections: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM sections", [], |r| r.get(0))
        .expect("count sections");
    let projects: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))
        .expect("count projects");

    assert_eq!(builds, 0, "a rolled-back import must not leave a build row");
    assert_eq!(complete, 0);
    assert_eq!(
        sections, 0,
        "no partial section list may survive a failed import"
    );
    assert_eq!(
        projects, 0,
        "the whole transaction must unwind, not just its tail"
    );

    // And the database stays usable afterwards.
    let build_id = db
        .import_snapshot("proj-1", "P0 project", &base)
        .expect("a clean import still works after a rollback");
    assert_eq!(db.summary(&build_id).expect("summary").state, "COMPLETE");
}

#[test]
fn an_unknown_newer_schema_is_refused_rather_than_reset() {
    let db_file = TempDb::new("future");

    {
        let db = Database::open(db_file.path()).expect("create");
        db.connection()
            .execute(
                "INSERT INTO schema_migrations (version, name) VALUES (99, 'from-the-future')",
                [],
            )
            .expect("insert future version");
    }

    match Database::open(db_file.path()) {
        Ok(_) => panic!("a newer schema must not be silently accepted"),
        Err(err) => assert!(
            err.to_string().contains("does not understand"),
            "expected an explicit version refusal, got {err:?}"
        ),
    }

    // The refusal must not have destroyed anything.
    let conn = rusqlite::Connection::open(db_file.path()).expect("raw open");
    let still_there: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = 99",
            [],
            |r| r.get(0),
        )
        .expect("future row preserved");
    assert_eq!(
        still_there, 1,
        "refusing a newer schema must never delete it"
    );
}

#[test]
fn the_same_snapshot_is_never_stored_twice() {
    // Snapshot ids are content-addressed, so a second appearance of one is the identical import.
    // The desktop re-derives a summary on every click; if that wrote a duplicate build each time,
    // history would fill with copies of one artifact and the primary key would reject it anyway.
    let db_file = TempDb::new("dedupe");
    let mut db = Database::open(db_file.path()).expect("create");
    let snapshot = real_snapshot();

    assert!(
        db.build_id_for_snapshot(snapshot.id().as_str())
            .expect("lookup before any import")
            .is_none(),
        "an empty database must report nothing"
    );

    let build_id = db
        .import_snapshot("proj-1", "P0 project", &snapshot)
        .expect("first import");
    assert_eq!(
        db.build_id_for_snapshot(snapshot.id().as_str())
            .expect("lookup after import")
            .as_deref(),
        Some(build_id.as_str()),
        "the stored build must be findable by snapshot id"
    );

    let builds: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM builds", [], |r| r.get(0))
        .expect("count builds");
    assert_eq!(builds, 1);
}

#[test]
fn the_connection_carries_the_pragmas_the_schema_assumes() {
    // Foreign keys and a busy timeout are set at open time, and the whole concurrency story
    // depends on them. Asserting them here means a future change to `establish()` fails a test
    // instead of quietly weakening the database.
    let db_file = TempDb::new("pragmas");
    let db = Database::open(db_file.path()).expect("open");

    let foreign_keys: i64 = db
        .connection()
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("read foreign_keys pragma");
    assert_eq!(
        foreign_keys, 1,
        "cascade rules are meaningless with FK enforcement off"
    );

    let journal_mode: String = db
        .connection()
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("read journal mode");
    assert!(
        matches!(
            journal_mode.as_str(),
            "wal" | "delete" | "memory" | "truncate"
        ),
        "an unexpected journal mode means the open path changed: got {journal_mode}"
    );

    let timeout: i64 = db
        .connection()
        .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
        .expect("read busy timeout");
    assert!(
        timeout > 0,
        "a zero busy timeout turns contention into an immediate error"
    );
}

#[test]
fn two_builds_may_record_the_same_evidence_identifier() {
    // `04_TECH/15` 4 makes evidence a child of a build, and the analyzer names its evidence within
    // one analysis: `ev-sha256` is the sha256 fact of whatever artifact was read, so every build
    // has one. Keying that identifier globally - which `0001_initial.sql` did - means the second
    // artifact a user analyzes cannot be stored at all. The desktop shipped that way: analyzing
    // fixture B and then fixture A failed with `UNIQUE constraint failed: evidence.id` and the
    // window showed ERR-STORAGE-4006, while 102 Rust tests stayed green because no test had ever
    // put two builds in one database.
    let db_file = TempDb::new("two-builds");
    let mut db = Database::open(db_file.path()).expect("open");

    let first = db
        .import_snapshot("proj-1", "P0 project", &real_snapshot())
        .expect("the first build imports");
    let second = db
        .import_snapshot("proj-1", "P0 project", &second_snapshot())
        .expect("a second build may reuse an identifier the first one used");
    assert_ne!(first, second, "two artifacts are two builds");

    for (build, expected) in [(&first, 11_usize), (&second, 10_usize)] {
        let stored: i64 = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM evidence WHERE build_id = ?1",
                [build],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(
            stored as usize, expected,
            "each build keeps its own full evidence set"
        );

        let shared: i64 = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM evidence WHERE build_id = ?1 AND id = 'ev-sha256'",
                [build],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(shared, 1, "one sha256 fact per build, resolved by build");
    }

    // The two builds must still disagree with each other, which is the point of storing both.
    let sha_of = |build: &str| {
        db.connection()
            .query_row(
                "SELECT raw_value FROM evidence WHERE build_id = ?1 AND id = 'ev-sha256'",
                [build],
                |r| r.get::<_, String>(0),
            )
            .expect("each build stores its own sha256 fact")
    };
    let first_hash = sha_of(&first);
    let second_hash = sha_of(&second);
    assert_ne!(
        first_hash, second_hash,
        "the shared identifier must not make the two builds share a value"
    );
}

#[test]
fn a_version_one_database_is_upgraded_without_losing_its_evidence() {
    // The upgrade rebuilds the evidence table, so the rows already in it are the risk. This builds
    // the version-1 shape by hand - schema, recorded migration, and one evidence row - and requires
    // `open` to bring it forward with the history intact and the new key in place.
    let db_file = TempDb::new("upgrade-0002");
    {
        let conn = rusqlite::Connection::open(db_file.path()).expect("raw create");
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .expect("version 1 schema");
        conn.execute_batch(
            "CREATE TABLE schema_migrations (
                 version    INTEGER PRIMARY KEY NOT NULL,
                 name       TEXT NOT NULL,
                 applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
             );
             INSERT INTO schema_migrations (version, name) VALUES (1, '0001_initial');
             INSERT INTO projects (id, name) VALUES ('proj-1', 'P0 project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
                  VALUES ('build-old', 'proj-1', 'snap-old', 'p0-normalize-1', '0.1.0', 'COMPLETE');
             INSERT INTO evidence (id, build_id, field, classification, source_type,
                                   source_locator, raw_value, rule, confidence)
                  VALUES ('ev-sha256', 'build-old', 'sha256', 'observed', 'filesystem',
                          'file:firmware.elf', 'old-hash', 'streaming-sha256/64KiB', NULL);",
        )
        .expect("version 1 contents");
    }

    let mut db = Database::open(db_file.path()).expect("open must upgrade in place");

    let version: i64 = db
        .connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .expect("version");
    assert_eq!(version, SCHEMA_VERSION);

    let kept: String = db
        .connection()
        .query_row(
            "SELECT raw_value FROM evidence WHERE build_id = 'build-old' AND id = 'ev-sha256'",
            [],
            |r| r.get(0),
        )
        .expect("the pre-upgrade row survives");
    assert_eq!(kept, "old-hash", "an upgrade must not rewrite history");

    let recorded: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
        .expect("count");
    assert_eq!(recorded, 3, "all three migrations are recorded, once each");

    db.import_snapshot("proj-1", "P0 project", &real_snapshot())
        .expect("the upgraded database accepts a build reusing the old identifier");
}
