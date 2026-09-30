//! The same bytes with a stronger evidence input must become a second, distinct build.
//!
//! This is the persistence half of the P1-A0 correctness gap: an ELF analyzed alone and that same
//! ELF analyzed with its GNU ld MAP produced the same snapshot id, so `Session::store` recognised
//! the id as already imported and refused the stronger record. The UI showed MAP-backed evidence
//! while SQLite kept the ELF-only version of the truth.
//!
//! No migration is involved: the schema already stores N artifacts per build, and these tests
//! prove that the second artifact row is what the identity, and therefore the dedupe, now sees.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_storage::{BuildSummary, Database, SCHEMA_VERSION};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/<name>")
        .to_path_buf()
}

fn elf() -> PathBuf {
    repo_root().join("fixtures/elf/p0-dual-region/firmware.elf")
}

fn map() -> PathBuf {
    repo_root().join("fixtures/elf/p0-dual-region/firmware.map")
}

fn snapshot(with_map: bool) -> BuildSnapshot {
    let mut request = AnalysisRequest::new(elf());
    if with_map {
        request = request.with_map(map());
    }
    pipeline::analyze(&request)
        .expect("the committed fixture analyzes")
        .snapshot
        .clone()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-map-companion-{label}-{}-{:x}.sqlite",
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

fn build_count(db: &Database) -> i64 {
    db.connection()
        .query_row("SELECT COUNT(*) FROM builds", [], |row| row.get(0))
        .expect("build count")
}

fn artifact_rows(db: &Database, build_id: &str) -> Vec<(String, String, String)> {
    let mut stmt = db
        .connection()
        .prepare("SELECT id, kind, sha256 FROM artifacts WHERE build_id = ?1 ORDER BY id")
        .expect("select artifacts");
    stmt.query_map((build_id,), |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })
    .expect("artifact rows")
    .map(|r| r.expect("row reads"))
    .collect()
}

#[test]
fn a_map_makes_a_second_build_rather_than_updating_the_first() {
    let dir = TempDb::new("two-builds");
    let mut db = Database::open(dir.path()).expect("opens");

    let weak = snapshot(false);
    let strong = snapshot(true);
    assert_ne!(
        weak.id(),
        strong.id(),
        "the two runs must not collide onto one content address"
    );

    let build_x = db
        .import_snapshot("local-desktop", "Local analyses", &weak)
        .expect("ELF-only imports");
    assert_eq!(build_count(&db), 1);
    assert_eq!(artifact_rows(&db, &build_x).len(), 1);

    let build_y = db
        .import_snapshot("local-desktop", "Local analyses", &strong)
        .expect("ELF + MAP imports");
    assert_ne!(build_x, build_y, "the stronger record is its own build");
    assert_eq!(build_count(&db), 2, "the ELF-only build stays as history");
}

#[test]
fn the_map_build_persists_both_artifacts_and_the_stronger_evidence() {
    let dir = TempDb::new("strong");
    let mut db = Database::open(dir.path()).expect("opens");

    let weak = snapshot(false);
    let strong = snapshot(true);
    let build_x = db
        .import_snapshot("local-desktop", "Local analyses", &weak)
        .expect("ELF-only imports");
    let build_y = db
        .import_snapshot("local-desktop", "Local analyses", &strong)
        .expect("ELF + MAP imports");

    let rows = artifact_rows(&db, &build_y);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].1, "Elf");
    assert_eq!(rows[1].1, "Map");
    assert_eq!(
        rows[1].2,
        strong.artifacts()[1].sha256.hex(),
        "the MAP row carries the MAP bytes hash"
    );
    assert_eq!(
        artifact_rows(&db, &build_x).len(),
        1,
        "the earlier ELF-only build is untouched"
    );

    let x = db.summary(&build_x).expect("ELF-only summary");
    let y = db.summary(&build_y).expect("ELF + MAP summary");
    assert_eq!(y.layout_source, "map");
    assert_eq!(x.layout_source, "none");
    assert!(
        y.admissible_hard_block,
        "the MAP build may support a hard verdict"
    );
    assert!(!x.admissible_hard_block);
    assert!(
        y.evidence_count > x.evidence_count,
        "the persisted stronger evidence must be more than the weaker set: {x:?} vs {y:?}"
    );
}

#[test]
fn repeating_either_input_dedupes_and_never_stores_a_third_build() {
    let dir = TempDb::new("dedupe");
    let mut db = Database::open(dir.path()).expect("opens");

    let weak = snapshot(false);
    let strong = snapshot(true);

    // The shell asks before importing, which is what makes a repeat a no-op rather than a UNIQUE
    // violation on the build id. `store` mirrors `Session::store` exactly.
    let store = |db: &mut Database, s: &BuildSnapshot| {
        if db
            .build_id_for_snapshot(s.id().as_str())
            .expect("lookup")
            .is_none()
        {
            db.import_snapshot("local-desktop", "Local analyses", s)
                .expect("imports");
        }
    };

    store(&mut db, &weak);
    store(&mut db, &strong);
    store(&mut db, &strong);
    store(&mut db, &weak);
    store(&mut db, &strong);

    assert_eq!(
        build_count(&db),
        2,
        "two distinct evidence inputs, however often re-analyzed, are two builds"
    );
    assert_eq!(
        db.build_id_for_snapshot(weak.id().as_str())
            .expect("lookup")
            .map(|id| artifact_rows(&db, &id).len()),
        Some(1),
        "removing the MAP returns to the ELF-only build, not to a third one"
    );
    assert_eq!(
        db.build_id_for_snapshot(strong.id().as_str())
            .expect("lookup")
            .map(|id| artifact_rows(&db, &id).len()),
        Some(2)
    );
}

#[test]
fn a_two_artifact_build_summarizes_the_primary_elf_never_the_map_placeholder() {
    let dir = TempDb::new("primary");
    let mut db = Database::open(dir.path()).expect("opens");

    let strong = snapshot(true);
    let build_y = db
        .import_snapshot("local-desktop", "Local analyses", &strong)
        .expect("imports");

    let summary: BuildSummary = db.summary(&build_y).expect("summary");
    let elf = &strong.artifacts()[0];
    let map = &strong.artifacts()[1];

    assert_eq!(summary.sha256, elf.sha256.hex());
    assert_eq!(summary.byte_size, elf.byte_size);
    assert_eq!(summary.architecture, format!("{:?}", elf.architecture));
    assert_eq!(summary.bitness, format!("{:?}", elf.bitness));
    assert_eq!(summary.endianness, format!("{:?}", elf.endianness));

    assert_ne!(summary.sha256, map.sha256.hex(), "the MAP is not the build");
    assert_ne!(summary.byte_size, map.byte_size);
    assert_ne!(
        summary.architecture, "Unknown",
        "a MAP artifact reports Unknown architecture, so a summary that says Unknown read the \
         wrong row"
    );
    assert_ne!(summary.bitness, "Unknown");
}

#[test]
fn the_summary_still_names_the_primary_elf_when_the_rows_are_not_in_insertion_order() {
    // Without this the test above only proves SQLite happened to return the ELF first. The old
    // summary joined the artifacts table by build id and took whichever row came back, so the
    // hazard is invisible until a row order changes. The scratch database here is this test's own,
    // which is what lets it be reshuffled deliberately: `#0` is moved to the highest rowid, so any
    // scan that leans on row order now finds the MAP artifact first.
    let dir = TempDb::new("order");
    let mut db = Database::open(dir.path()).expect("opens");

    let strong = snapshot(true);
    let build_y = db
        .import_snapshot("local-desktop", "Local analyses", &strong)
        .expect("imports");

    let shuffled = db
        .connection()
        .execute(
            "UPDATE artifacts SET rowid = (SELECT MAX(rowid) + 1 FROM artifacts) WHERE id = ?1",
            (format!("{build_y}#0"),),
        )
        .expect("the primary row moves to the end");
    assert_eq!(shuffled, 1, "the ELF artifact row is the one that moved");

    let first_by_rowid: String = db
        .connection()
        .query_row(
            "SELECT kind FROM artifacts WHERE build_id = ?1 ORDER BY rowid LIMIT 1",
            (build_y.clone(),),
            |row| row.get(0),
        )
        .expect("row order observed");
    assert_eq!(
        first_by_rowid, "Map",
        "the fixture no longer reproduces the hazard, so this test proves nothing"
    );

    let summary = db.summary(&build_y).expect("summary");
    assert_eq!(
        summary.sha256,
        strong.artifacts()[0].sha256.hex(),
        "the summary must follow the artifact index, not the row order"
    );
    assert_eq!(
        summary.architecture,
        format!("{:?}", strong.artifacts()[0].architecture)
    );
}

#[test]
fn closing_the_identity_gap_adds_no_schema_version_and_no_migration() {
    let dir = TempDb::new("schema");
    let mut db = Database::open(dir.path()).expect("opens");
    db.import_snapshot("local-desktop", "Local analyses", &snapshot(true))
        .expect("imports");

    assert_eq!(SCHEMA_VERSION, 3);
    let version: i64 = db
        .connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .expect("schema version");
    assert_eq!(
        version, 3,
        "the identity closure must not move the schema on its own"
    );

    let applied: Vec<(i64, String)> = {
        let mut stmt = db
            .connection()
            .prepare("SELECT version, name FROM schema_migrations ORDER BY version")
            .expect("select migrations");
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("migrations")
            .map(|r| r.expect("row reads"))
            .collect()
    };
    assert_eq!(
        applied,
        vec![
            (1, "0001_initial".to_owned()),
            (2, "0002_evidence_keyed_by_build".to_owned()),
            // 0003 is P3's Gate history. The identity closure wrote none of the three.
            (3, "0003_gate_history".to_owned()),
        ],
        "every migration belongs to a named stage, and none of them is the identity closure's"
    );
}
