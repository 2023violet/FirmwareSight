//! The pinned baseline (R5 / D3, migration 0008): one measurement reference per project, kept as an
//! append-only log so that pinning and unpinning both leave a record.
//!
//! The contract these tests hold:
//!
//! - nothing is pinned until a person pins it, and an unpin returns the project to that state;
//! - a pin names a complete build of the project and nothing else: an unknown snapshot, or a build of
//!   another project, is refused as not found;
//! - pinning another build replaces the current pin, and the old one stays in the log;
//! - the log refuses an edit or a delete;
//! - a pin is read back by file name and digest, never by path.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_storage::{Database, SCHEMA_VERSION, StorageError};
use rusqlite::params;

const LOCAL: &str = "local-desktop";
const OTHER: &str = "someone-elses-project";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/<name>")
        .to_path_buf()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-pins-{label}-{}-{:x}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_file(&path);
        Self(path)
    }

    fn open(&self) -> Database {
        Database::open(&self.0).expect("database opens")
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

fn analyze(side: &str) -> BuildSnapshot {
    let root = repo_root().join(format!("fixtures/elf/p2-diff/{side}"));
    pipeline::analyze(
        &AnalysisRequest::new(root.join("firmware.elf")).with_map(root.join("firmware.map")),
    )
    .unwrap_or_else(|err| panic!("the {side} fixture must analyze: {err}"))
    .snapshot
    .clone()
}

/// Two complete builds in the local project, returned as their snapshot ids (base, target).
fn two_builds(db: &mut Database) -> (String, String) {
    let snapshot_of = |db: &mut Database, snapshot: &BuildSnapshot| -> String {
        let build = db
            .import_snapshot(LOCAL, "Pin test project", snapshot)
            .expect("import");
        db.connection()
            .query_row(
                "SELECT snapshot_id FROM builds WHERE id = ?1",
                params![build],
                |row| row.get(0),
            )
            .expect("the imported build has a snapshot id")
    };
    let base = snapshot_of(db, &analyze("base"));
    let target = snapshot_of(db, &analyze("target"));
    (base, target)
}

fn events(db: &Database) -> Vec<(String, Option<String>)> {
    let mut stmt = db
        .connection()
        .prepare("SELECT action, build_id FROM baseline_pin_events ORDER BY seq")
        .expect("prepare");
    stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("query")
        .collect::<Result<Vec<_>, _>>()
        .expect("rows")
}

#[test]
fn the_schema_reaches_version_8_with_the_pin_log() {
    let store = TempDb::new("schema");
    let db = store.open();
    assert_eq!(SCHEMA_VERSION, 8);
    let count: i64 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'baseline_pin_events'",
            [],
            |row| row.get(0),
        )
        .expect("read schema");
    assert_eq!(count, 1);
}

#[test]
fn nothing_is_pinned_until_a_person_pins_a_build() {
    let store = TempDb::new("none");
    let mut db = store.open();
    two_builds(&mut db);
    assert_eq!(db.current_baseline_pin(LOCAL).expect("read"), None);
    assert!(
        !db.unpin_baseline(LOCAL).expect("unpin nothing"),
        "an unpin with nothing pinned changes nothing"
    );
    assert!(events(&db).is_empty(), "and writes no row");
}

#[test]
fn a_pin_reads_back_by_file_name_and_digest_never_by_path() {
    let store = TempDb::new("pin");
    let mut db = store.open();
    let (base, _) = two_builds(&mut db);
    let pin = db.pin_baseline(LOCAL, &base).expect("pin base");
    assert_eq!(pin.snapshot_id, base);
    assert_eq!(pin.file_name, "firmware.elf");
    assert_eq!(pin.sha256.len(), 64);
    assert!(!pin.file_name.contains('/') && !pin.file_name.contains('\\'));
    assert_eq!(db.current_baseline_pin(LOCAL).expect("read"), Some(pin));
}

#[test]
fn pinning_another_build_replaces_the_pin_and_keeps_the_old_one_in_the_log() {
    let store = TempDb::new("replace");
    let mut db = store.open();
    let (base, target) = two_builds(&mut db);
    let first = db.pin_baseline(LOCAL, &base).expect("pin base");
    // Pinning the same build again writes nothing.
    assert_eq!(db.pin_baseline(LOCAL, &base).expect("again"), first);
    assert_eq!(events(&db).len(), 1);
    let second = db.pin_baseline(LOCAL, &target).expect("pin target");
    assert_eq!(second.snapshot_id, target);
    assert_eq!(
        db.current_baseline_pin(LOCAL)
            .expect("read")
            .map(|p| p.snapshot_id),
        Some(target)
    );
    let log = events(&db);
    assert_eq!(log.len(), 2);
    assert_eq!(log[0].1.as_deref(), Some(first.build_id.as_str()));
}

#[test]
fn an_unpin_is_a_record_and_returns_the_project_to_no_pin() {
    let store = TempDb::new("unpin");
    let mut db = store.open();
    let (base, _) = two_builds(&mut db);
    db.pin_baseline(LOCAL, &base).expect("pin");
    assert!(db.unpin_baseline(LOCAL).expect("unpin"));
    assert_eq!(db.current_baseline_pin(LOCAL).expect("read"), None);
    let log = events(&db);
    assert_eq!(log.len(), 2);
    assert_eq!(log[1], ("unpin".to_owned(), None));
}

#[test]
fn only_a_complete_build_of_the_project_can_be_pinned() {
    let store = TempDb::new("scope");
    let mut db = store.open();
    let (base, _) = two_builds(&mut db);
    assert!(matches!(
        db.pin_baseline(LOCAL, "snap-that-does-not-exist"),
        Err(StorageError::NotFound { .. })
    ));
    // A build stored under another project is not this project's to pin, and the pins do not mix.
    assert!(matches!(
        db.pin_baseline(OTHER, &base),
        Err(StorageError::NotFound { .. })
    ));
    db.pin_baseline(LOCAL, &base).expect("pin local");
    assert_eq!(db.current_baseline_pin(OTHER).expect("read other"), None);
}

#[test]
fn the_log_refuses_an_edit_or_a_delete() {
    let store = TempDb::new("immutable");
    let mut db = store.open();
    let (base, _) = two_builds(&mut db);
    db.pin_baseline(LOCAL, &base).expect("pin");
    let edit = db.connection().execute(
        "UPDATE baseline_pin_events SET action = 'unpin', build_id = NULL",
        params![],
    );
    assert!(edit.is_err(), "an edit is refused");
    let delete = db
        .connection()
        .execute("DELETE FROM baseline_pin_events", params![]);
    assert!(delete.is_err(), "a delete is refused");
    assert_eq!(events(&db).len(), 1);
}
