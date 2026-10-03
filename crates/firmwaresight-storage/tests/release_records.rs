//! The Release Bundle audit boundary: migration `0004`, the immutability of a published release record,
//! and the rule that a release record is an index rather than the bundle (prompt §37, §38, §39, §54, §67).
//!
//! The runs these records point at are real `firmwaresight-core` evaluations of a hand-built
//! `GateContext`, so the foreign key is tested against the shape the Gate path actually writes.
//!
//! Two boundaries are asserted here rather than trusted. First, storage keeps **no release semantics**: it
//! does not look at a Gate disposition, because §8 is enforced by `ReleaseModel::validated` in Core, and a
//! second copy of that rule in SQL would be a second thing that can drift from the first. The tests below
//! therefore pin the *shape* rules only. Second, the table has no column that could hold a path or a blob,
//! which is read out of `PRAGMA table_info` so the promise survives a future `ALTER`.

use std::path::{Path, PathBuf};

use firmwaresight_core::domain::diff::{ByteChange, Comparability};
use firmwaresight_core::domain::gate::{
    EffectiveSeverity, FindingState, GateArtifactFact, GateBudgetFact, GateContext, GateGitFacts,
    GateGrowthFacts, GateMemoryFacts, GatePolicy, GateUnknownEvidence,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};
use firmwaresight_storage::{
    Database, GateRunDraft, GateRunWrite, ReleaseRecordDraft, ReleaseRecordWrite, SCHEMA_VERSION,
    StorageError, StoredReleaseRecord,
};
use rusqlite::params;

const TARGET_SNAPSHOT: &str = "11a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f80";
const BASELINE_SNAPSHOT: &str = "9f8e7d6c5b4a39281706f5e4d3c2b1a09876543210fedcba9876543210fedcba";
const HEAD_COMMIT: &str = "b12961ce1f2d3c4b5a69788796a5b4c3d2e1f004";

/// A 64-character lowercase hex digest for one test label, distinct per label, so the schema's digest
/// checks are exercised by name rather than skipped.
fn hex64(label: &str) -> String {
    let mut digest = String::with_capacity(64);
    for byte in label.bytes() {
        digest.push_str(&format!("{byte:02x}"));
    }
    while digest.len() < 64 {
        digest.push('0');
    }
    digest.truncate(64);
    digest
}

fn sha(label: &str) -> String {
    hex64(label)
}

fn run_id(label: &str) -> String {
    format!("gate-{}", hex64(label))
}

fn release_id(label: &str) -> String {
    format!("release-{}", hex64(label))
}

/// `import_snapshot` names a build `build-<snapshot id>`, so the tests use the same rule and keep one
/// identity for a build instead of two that could drift.
static TARGET_BUILD: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| format!("build-{TARGET_SNAPSHOT}"));
static BASELINE_BUILD: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| format!("build-{BASELINE_SNAPSHOT}"));
static POLICY_SHA: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| sha("rpolicy"));

/// A context whose verdict is `PASS`: a clean workspace, one ELF whose hash is recorded, exact
/// admissible budgets with no ceiling configured, no evidence gap and no requirement of notes.
fn pass_context() -> GateContext {
    GateContext {
        snapshot_id: TARGET_SNAPSHOT.to_owned(),
        artifacts: vec![GateArtifactFact {
            kind: ArtifactKind::Elf,
            sha256: Fact::known(sha("relf")),
            byte_size: 4_096,
        }],
        memory: Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact::exact(
                3_000,
                "elf-program-headers",
                "evidence:ev-memory-1",
            )),
            runtime_ram: Some(GateBudgetFact::exact(
                512,
                "elf-section-headers",
                "evidence:ev-memory-2",
            )),
        }),
        git: GateGitFacts {
            available: true,
            head_commit: Fact::known(HEAD_COMMIT.to_owned()),
            exact_tag: Fact::known("v1.4.2".to_owned()),
            dirty: Fact::known(false),
        },
        version: None,
        release_notes: None,
        growth: GateGrowthFacts {
            baseline_snapshot_id: None,
            nonvolatile: None,
            runtime_ram: None,
        },
        unknown_evidence: GateUnknownEvidence::default(),
        policy: GatePolicy {
            require_release_notes: false,
            ..GatePolicy::default()
        },
    }
}

/// The same build against a baseline it outgrew past the configured review ceiling: a `REVIEW` verdict,
/// which is the only kind a person can accept. Only the acceptance-preservation tests need a finding in
/// that state; the shape and immutability tests use the `PASS` run above.
fn review_context() -> GateContext {
    GateContext {
        growth: GateGrowthFacts {
            baseline_snapshot_id: Some(BASELINE_SNAPSHOT.to_owned()),
            nonvolatile: Some(ByteChange::between(
                Some(1_000),
                Some(2_000),
                Comparability::Exact,
            )),
            runtime_ram: None,
        },
        policy: GatePolicy {
            require_release_notes: false,
            flash_growth_review_bytes: Some(500),
            ..GatePolicy::default()
        },
        ..pass_context()
    }
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "fwsight-p4-release-{label}-{}-{:?}.sqlite",
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

fn seed_build(db: &Database, build_id: &str, snapshot_id: &str) {
    let conn = db.connection();
    conn.execute(
        "INSERT OR IGNORE INTO projects (id, name) VALUES ('proj-rel', 'Release project')",
        [],
    )
    .expect("project row");
    conn.execute(
        "INSERT INTO builds (id, project_id, snapshot_id, normalization_version, created_by_fwsight,
                             state)
         VALUES (?1, 'proj-rel', ?2, 'p0-normalize-1', '0.1.0', 'COMPLETE')",
        params![build_id, snapshot_id],
    )
    .expect("build row");
}

/// One stored Gate run on an already-open database, written through `gate.rs` so the release record's
/// foreign key lands on a row the Gate path really produces. The verdict is asserted, because a fixture
/// that silently stopped being the kind of run it claims to be would make every test below vacuous.
fn seed_run(
    db: &mut Database,
    label: &str,
    context: &GateContext,
    expected: EffectiveSeverity,
) -> String {
    let id = run_id(label);
    let evaluation = context.evaluate(&id);
    assert_eq!(
        evaluation.overall_effective_severity,
        expected,
        "the fixture run has to be {expected:?}, and it is not: {:?}",
        evaluation
            .findings
            .iter()
            .map(|finding| (finding.rule_id.as_str(), finding.state.as_str()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        db.persist_gate_run(&GateRunDraft {
            run_id: &id,
            build_id: TARGET_BUILD.as_str(),
            baseline_build_id: Some(BASELINE_BUILD.as_str()),
            policy_sha256: POLICY_SHA.as_str(),
            evaluation: &evaluation,
        })
        .expect("the run stores"),
        GateRunWrite::Inserted
    );
    id
}

fn seed_gate_run(db: &mut Database, label: &str) -> String {
    seed_run(db, label, &pass_context(), EffectiveSeverity::Pass)
}

/// The file plus a database on it, with both builds and one run. The `TempDb` is passed in rather than
/// returned, so a test can close the connection, reopen the same file, and still have it deleted.
fn seeded(file: &TempDb, label: &str) -> (Database, String) {
    let mut db = Database::open(file.path()).expect("open");
    seed_build(&db, TARGET_BUILD.as_str(), TARGET_SNAPSHOT);
    seed_build(&db, BASELINE_BUILD.as_str(), BASELINE_SNAPSHOT);
    let run = seed_gate_run(&mut db, label);
    (db, run)
}

/// The same file shape with a `REVIEW` run instead of a `PASS` one, for the two tests that need a finding
/// a person can actually accept.
fn review_seeded(file: &TempDb, label: &str) -> (Database, String) {
    let mut db = Database::open(file.path()).expect("open");
    seed_build(&db, TARGET_BUILD.as_str(), TARGET_SNAPSHOT);
    seed_build(&db, BASELINE_BUILD.as_str(), BASELINE_SNAPSHOT);
    let run = seed_run(&mut db, label, &review_context(), EffectiveSeverity::Review);
    (db, run)
}

/// The first acceptable finding of a stored run, which is what an acceptance has to name.
fn reviewable_finding(db: &Database, run: &str) -> String {
    db.gate_run_by_id(run)
        .expect("read the run")
        .expect("the run is stored")
        .acceptable_findings()
        .into_iter()
        .next()
        .expect("a REVIEW run has something to accept")
        .id
        .clone()
}

fn draft<'a>(
    release_id: &'a str,
    gate_run_id: &'a str,
    manifest_sha256: &'a str,
) -> ReleaseRecordDraft<'a> {
    ReleaseRecordDraft {
        release_id,
        build_id: TARGET_BUILD.as_str(),
        baseline_build_id: Some(BASELINE_BUILD.as_str()),
        gate_run_id,
        release_version: "1.4.2",
        manifest_sha256,
    }
}

/// A draft whose identity facts are all valid, so each malformed case overrides exactly one field.
fn valid_draft<'a>(
    gate_run_id: &'a str,
    manifest_sha256: &'a str,
    release_id: &'a str,
) -> ReleaseRecordDraft<'a> {
    draft(release_id, gate_run_id, manifest_sha256)
}

fn count(db: &Database, table: &str) -> i64 {
    db.connection()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .expect("count")
}

fn schema_version(db: &Database) -> i64 {
    db.connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .expect("version")
}

/// `strftime`'s UTC shape, checked without a date library.
fn looks_utc(stamp: &str) -> bool {
    let bytes = stamp.as_bytes();
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && stamp
            .bytes()
            .all(|c| c.is_ascii_digit() || b"-T:Z".contains(&c))
}

/// Turn a current database into the file a version 3 build left behind, by removing exactly what
/// migrations `0004` and `0005` add. `DROP TABLE` takes its indexes and triggers with it, and the two
/// `0005` columns have to go by name because `ADD COLUMN` is not idempotent: re-running the upgrade on a
/// file that still physically holds them would fail, which is the same reason a real v3 file never can.
fn step_down_to_v3(db: &Database) {
    db.connection()
        .execute_batch(
            "DROP INDEX idx_builds_created;
             ALTER TABLE symbols DROP COLUMN address_unknown;
             ALTER TABLE sections DROP COLUMN file_offset_unknown;
             DROP TABLE release_records;
             DELETE FROM schema_migrations WHERE version >= 4;",
        )
        .expect("stepped down");
}

// --------------------------------------------------------------------------- migration 0004

#[test]
fn the_schema_is_at_version_five_and_the_release_table_starts_empty() {
    let file = TempDb::new("fresh");
    let (db, _run) = seeded(&file, "fresh");

    assert_eq!(
        SCHEMA_VERSION, 5,
        "P4 raised the schema to version 4 and P5 to version 5"
    );
    assert_eq!(schema_version(&db), SCHEMA_VERSION);
    assert_eq!(count(&db, "release_records"), 0);

    let applied: Vec<(i64, String)> = {
        let mut stmt = db
            .connection()
            .prepare("SELECT version, name FROM schema_migrations ORDER BY version")
            .expect("select");
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("migrations")
            .map(|r| r.expect("row"))
            .collect()
    };
    assert_eq!(
        applied,
        vec![
            (1, "0001_initial".to_owned()),
            (2, "0002_evidence_keyed_by_build".to_owned()),
            (3, "0003_gate_history".to_owned()),
            (4, "0004_release_records".to_owned()),
            (5, "0005_unknown_reasons".to_owned()),
        ],
        "each stage contributes exactly one numbered step"
    );
}

#[test]
fn a_v1_database_reaches_v4_with_its_build_intact() {
    let file = TempDb::new("from-v1");
    {
        let conn = rusqlite::Connection::open(file.path()).expect("create");
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .expect("v1 schema");
        conn.execute_batch(
            "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL);
             INSERT INTO schema_migrations (version, name) VALUES (1, '0001_initial');
             INSERT INTO projects (id, name) VALUES ('proj-old', 'Old project');
             INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                 created_by_fwsight, state)
             VALUES ('build-old', 'proj-old', 'snap-old', 'p0-normalize-1', '0.1.0', 'COMPLETE');",
        )
        .expect("v1 bookkeeping and one build");
    }

    let db = Database::open(file.path()).expect("v1 upgrades straight to v4");
    assert_eq!(schema_version(&db), SCHEMA_VERSION);
    let builds: i64 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM builds WHERE id = 'build-old'",
            [],
            |r| r.get(0),
        )
        .expect("count");
    assert_eq!(builds, 1, "an upgrade must not lose the build it names");
    assert_eq!(count(&db, "release_records"), 0);
}

#[test]
fn a_v2_database_reaches_v4_with_its_evidence_still_keyed_by_build() {
    let file = TempDb::new("from-v2");
    {
        let conn = rusqlite::Connection::open(file.path()).expect("create");
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
            .expect("v1 schema");
        conn.execute_batch(include_str!(
            "../migrations/0002_evidence_keyed_by_build.sql"
        ))
        .expect("v2 schema");
        conn.execute_batch(
            "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL, name TEXT NOT NULL);
             INSERT INTO schema_migrations (version, name) VALUES (1, '0001_initial');
             INSERT INTO schema_migrations (version, name) VALUES (2, '0002_evidence_keyed_by_build');",
        )
        .expect("v2 bookkeeping");
    }

    let db = Database::open(file.path()).expect("v2 upgrades to v4");
    assert_eq!(schema_version(&db), SCHEMA_VERSION);
    let keyed: String = db
        .connection()
        .query_row(
            "SELECT name FROM pragma_table_info('evidence') WHERE name = 'build_id'",
            [],
            |r| r.get(0),
        )
        .expect("the v2 key survives the upgrade");
    assert_eq!(keyed, "build_id");
}

#[test]
fn a_v3_database_gains_the_release_table_and_keeps_every_gate_row() {
    // Write a real v3 file: run the Gate through `gate.rs`, then remove what 0004 adds.
    let file = TempDb::new("from-v3");
    let run;
    {
        let (db, existing) = seeded(&file, "from-v3");
        run = existing;
        step_down_to_v3(&db);
    }

    let db = Database::open(file.path()).expect("the v3 file upgrades to v4");
    assert_eq!(schema_version(&db), SCHEMA_VERSION);
    assert_eq!(
        count(&db, "release_records"),
        0,
        "the new table arrives empty"
    );
    let stored = db
        .gate_run_by_id(&run)
        .expect("read")
        .expect("a stored Gate run survives the upgrade it predates");
    assert!(!stored.findings.is_empty(), "and so do its findings");
    assert_eq!(stored.overall_effective_severity, EffectiveSeverity::Pass);
}

#[test]
fn an_acceptance_written_before_the_upgrade_is_still_there_after_it() {
    // §39's "accepted reviews preserved" is only a real check if the row exists before the upgrade, so
    // this run is a REVIEW one: the acceptance is written through `gate.rs`, the file is then stepped
    // down to version 3, and the same file is opened again.
    let file = TempDb::new("from-v3-acceptance");
    let run;
    let finding_id;
    {
        let (mut db, existing) = review_seeded(&file, "from-v3-acceptance");
        run = existing;
        finding_id = reviewable_finding(&db, &run);
        db.accept_review(
            &run,
            &finding_id,
            "release-owner",
            "kept for the upgrade test",
        )
        .expect("acceptance writes");
        step_down_to_v3(&db);
    }

    let db = Database::open(file.path()).expect("the v3 file upgrades to v4");
    assert_eq!(schema_version(&db), SCHEMA_VERSION);
    assert_eq!(
        count(&db, "release_records"),
        0,
        "the new table arrives empty"
    );

    let stored = db
        .gate_run_by_id(&run)
        .expect("read")
        .expect("the run survives the upgrade");
    assert!(
        stored
            .findings
            .iter()
            .any(|finding| finding.state == FindingState::Review && finding.id == finding_id),
        "and so does the finding, still at REVIEW, which an acceptance never rewrites"
    );

    let accepted = db.accepted_reviews_for_run(&run).expect("read acceptances");
    assert_eq!(accepted.len(), 1, "the acceptance survived too");
    assert_eq!(accepted[0].finding_id, finding_id);
    assert_eq!(accepted[0].actor, "release-owner");
    assert_eq!(accepted[0].reason, "kept for the upgrade test");
}

#[test]
fn a_database_from_the_future_is_refused_and_left_alone() {
    let file = TempDb::new("future");
    {
        let (db, _run) = seeded(&file, "future");
        db.connection()
            .execute(
                "INSERT INTO schema_migrations (version, name) VALUES (99, 'written_by_a_newer_build')",
                [],
            )
            .expect("stamp a future version");
    }

    let err = Database::open(file.path())
        .err()
        .expect("a schema this build does not understand is a hard refusal");
    assert!(
        matches!(
            err,
            StorageError::UnsupportedSchemaVersion {
                found: 99,
                supported
            } if supported == SCHEMA_VERSION
        ),
        "{err:?}"
    );
    assert_eq!(err.stable_code(), "ERR-STORAGE-4004");
    assert!(
        err.remediation().contains("Restore a backup"),
        "the refusal has to say what to do: {}",
        err.remediation()
    );

    // Refusing is not the same as resetting: nothing was deleted on the way out.
    let conn = rusqlite::Connection::open(file.path()).expect("reopen raw");
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
            r.get(0)
        })
        .expect("version");
    assert_eq!(version, 99, "no silent reset");
}

#[test]
fn a_failed_0004_leaves_the_v3_database_recoverable() {
    let file = TempDb::new("failed-0004");
    {
        let (db, _run) = seeded(&file, "failed-0004");
        step_down_to_v3(&db);
    }

    // Obstruct 0004 the way a half-applied upgrade would: the table it creates already exists, with a
    // shape that is not its own.
    {
        let conn = rusqlite::Connection::open(file.path()).expect("raw open");
        conn.execute("CREATE TABLE release_records (id TEXT PRIMARY KEY)", [])
            .expect("obstruction");
        conn.execute("INSERT INTO release_records (id) VALUES ('junk')", [])
            .expect("obstruction row");
    }

    let err = Database::open(file.path())
        .err()
        .expect("an obstructed migration must fail, not be skipped");
    assert!(
        matches!(err, StorageError::Migration { version: 4, .. }),
        "the failure must name version 4, got {err:?}"
    );

    {
        let conn = rusqlite::Connection::open(file.path()).expect("raw reopen");
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| {
                r.get(0)
            })
            .expect("version");
        assert_eq!(version, 3, "a rolled-back migration records nothing");
        let runs: i64 = conn
            .query_row("SELECT COUNT(*) FROM gate_runs", [], |r| r.get(0))
            .expect("count");
        assert_eq!(runs, 1, "the failure must not have touched Gate history");
        let junk: i64 = conn
            .query_row("SELECT COUNT(*) FROM release_records", [], |r| r.get(0))
            .expect("count");
        assert_eq!(
            junk, 1,
            "and it must not have deleted the obstruction either"
        );
    }

    // Clear the obstruction and the same file upgrades normally, with its history intact.
    {
        let conn = rusqlite::Connection::open(file.path()).expect("raw reopen");
        conn.execute("DROP TABLE release_records", [])
            .expect("drop the obstructing table");
    }
    let db = Database::open(file.path()).expect("the database is recoverable");
    assert_eq!(schema_version(&db), SCHEMA_VERSION);
    assert_eq!(count(&db, "release_records"), 0);
    assert_eq!(count(&db, "gate_runs"), 1, "and the history is still there");
}

// --------------------------------------------------------------------------- the record itself

#[test]
fn a_record_round_trips_and_carries_only_what_a_bundle_index_needs() {
    let file = TempDb::new("roundtrip");
    let (mut db, run) = seeded(&file, "roundtrip");
    let manifest = sha("manifest1");
    let id = release_id("one");

    assert_eq!(
        db.persist_release_record(&draft(&id, &run, &manifest))
            .expect("stores"),
        ReleaseRecordWrite::Inserted
    );

    let stored: StoredReleaseRecord = db
        .release_record_by_id(&id)
        .expect("read")
        .expect("the record is there");
    assert_eq!(stored.release_id, id);
    assert_eq!(stored.build_id, *TARGET_BUILD);
    assert_eq!(
        stored.baseline_build_id.as_deref(),
        Some(BASELINE_BUILD.as_str())
    );
    assert_eq!(stored.gate_run_id, run);
    assert_eq!(stored.release_version, "1.4.2");
    assert_eq!(stored.manifest_sha256, manifest);
    assert!(looks_utc(&stored.created_at), "{}", stored.created_at);

    // §37: nothing in the row says where the bundle went.
    let rendered = format!("{stored:?}");
    for forbidden in ["C:\\", "\\\\", ":/", "repo_root", "target"] {
        assert!(
            !rendered.contains(forbidden),
            "{forbidden} leaked into {rendered}"
        );
    }
}

#[test]
fn the_release_table_has_no_column_that_could_hold_a_path_or_a_blob() {
    let file = TempDb::new("columns");
    let (db, _run) = seeded(&file, "columns");
    let columns: Vec<(String, String)> = db
        .connection()
        .prepare("SELECT name, type FROM pragma_table_info('release_records')")
        .expect("pragma")
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("rows")
        .map(|r| r.expect("column"))
        .collect();
    let names: Vec<&str> = columns.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "id",
            "build_id",
            "baseline_build_id",
            "gate_run_id",
            "release_version",
            "manifest_sha256",
            "created_at",
        ],
        "the audit index is exactly these seven facts"
    );
    for (name, declared) in &columns {
        for forbidden in ["path", "dir", "root", "dest", "blob", "html", "json"] {
            assert!(
                !name.to_lowercase().contains(forbidden),
                "{name} would store a location or a payload"
            );
        }
        assert!(
            declared.as_str() != "BLOB" && declared.as_str() != "CLOB",
            "{name} is declared {declared}; a release record stores text only"
        );
    }
}

#[test]
fn the_same_release_recorded_twice_dedupes_instead_of_duplicating() {
    let file = TempDb::new("dedupe");
    let (mut db, run) = seeded(&file, "dedupe");
    let id = release_id("twice");
    let manifest = sha("manifest2");

    assert_eq!(
        db.persist_release_record(&draft(&id, &run, &manifest))
            .expect("first write"),
        ReleaseRecordWrite::Inserted
    );
    assert_eq!(
        db.persist_release_record(&draft(&id, &run, &manifest))
            .expect("second write"),
        ReleaseRecordWrite::AlreadyStored,
        "one deterministic release is one record"
    );
    assert_eq!(count(&db, "release_records"), 1);
}

#[test]
fn one_release_id_with_different_facts_is_an_invariant_not_an_update() {
    let file = TempDb::new("collision");
    let (mut db, run) = seeded(&file, "collision");
    let id = release_id("clash");
    let original = sha("manifest3");
    db.persist_release_record(&draft(&id, &run, &original))
        .expect("first write");

    // The same id over a different manifest: two bundles claiming one identity.
    let err = db
        .persist_release_record(&draft(&id, &run, &sha("manifest4")))
        .expect_err("a colliding record is refused");
    assert!(matches!(err, StorageError::Invariant { .. }), "{err:?}");
    assert_eq!(err.stable_code(), "ERR-INTERNAL-9002");

    // A different version under the same id is refused the same way.
    let other = release_id("clash");
    let err = db
        .persist_release_record(&ReleaseRecordDraft {
            release_id: &other,
            build_id: TARGET_BUILD.as_str(),
            baseline_build_id: Some(BASELINE_BUILD.as_str()),
            gate_run_id: &run,
            release_version: "1.4.3",
            manifest_sha256: &original,
        })
        .expect_err("the version is part of the identity");
    assert!(matches!(err, StorageError::Invariant { .. }), "{err:?}");

    let stored = db
        .release_record_by_id(&id)
        .expect("read")
        .expect("the original stands untouched");
    assert_eq!(stored.manifest_sha256, original);
    assert_eq!(stored.release_version, "1.4.2");
    assert_eq!(count(&db, "release_records"), 1);
}

#[test]
fn a_stored_record_refuses_update_and_refuses_delete() {
    let file = TempDb::new("immutable");
    let (mut db, run) = seeded(&file, "immutable");
    let id = release_id("frozen");
    db.persist_release_record(&draft(&id, &run, &sha("manifest5")))
        .expect("stores");

    let err = db
        .connection()
        .execute(
            "UPDATE release_records SET release_version = '9.9.9' WHERE id = ?1",
            params![id],
        )
        .expect_err("a published release record is not editable");
    assert!(
        err.to_string().contains("immutable"),
        "the trigger has to explain itself: {err}"
    );

    let err = db
        .connection()
        .execute("DELETE FROM release_records WHERE id = ?1", params![id])
        .expect_err("a published release record is not deletable");
    assert!(err.to_string().contains("audit row"), "{err}");

    assert_eq!(count(&db, "release_records"), 1);
    let stored = db
        .release_record_by_id(&id)
        .expect("read")
        .expect("still there");
    assert_eq!(stored.release_version, "1.4.2", "and unchanged");
}

#[test]
fn the_run_and_the_build_a_record_names_must_exist() {
    let file = TempDb::new("foreign-keys");
    let (mut db, run) = seeded(&file, "foreign-keys");
    let manifest = sha("manifest6");

    let err = db
        .persist_release_record(&ReleaseRecordDraft {
            release_id: &release_id("fk1"),
            build_id: "build-does-not-exist",
            baseline_build_id: None,
            gate_run_id: &run,
            release_version: "1.4.2",
            manifest_sha256: &manifest,
        })
        .expect_err("a record must point at a stored build");
    assert!(matches!(err, StorageError::Write { .. }), "{err:?}");

    let err = db
        .persist_release_record(&ReleaseRecordDraft {
            release_id: &release_id("fk2"),
            build_id: TARGET_BUILD.as_str(),
            baseline_build_id: None,
            gate_run_id: &run_id("never-ran"),
            release_version: "1.4.2",
            manifest_sha256: &manifest,
        })
        .expect_err("a record must point at a stored Gate run");
    assert!(matches!(err, StorageError::Write { .. }), "{err:?}");

    assert_eq!(
        count(&db, "release_records"),
        0,
        "a refused write leaves no partial row"
    );
}

#[test]
fn a_release_is_never_its_own_baseline_and_a_baseline_is_optional() {
    let file = TempDb::new("self-baseline");
    let (mut db, run) = seeded(&file, "self-baseline");
    let manifest = sha("manifest7");

    let err = db
        .persist_release_record(&ReleaseRecordDraft {
            release_id: &release_id("self"),
            build_id: TARGET_BUILD.as_str(),
            baseline_build_id: Some(TARGET_BUILD.as_str()),
            gate_run_id: &run,
            release_version: "1.4.2",
            manifest_sha256: &manifest,
        })
        .expect_err("Core refuses that comparison, and so does the row");
    assert!(matches!(err, StorageError::Write { .. }), "{err:?}");
    assert_eq!(count(&db, "release_records"), 0);

    // A record with no baseline at all is the ordinary case, and it stores.
    let id = release_id("no-baseline");
    assert_eq!(
        db.persist_release_record(&ReleaseRecordDraft {
            release_id: &id,
            build_id: TARGET_BUILD.as_str(),
            baseline_build_id: None,
            gate_run_id: &run,
            release_version: "1.4.2",
            manifest_sha256: &manifest,
        })
        .expect("stores"),
        ReleaseRecordWrite::Inserted
    );
    assert_eq!(
        db.release_record_by_id(&id)
            .expect("read")
            .expect("stored")
            .baseline_build_id,
        None
    );
}

#[test]
fn malformed_release_facts_are_refused_before_any_write() {
    let file = TempDb::new("malformed");
    let (mut db, run) = seeded(&file, "malformed");
    let good = sha("manifest8");
    let id = release_id("malformed");

    for bad_id in [
        "not a release id at all".to_owned(),
        // A Gate run id is the same shape family and the wrong one: 72 characters of `gate-` plus hex is
        // not a release, and mixing them up would put a verdict in the release history.
        run_id("a-gate-id-is-not-a-release-id"),
        release_id("short-tail")
            .chars()
            .take(20)
            .collect::<String>(),
    ] {
        let err = db
            .persist_release_record(&valid_draft(&run, &good, &bad_id))
            .expect_err("`{bad_id}` is not a release id");
        assert!(matches!(err, StorageError::Invariant { .. }), "{err:?}");
    }

    // A manifest digest is a digest: the same shape rule the column enforces.
    for bad_digest in [good.to_uppercase(), "abc".to_owned(), String::new()] {
        let err = db
            .persist_release_record(&valid_draft(&run, &bad_digest, &id))
            .expect_err("a digest in the wrong spelling verifies nothing");
        assert!(matches!(err, StorageError::Invariant { .. }), "{err:?}");
    }

    // The version reaches a directory name, so the path rules apply to it here as well.
    for bad_version in ["", "   ", "../1.4.2", "1.4.2\n", "a\\b", &"9".repeat(65)] {
        let err = db
            .persist_release_record(&ReleaseRecordDraft {
                release_id: &id,
                build_id: TARGET_BUILD.as_str(),
                baseline_build_id: None,
                gate_run_id: &run,
                release_version: bad_version,
                manifest_sha256: &good,
            })
            .expect_err("`{bad_version}` cannot be a release version");
        assert!(matches!(err, StorageError::Invariant { .. }), "{err:?}");
    }

    assert_eq!(count(&db, "release_records"), 0, "and nothing was written");
}

// --------------------------------------------------------------------------- the schema, on its own

#[test]
fn the_constraints_hold_for_a_hand_written_insert_too() {
    // The API is not the only writer. `0004`'s CHECK constraints are the reason a debugging session with
    // the sqlite3 CLI cannot invent a release record either.
    let file = TempDb::new("constraints");
    let (db, run) = seeded(&file, "constraints");
    let conn = db.connection();
    let good = sha("manifest9");

    // Each case differs from the accepted rows at the end by exactly one value, and each refusal is
    // asserted as a *constraint* failure rather than as "some error", so a CHECK that fires for the wrong
    // reason cannot quietly stand in for the one under test.
    let refused: Vec<(&str, String, &'static str, String)> = vec![
        (
            "a short id",
            "release-abc".to_owned(),
            "1.4.2",
            good.clone(),
        ),
        (
            "an id with an uppercase tail",
            format!("release-{}", good.to_uppercase()),
            "1.4.2",
            good.clone(),
        ),
        (
            "a version that is only whitespace",
            release_id("c2"),
            "   ",
            good.clone(),
        ),
        (
            "a version carrying a slash",
            release_id("c3"),
            "../evil",
            good.clone(),
        ),
        (
            "a version carrying a backslash",
            release_id("c4"),
            "a\\b",
            good.clone(),
        ),
        (
            "a version carrying a newline",
            release_id("c6"),
            "1.4.2\n",
            good.clone(),
        ),
        (
            "an uppercase manifest digest",
            release_id("u1"),
            "1.4.2",
            good.to_uppercase(),
        ),
        (
            "a short manifest digest",
            release_id("u2"),
            "1.4.2",
            "abc".to_owned(),
        ),
    ];

    for (label, id, version, digest) in &refused {
        let err = conn
            .execute(
                "INSERT INTO release_records
                     (id, build_id, gate_run_id, release_version, manifest_sha256)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, &*TARGET_BUILD, run, version, digest],
            )
            .expect_err(label);
        assert_constraint(label, &err);
    }

    // A release compared against itself, refused by the CHECK.
    let err = conn
        .execute(
            "INSERT INTO release_records
                 (id, build_id, baseline_build_id, gate_run_id, release_version, manifest_sha256)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                release_id("same"),
                &*TARGET_BUILD,
                &*TARGET_BUILD,
                run,
                "1.4.2",
                good
            ],
        )
        .expect_err("a build cannot be its own baseline");
    assert_constraint("the self-baseline CHECK", &err);

    // And well-formed rows do insert, which is what makes the refusals above meaningful. Two of them,
    // because `baseline_build_id` is optional and both shapes have to be writable.
    conn.execute(
        "INSERT INTO release_records
             (id, build_id, gate_run_id, release_version, manifest_sha256)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![release_id("c5"), &*TARGET_BUILD, run, "1.4.2", good],
    )
    .expect("a well-formed hand-written record is accepted");
    conn.execute(
        "INSERT INTO release_records
             (id, build_id, baseline_build_id, gate_run_id, release_version, manifest_sha256)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            release_id("c7"),
            &*TARGET_BUILD,
            &*BASELINE_BUILD,
            run,
            "1.4.2",
            good
        ],
    )
    .expect("a well-formed record with a distinct baseline is accepted");
    assert_eq!(count(&db, "release_records"), 2);
}

/// The refusal must come from a constraint, not from a mistyped statement or a missing table.
fn assert_constraint(label: &str, err: &rusqlite::Error) {
    assert_eq!(
        err.sqlite_error_code(),
        Some(rusqlite::ErrorCode::ConstraintViolation),
        "{label} failed for a reason other than a constraint: {err}"
    );
}

#[test]
fn accepting_a_review_after_a_release_was_recorded_leaves_the_record_alone() {
    // §38's immutability belongs to the release row. The Gate's own audit rules keep working beside it,
    // and an acceptance written afterwards must not disturb what was already published — which is the
    // same property `0003`'s triggers give a finding, seen from the other side.
    //
    // The run here is deliberately a REVIEW one: §8 (only a PASS may be packaged) is enforced by
    // `ReleaseModel::validated` in Core, and this test is about what the storage layer does *not* decide.
    let file = TempDb::new("accept-after");
    let (mut db, run) = review_seeded(&file, "accept-after");
    let id = release_id("before-accept");
    let manifest = sha("manifest10");
    db.persist_release_record(&draft(&id, &run, &manifest))
        .expect("stores");

    let finding_id = reviewable_finding(&db, &run);
    db.accept_review(&run, &finding_id, "release-owner", "later evidence arrived")
        .expect("acceptance writes");

    let stored = db
        .release_record_by_id(&id)
        .expect("read")
        .expect("the record is exactly as published");
    assert_eq!(stored.manifest_sha256, manifest);
    assert_eq!(stored.gate_run_id, run);
    assert_eq!(count(&db, "release_records"), 1, "and still only one");

    // The acceptance landed where it belongs and nowhere else.
    let accepted = db.accepted_reviews_for_run(&run).expect("read acceptances");
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].finding_id, finding_id);
}
