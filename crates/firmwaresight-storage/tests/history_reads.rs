//! The History read layer: three bounded, scoped, ordered reads over facts the application already
//! wrote.
//!
//! P5 adds no table for History (`P5_VALIDATION/P5_MIGRATION_DECISION.md` records why none was
//! needed), so every value a row shows was persisted by Analyze, by the Gate or by a Bundle export.
//! What this layer is responsible for is therefore entirely about *reading*: a page that is bounded
//! whatever the caller asked for, a scope the caller must name, an order that does not depend on
//! SQLite's visit order, and a host path that never leaves the crate.
//!
//! Five rules, stated once because together they are the contract:
//!
//! - **Nothing is opened from disk.** A stored row stays readable after the artifact has moved or
//!   been deleted; one test proves it by deleting the file it just imported.
//! - **Only a complete import is history.** A build still in `IMPORTING`, or one that failed, is not
//!   a snapshot and must not be presented as one (`04_TECH/15` §6).
//! - **No host path crosses.** `artifacts.path` is reduced to its file name here, per `AGENTS.md` 7.
//! - **Unknown stays unknown.** A footprint stored without a byte count comes back with no byte
//!   count, never as `0` (`AGENTS.md` 8).
//! - **A filter is text, not LIKE syntax.** `%` and `_` typed by a person match themselves.
//!
//! Gate dispositions and finding counts are read back through the same closed parsers the Gate
//! history reader uses, so a stored word this build does not recognize fails the read closed rather
//! than being filed under `UNKNOWN` — the state ADR-0023 reserves for evidence that never arrived.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_core::domain::gate::{
    EffectiveSeverity, GateArtifactFact, GateBudgetFact, GateContext, GateEvaluation, GateGitFacts,
    GateGrowthFacts, GateMemoryFacts, GatePolicy, GateStateCounts, GateUnknownEvidence,
};
use firmwaresight_core::domain::identity::{ArtifactKind, Fact};
use firmwaresight_storage::{
    CompareCandidate, DEFAULT_CANDIDATE_LIMIT, DEFAULT_HISTORY_LIMIT, Database, GateRunDraft,
    HistoryGateRun, HistoryQuery, HistoryReleaseRecord, MAX_HISTORY_LIMIT, Page,
    ReleaseRecordDraft, StorageError,
};
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
            "fwsight-history-{label}-{}-{:x}.sqlite",
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

    fn open(&self) -> Database {
        Database::open(self.path()).expect("database opens")
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

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-history-dir-{label}-{}-{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.path());
    }
}

/// Analyze one committed fixture side inside a scratch directory, so the path the importer stores is
/// a temporary directory the assertions below can prove never reaches the read layer.
fn analyze_fixture_copy(dir: &TempDir, side: &str) -> BuildSnapshot {
    let source = repo_root().join(format!("fixtures/elf/p2-diff/{side}"));
    let elf = dir.path().join(format!("{side}.elf"));
    let map = dir.path().join(format!("{side}.map"));
    std::fs::copy(source.join("firmware.elf"), &elf).expect("copy elf");
    std::fs::copy(source.join("firmware.map"), &map).expect("copy map");
    pipeline::analyze(&AnalysisRequest::new(&elf).with_map(&map))
        .unwrap_or_else(|err| panic!("the {side} fixture must analyze: {err}"))
        .snapshot
        .clone()
}

/// Analyze a committed fixture where it lies, so its stored path carries the repository's own
/// directory names.
fn analyze_repo_fixture(relative: &str) -> BuildSnapshot {
    let root = repo_root();
    let elf = root.join(format!("fixtures/elf/{relative}/firmware.elf"));
    let map = root.join(format!("fixtures/elf/{relative}/firmware.map"));
    pipeline::analyze(&AnalysisRequest::new(&elf).with_map(&map))
        .unwrap_or_else(|err| panic!("{relative} must analyze: {err}"))
        .snapshot
        .clone()
}

fn import(db: &mut Database, project: &str, snapshot: &BuildSnapshot) -> String {
    db.import_snapshot(project, "History test project", snapshot)
        .unwrap_or_else(|err| panic!("import into {project}: {err}"))
}

/// Restamp a build row. `builds` has no immutability trigger, and a test cannot wait a month
/// between two imports; `gate_runs` and `release_records` do refuse an UPDATE, so their stored
/// times are never touched here and the tests that need two of them a second apart wait for one.
fn set_build_stored_at(db: &Database, id: &str, stamp: &str) {
    db.connection()
        .execute(
            "UPDATE builds SET created_at = ?2 WHERE id = ?1",
            params![id, stamp],
        )
        .expect("stamp stored time");
}

/// What SQLite wrote for one stored row, read back from the column the read is supposed to surface.
fn stored_time_of(db: &Database, table: &str, id: &str) -> String {
    db.connection()
        .query_row(
            &format!("SELECT created_at FROM {table} WHERE id = ?1"),
            params![id],
            |row| row.get(0),
        )
        .expect("the row this test just wrote")
}

/// A 64-character lowercase hex stand-in for one test label, shaped like a real digest so the
/// schema's digest CHECKs are exercised rather than skipped.
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

fn run_id(tag: &str) -> String {
    format!("gate-{}", hex64(tag))
}

fn release_id(tag: &str) -> String {
    format!("release-{}", hex64(tag))
}

/// A context the Gate evaluates to `PASS`: a clean workspace, no configured budget to breach, no
/// `[version]` policy to fail and no baseline to grow against, which is the configuration Core's own
/// test `an_unconfigured_policy_is_all_not_applicable_and_passes` documents. Release records only
/// point at runs like this one, because a release qualified by a run that did not pass would be an
/// invalid fact invented to fill a count (§49).
fn passing_context(snapshot_id: &str) -> GateContext {
    GateContext {
        snapshot_id: snapshot_id.to_owned(),
        artifacts: vec![GateArtifactFact {
            kind: ArtifactKind::Elf,
            sha256: Fact::known(hex64("artifact")),
            byte_size: 4096,
        }],
        attachments: Vec::new(),
        memory: Some(GateMemoryFacts {
            nonvolatile: Some(GateBudgetFact::exact(
                2048,
                "every allocatable section attributed",
                "evidence:ev-flash",
            )),
            runtime_ram: Some(GateBudgetFact::exact(
                512,
                "every allocatable section attributed",
                "evidence:ev-ram",
            )),
        }),
        git: GateGitFacts {
            available: true,
            head_commit: Fact::known(hex64("commit")),
            exact_tag: Fact::unknown("HEAD carries no tag"),
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

/// Store one real Gate evaluation of `build_id`, with the findings Core produced, and give it a
/// recorded storage time.
fn persist_run(db: &mut Database, tag: &str, build_id: &str, snapshot_id: &str) -> GateEvaluation {
    let id = run_id(tag);
    let evaluation = passing_context(snapshot_id).evaluate(&id);
    db.persist_gate_run(&GateRunDraft {
        run_id: &id,
        build_id,
        baseline_build_id: None,
        policy_sha256: &hex64("policy"),
        evaluation: &evaluation,
        attachments: &[],
    })
    .unwrap_or_else(|err| panic!("persist gate run {tag}: {err}"));
    evaluation
}

/// Record a published release whose qualifying run actually passed, and return its id.
fn persist_release(
    db: &mut Database,
    tag: &str,
    build_id: &str,
    run: &GateEvaluation,
    number: u32,
) -> String {
    assert_eq!(
        run.overall_effective_severity,
        EffectiveSeverity::Pass,
        "a release record may only qualify a PASS run"
    );
    let id = release_id(tag);
    db.persist_release_record(&ReleaseRecordDraft {
        release_id: &id,
        build_id,
        baseline_build_id: None,
        gate_run_id: &run.run_id,
        release_version: &format!("1.{number}.0"),
        manifest_sha256: &hex64(&format!("manifest-{tag}")),
    })
    .unwrap_or_else(|err| panic!("persist release {tag}: {err}"));
    id
}

fn stored_snapshot_id(db: &Database, build_id: &str) -> String {
    db.connection()
        .query_row(
            "SELECT snapshot_id FROM builds WHERE id = ?1",
            params![build_id],
            |row| row.get(0),
        )
        .expect("the build this test just imported")
}

fn query(projects: &[&str], offset: i64, limit: i64, filter: Option<&str>) -> HistoryQuery {
    HistoryQuery {
        project_ids: projects.iter().map(|id| (*id).to_owned()).collect(),
        offset,
        limit,
        filter: filter.map(str::to_owned),
    }
}

fn build_rows(
    db: &Database,
    projects: &[&str],
    offset: i64,
    limit: i64,
    filter: Option<&str>,
) -> Page<CompareCandidate> {
    db.list_history_builds(&query(projects, offset, limit, filter))
        .expect("history build page")
}

fn run_rows(
    db: &Database,
    projects: &[&str],
    offset: i64,
    limit: i64,
    filter: Option<&str>,
) -> Page<HistoryGateRun> {
    db.list_history_gate_runs(&query(projects, offset, limit, filter))
        .expect("history gate page")
}

fn release_rows(
    db: &Database,
    projects: &[&str],
    offset: i64,
    limit: i64,
    filter: Option<&str>,
) -> Page<HistoryReleaseRecord> {
    db.list_history_releases(&query(projects, offset, limit, filter))
        .expect("history release page")
}

/// A database holding two builds under the local project and one under a foreign project, with
/// distinct stored times. The newest local row is `target.elf`, the older is `base.elf`, and the
/// foreign row is the dual-region fixture, whose stored path is inside this repository.
fn seeded(label: &str) -> (TempDb, Database) {
    let file = TempDb::new(label);
    let mut db = file.open();
    {
        let scratch = TempDir::new(label);
        let base = analyze_fixture_copy(&scratch, "base");
        let target = analyze_fixture_copy(&scratch, "target");
        // A build is keyed by its content (`builds.id` is `build-<snapshot id>`), so the foreign
        // project cannot reuse either of those snapshots; the dual-region fixture is a third.
        let foreign = analyze_repo_fixture("p0-dual-region");

        let base_build = import(&mut db, LOCAL, &base);
        let target_build = import(&mut db, LOCAL, &target);
        let foreign_build = import(&mut db, OTHER, &foreign);
        set_build_stored_at(&db, &base_build, "2026-08-01T00:00:00Z");
        set_build_stored_at(&db, &target_build, "2026-09-01T00:00:00Z");
        set_build_stored_at(&db, &foreign_build, "2026-10-01T00:00:00Z");
    }
    (file, db)
}

#[test]
fn history_build_rows_are_scoped_and_newest_first() {
    let (_file, db) = seeded("build-order");

    let page = build_rows(&db, &[LOCAL], 0, 10, None);
    let names: Vec<&str> = page.rows.iter().map(|row| row.file_name.as_str()).collect();
    assert_eq!(
        page.total, 2,
        "two builds were imported under the local project"
    );
    assert_eq!(page.offset, 0);
    assert_eq!(page.limit, 10);
    assert_eq!(
        names,
        vec!["target.elf", "base.elf"],
        "newest stored time first"
    );
    assert_eq!(page.next_offset, None, "one page held all of it");

    // The foreign build is the newest row in the database and must still be absent from a local page.
    let foreign = build_rows(&db, &[OTHER], 0, 10, None);
    assert_eq!(foreign.total, 1);
    assert_eq!(foreign.rows[0].file_name, "firmware.elf");

    // A caller that names no project gets nothing, not everything.
    let unscoped = build_rows(&db, &[], 0, 10, None);
    assert_eq!(unscoped.total, 0);
    assert!(unscoped.rows.is_empty());
}

#[test]
fn history_page_size_is_a_server_side_ceiling() {
    let (_file, db) = seeded("ceiling");

    let huge = build_rows(&db, &[LOCAL], 0, i64::from(u32::MAX), None);
    assert_eq!(
        huge.limit, MAX_HISTORY_LIMIT,
        "an absurd page size is clamped, not honoured"
    );
    assert_eq!(huge.rows.len(), 2, "and there were only two rows to give");

    let none = build_rows(&db, &[LOCAL], 0, 0, None);
    assert_eq!(none.limit, DEFAULT_HISTORY_LIMIT, "zero means 'not asked'");

    let negative = build_rows(&db, &[LOCAL], -5, 5, None);
    assert_eq!(
        negative.offset, 0,
        "a negative offset is not a page before the first"
    );

    let far = build_rows(&db, &[LOCAL], 500, 5, None);
    assert_eq!(far.rows.len(), 0, "an offset past the end is an empty page");
    assert_eq!(far.total, 2, "and still reports the true total");

    // The same bounds answer for the other two tables, so one screen never shows three page sizes.
    assert_eq!(
        run_rows(&db, &[LOCAL], 0, i64::from(u32::MAX), None).limit,
        MAX_HISTORY_LIMIT
    );
    assert_eq!(
        release_rows(&db, &[LOCAL], 0, 0, None).limit,
        DEFAULT_HISTORY_LIMIT
    );
    assert_eq!(DEFAULT_HISTORY_LIMIT, DEFAULT_CANDIDATE_LIMIT);
}

#[test]
fn a_build_still_in_importing_is_not_history() {
    let (_file, db) = seeded("importing");
    let total = build_rows(&db, &[LOCAL], 0, 10, None).total;

    db.connection()
        .execute(
            "UPDATE builds SET state = 'IMPORTING' WHERE id = (SELECT MIN(id) FROM builds
                      WHERE project_id = 'local-desktop')",
            [],
        )
        .expect("demote one build to a half-written import");

    let page = build_rows(&db, &[LOCAL], 0, 10, None);
    assert_eq!(
        page.total,
        total - 1,
        "the incomplete build left the history"
    );
    assert_eq!(page.rows.len(), 1, "and its row is not in the page either");
}

#[test]
fn history_build_rows_survive_the_artifact_being_deleted() {
    let file = TempDb::new("deleted-source");
    let mut db = file.open();
    let scratch = TempDir::new("deleted-source");

    let elf = scratch.path().join("disposable.elf");
    let map = scratch.path().join("disposable.map");
    let source = repo_root().join("fixtures/elf/p2-diff/base");
    std::fs::copy(source.join("firmware.elf"), &elf).expect("copy elf");
    std::fs::copy(source.join("firmware.map"), &map).expect("copy map");
    let snapshot = pipeline::analyze(&AnalysisRequest::new(&elf).with_map(&map))
        .expect("the fixture analyzes")
        .snapshot
        .clone();
    let build = import(&mut db, LOCAL, &snapshot);
    drop(db);

    std::fs::remove_file(&elf).expect("delete the artifact the build came from");
    std::fs::remove_file(&map).expect("delete the MAP");

    let db = file.open();
    let page = build_rows(&db, &[LOCAL], 0, 10, None);
    assert_eq!(page.total, 1, "the stored row is the history now");
    assert_eq!(page.rows[0].build_id, build);
    assert_eq!(page.rows[0].file_name, "disposable.elf");
    assert_eq!(page.rows[0].sha256.len(), 64);
    assert!(page.rows[0].byte_size > 0);
    assert!(!page.rows[0].architecture.is_empty());
    assert!(!page.rows[0].imported_at.is_empty());
}

#[test]
fn history_rows_carry_a_file_name_and_never_a_path() {
    let (_file, mut db) = seeded("path-leak");
    // The foreign build was analyzed where it lies, so its stored path contains this repository's
    // own directory names. Reading that project makes the assertions below bite.
    let leak = build_rows(&db, &[OTHER], 0, 10, None);
    assert_eq!(leak.total, 1, "the fixture-pathed build");
    let row = &leak.rows[0];
    assert_eq!(row.file_name, "firmware.elf", "the name, nothing else");
    assert!(
        !row.file_name.contains('/') && !row.file_name.contains('\\'),
        "a file name reached the read layer with a separator: {}",
        row.file_name
    );
    let rendered = format!("{row:?}");
    assert!(
        !rendered.contains("fixtures"),
        "a build row carried a directory: {rendered}"
    );

    // A Gate row and a release row about the same build name it the same way.
    let build = row.build_id.clone();
    let snapshot = stored_snapshot_id(&db, &build);
    let run = persist_run(&mut db, "p", &build, &snapshot);
    persist_release(&mut db, "p", &build, &run, 0);

    let rendered = format!("{:?}", run_rows(&db, &[OTHER], 0, 10, None).rows);
    assert!(
        !rendered.contains("fixtures"),
        "a gate row carried a path: {rendered}"
    );
    let rendered = format!("{:?}", release_rows(&db, &[OTHER], 0, 10, None).rows);
    assert!(
        !rendered.contains("fixtures"),
        "a release row carried a path: {rendered}"
    );
}

#[test]
fn an_unknown_footprint_reads_as_unknown_and_never_as_zero() {
    let (_file, db) = seeded("unknown-footprint");
    let rows = build_rows(&db, &[LOCAL], 0, 10, None);
    let target = rows.rows[0].build_id.clone();

    // Write the footprint the way an unmeasured one is actually stored: the state column carries
    // `unknown` and the byte column carries nothing.
    db.connection()
        .execute(
            "UPDATE memory_footprints SET nonvolatile_state = 'unknown', nonvolatile_bytes = NULL
              WHERE build_id = ?1",
            params![target.clone()],
        )
        .expect("store an unknown footprint");

    let after = build_rows(&db, &[LOCAL], 0, 10, None);
    let row = after
        .rows
        .iter()
        .find(|row| row.build_id == target)
        .expect("the row still reads back");
    assert_eq!(row.nonvolatile.state, "unknown");
    assert_eq!(
        row.nonvolatile.bytes, None,
        "a footprint with no byte count keeps having no byte count"
    );
}

#[test]
fn a_history_filter_is_text_and_not_like_syntax() {
    let (_file, db) = seeded("filter");
    let rows = build_rows(&db, &[LOCAL], 0, 10, None);
    let digest = rows.rows[0].sha256.clone();

    assert_eq!(
        build_rows(&db, &[LOCAL], 0, 10, Some(&digest[..16])).total,
        1
    );

    // `%` and `_` are LIKE syntax. Typed by a person they are literal text, so a bare percent must
    // find the rows that carry one rather than every row in the table.
    assert_eq!(
        build_rows(&db, &[LOCAL], 0, 10, Some("%")).total,
        0,
        "a percent is not a match-all"
    );
    assert_eq!(
        build_rows(&db, &[LOCAL], 0, 10, Some("build_")).total,
        0,
        "an underscore does not stand for one character"
    );
    assert_eq!(
        build_rows(&db, &[LOCAL], 0, 10, Some("")).total,
        2,
        "an empty filter is no filter at all"
    );

    let absent = build_rows(&db, &[LOCAL], 0, 10, Some("no-such-firmware"));
    assert_eq!(absent.total, 0);
    assert_eq!(absent.next_offset, None);
}

#[test]
fn a_filter_searches_identity_columns_and_never_a_directory() {
    let (_file, db) = seeded("filter-columns");
    let all = build_rows(&db, &[LOCAL], 0, 10, None);
    let row = &all.rows[0];

    assert_eq!(
        build_rows(&db, &[LOCAL], 0, 10, Some(&row.sha256[..12])).total,
        1,
        "a stored digest is searchable"
    );
    assert_eq!(
        build_rows(&db, &[LOCAL], 0, 10, Some(&row.build_id)).total,
        1,
        "a build id is searchable"
    );
    assert_eq!(
        build_rows(&db, &[LOCAL], 0, 10, Some(&row.snapshot_id[..12])).total,
        1,
        "a snapshot id is searchable"
    );
    assert!(
        build_rows(&db, &[LOCAL], 0, 10, Some(&row.architecture)).total >= 1,
        "the recorded architecture is searchable: {}",
        row.architecture
    );

    // The directory the artifact was really read from is not searchable, because no read predicate
    // and no displayed field carries it. A match here would be a path oracle in the filter box,
    // which is what prompt §16 and §44 forbid even when the row itself shows only a name.
    let scratch = scratch_marker("filter-columns");
    assert_eq!(build_rows(&db, &[LOCAL], 0, 10, Some(&scratch)).total, 0);
    assert_eq!(
        build_rows(&db, &[OTHER], 0, 10, Some("fixtures")).total,
        0,
        "the foreign build's stored directory matched, so the filter reads paths"
    );
}

fn scratch_marker(label: &str) -> String {
    format!("fwsight-history-dir-{label}")
}

#[test]
fn gate_rows_report_the_stored_disposition_and_the_counts_by_state() {
    let (_file, mut db) = seeded("gate-row");
    let builds = build_rows(&db, &[LOCAL], 0, 10, None);
    let newest = &builds.rows[0];
    let evaluation = persist_run(&mut db, "1", &newest.build_id, &newest.snapshot_id);

    let page = run_rows(&db, &[LOCAL], 0, 10, None);
    assert_eq!(page.total, 1);
    let row = &page.rows[0];
    assert_eq!(row.run_id, evaluation.run_id);
    assert_eq!(row.build_id, newest.build_id);
    assert_eq!(row.file_name, newest.file_name, "the row says which build");
    assert_eq!(row.baseline_build_id, None, "this run named no baseline");
    assert_eq!(row.baseline_file_name, None);
    assert_eq!(
        row.stored_at,
        stored_time_of(&db, "gate_runs", &row.run_id),
        "the row surfaces the audit time the run was stored at"
    );
    assert_eq!(
        row.disposition,
        evaluation.overall_effective_severity.as_str(),
        "the disposition is the stored one, not a recomputation"
    );

    let expected = GateStateCounts::of(&evaluation.findings);
    assert_eq!(row.counts.pass, expected.pass);
    assert_eq!(row.counts.review, expected.review);
    assert_eq!(row.counts.block, expected.block);
    assert_eq!(row.counts.unknown, expected.unknown);
    assert_eq!(row.counts.not_applicable, expected.not_applicable);
    assert_eq!(
        row.counts.pass
            + row.counts.review
            + row.counts.block
            + row.counts.unknown
            + row.counts.not_applicable,
        evaluation.findings.len(),
        "every stored finding lands in exactly one bucket"
    );
}

/// The order a bounded read must produce, computed in Rust from the stored pairs so the assertion
/// does not ask SQLite to confirm its own `ORDER BY`: newest stored time first, then identity.
fn ordered_by_time_then_identity(pairs: &[(String, String)]) -> Vec<String> {
    let mut sorted = pairs.to_vec();
    sorted.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    sorted.into_iter().map(|(_, id)| id).collect()
}

fn stored_pairs(db: &Database, table: &str, id_column: &str) -> Vec<(String, String)> {
    let mut stmt = db
        .connection()
        .prepare(&format!("SELECT created_at, {id_column} FROM {table}"))
        .expect("read the stored times this test wrote");
    stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })
    .expect("query")
    .collect::<Result<Vec<_>, _>>()
    .expect("rows")
}

#[test]
fn gate_rows_follow_the_stored_order_of_time_then_identity() {
    let (_file, mut db) = seeded("gate-order");
    let builds = build_rows(&db, &[LOCAL], 0, 10, None);

    let first = persist_run(
        &mut db,
        "1",
        &builds.rows[0].build_id,
        &builds.rows[0].snapshot_id,
    );
    let second = persist_run(
        &mut db,
        "2",
        &builds.rows[1].build_id,
        &builds.rows[1].snapshot_id,
    );
    let expected = ordered_by_time_then_identity(&stored_pairs(&db, "gate_runs", "id"));
    assert_eq!(expected.len(), 2);

    let page = run_rows(&db, &[LOCAL], 0, 1, None);
    assert_eq!(page.total, 2);
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.limit, 1);
    assert_eq!(
        page.next_offset,
        Some(1),
        "the page reports where to go next"
    );

    let mut seen = vec![page.rows[0].run_id.clone()];
    let next = run_rows(&db, &[LOCAL], 1, 1, None);
    assert_eq!(next.next_offset, None, "the last page says it is the last");
    seen.push(next.rows[0].run_id.clone());
    assert_eq!(
        seen, expected,
        "two bounded pages reproduce the stored order"
    );
    assert!(seen.contains(&first.run_id) && seen.contains(&second.run_id));

    // Where two runs were stored in the same second, the id is the tie-break, so the order cannot
    // depend on which row SQLite happened to visit first.
    let same_second = stored_time_of(&db, "gate_runs", &first.run_id)
        == stored_time_of(&db, "gate_runs", &second.run_id);
    if same_second {
        let lower = std::cmp::min(&first.run_id, &second.run_id).clone();
        assert_eq!(seen[0], lower, "same second, lower id first");
    }
}

/// `gate_runs` and `release_records` carry UPDATE triggers that refuse to rewrite history, so a test
/// cannot restamp a stored verdict the way it can a build. This is the honest version of the same
/// claim: it waits for a real second to pass between two writes.
#[test]
fn a_newer_run_and_a_newer_release_read_first() {
    let (_file, mut db) = seeded("newest-first");
    let builds = build_rows(&db, &[LOCAL], 0, 10, None);

    let older = persist_run(
        &mut db,
        "1",
        &builds.rows[0].build_id,
        &builds.rows[0].snapshot_id,
    );
    persist_release(&mut db, "a", &builds.rows[0].build_id, &older, 0);

    std::thread::sleep(std::time::Duration::from_millis(1_200));

    let newer = persist_run(
        &mut db,
        "2",
        &builds.rows[1].build_id,
        &builds.rows[1].snapshot_id,
    );
    let second_release = persist_release(&mut db, "b", &builds.rows[1].build_id, &newer, 1);

    let runs = run_rows(&db, &[LOCAL], 0, 10, None);
    assert_eq!(
        vec![runs.rows[0].run_id.clone(), runs.rows[1].run_id.clone()],
        vec![newer.run_id.clone(), older.run_id.clone()],
        "the run stored later is the first row"
    );

    let releases = release_rows(&db, &[LOCAL], 0, 10, None);
    assert_eq!(releases.rows[0].release_id, second_release);
    assert_eq!(
        ordered_by_time_then_identity(&stored_pairs(&db, "release_records", "id")),
        releases
            .rows
            .iter()
            .map(|row| row.release_id.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn gate_rows_filter_on_their_own_id_the_build_the_disposition_and_the_file_name() {
    let (_file, mut db) = seeded("gate-filter");
    let builds = build_rows(&db, &[LOCAL], 0, 10, None);
    let first = persist_run(
        &mut db,
        "1",
        &builds.rows[0].build_id,
        &builds.rows[0].snapshot_id,
    );
    persist_run(
        &mut db,
        "2",
        &builds.rows[1].build_id,
        &builds.rows[1].snapshot_id,
    );

    assert_eq!(
        run_rows(&db, &[LOCAL], 0, 10, Some(&first.run_id)).total,
        1,
        "a run id finds exactly its own row"
    );
    assert_eq!(
        run_rows(&db, &[LOCAL], 0, 10, Some(&first.snapshot_id[..12])).total,
        1,
        "the gated build's snapshot id is searchable"
    );

    let stored = run_rows(&db, &[LOCAL], 0, 10, None).rows[0]
        .disposition
        .clone();
    assert_eq!(
        run_rows(&db, &[LOCAL], 0, 10, Some(&stored)).total,
        2,
        "the stored disposition is searchable"
    );
    let absent = ["PASS", "REVIEW", "BLOCK"]
        .into_iter()
        .find(|state| *state != stored)
        .expect("three dispositions, one stored");
    assert_eq!(
        run_rows(&db, &[LOCAL], 0, 10, Some(absent)).total,
        0,
        "and a disposition nothing stored finds nothing"
    );

    assert_eq!(run_rows(&db, &[LOCAL], 0, 10, Some("%")).total, 0);
    assert_eq!(
        run_rows(&db, &[], 0, 10, Some(&stored)).total,
        0,
        "a filter never widens the scope"
    );
}

#[test]
fn gate_and_release_rows_are_scoped_to_the_project_that_owns_the_build() {
    let (_file, mut db) = seeded("scope");
    let mine = build_rows(&db, &[LOCAL], 0, 10, None);
    let theirs = build_rows(&db, &[OTHER], 0, 10, None);

    let my_run = persist_run(
        &mut db,
        "1",
        &mine.rows[0].build_id,
        &mine.rows[0].snapshot_id,
    );
    let their_run = persist_run(
        &mut db,
        "2",
        &theirs.rows[0].build_id,
        &theirs.rows[0].snapshot_id,
    );
    persist_release(&mut db, "a", &theirs.rows[0].build_id, &their_run, 0);

    let runs = run_rows(&db, &[LOCAL], 0, 10, None);
    assert_eq!(runs.total, 1, "one build is mine");
    assert_eq!(runs.rows[0].run_id, my_run.run_id);
    assert_ne!(runs.rows[0].run_id, their_run.run_id);
    assert!(
        runs.rows.iter().all(|row| row.file_name != *"firmware.elf"),
        "another project's Gate run appeared in this History"
    );

    assert_eq!(
        release_rows(&db, &[LOCAL], 0, 10, None).total,
        0,
        "and no release record of theirs either"
    );
    assert_eq!(
        run_rows(&db, &[], 0, 10, None).total,
        0,
        "an unnamed scope reads nothing"
    );
    assert_eq!(release_rows(&db, &[], 0, 10, None).total, 0);
    assert_eq!(run_rows(&db, &[OTHER], 0, 10, None).total, 1);
    assert_eq!(release_rows(&db, &[OTHER], 0, 10, None).total, 1);
}

#[test]
fn release_rows_carry_the_version_the_digest_and_the_run_that_qualified_them() {
    let (_file, mut db) = seeded("release-row");
    let builds = build_rows(&db, &[LOCAL], 0, 10, None);
    let newest = &builds.rows[0];
    let run = persist_run(&mut db, "1", &newest.build_id, &newest.snapshot_id);
    persist_release(&mut db, "b", &newest.build_id, &run, 7);

    let page = release_rows(&db, &[LOCAL], 0, 10, None);
    assert_eq!(page.total, 1);
    let row = &page.rows[0];
    assert_eq!(row.release_id, release_id("b"));
    assert_eq!(row.release_version, "1.7.0");
    assert_eq!(row.build_id, newest.build_id);
    assert_eq!(row.file_name, newest.file_name);
    assert_eq!(row.gate_run_id, run.run_id, "the run that qualified it");
    assert_eq!(row.manifest_sha256, hex64("manifest-b"));
    assert_eq!(
        row.stored_at,
        stored_time_of(&db, "release_records", &row.release_id),
        "and the same for the release's audit time"
    );
    let rendered = format!("{row:?}");
    assert!(
        !rendered.contains('/') && !rendered.contains('\\') && !rendered.contains("fixtures"),
        "a release row carried a path: {rendered}"
    );
}

#[test]
fn a_release_filter_finds_the_version_the_digest_the_id_and_the_run() {
    let (_file, mut db) = seeded("release-filter");
    let builds = build_rows(&db, &[LOCAL], 0, 10, None);
    let mut runs = Vec::new();
    for (index, row) in builds.rows.iter().enumerate() {
        let run = persist_run(&mut db, &index.to_string(), &row.build_id, &row.snapshot_id);
        persist_release(
            &mut db,
            &format!("c{index}"),
            &row.build_id,
            &run,
            index as u32,
        );
        runs.push(run.run_id);
    }

    let all = release_rows(&db, &[LOCAL], 0, 10, None);
    assert_eq!(all.total, 2);
    assert_eq!(
        release_rows(&db, &[LOCAL], 0, 10, Some(&hex64("manifest-c0"))).total,
        1,
        "a manifest digest is searchable"
    );
    assert_eq!(
        release_rows(&db, &[LOCAL], 0, 10, Some(&all.rows[0].release_id)).total,
        1,
        "a release id finds exactly its own row"
    );
    assert_eq!(
        release_rows(&db, &[LOCAL], 0, 10, Some(&runs[1])).total,
        1,
        "and so does the Gate run behind it"
    );
    assert_eq!(
        release_rows(&db, &[LOCAL], 0, 10, Some("1.1.0")).total,
        1,
        "a version is searchable"
    );
    assert_eq!(
        release_rows(&db, &[LOCAL], 0, 10, Some("nothing-matches")).total,
        0
    );
}

#[test]
fn reopening_the_database_reads_the_same_history_rows() {
    let file = TempDb::new("reopen");
    let (build, snapshot, run_id) = {
        let mut db = file.open();
        let scratch = TempDir::new("reopen");
        let snapshot = analyze_fixture_copy(&scratch, "base");
        let build = import(&mut db, LOCAL, &snapshot);
        let id = snapshot.id().as_str().to_owned();
        let run = persist_run(&mut db, "1", &build, &id);
        persist_release(&mut db, "d", &build, &run, 0);
        (build, id, run.run_id)
    };

    let db = file.open();
    assert_eq!(run_rows(&db, &[LOCAL], 0, 10, None).rows[0].run_id, run_id);
    assert_eq!(
        release_rows(&db, &[LOCAL], 0, 10, None).rows[0].gate_run_id,
        run_id
    );
    assert_eq!(
        build_rows(&db, &[LOCAL], 0, 10, None).rows[0].build_id,
        build
    );
    assert_eq!(stored_snapshot_id(&db, &build), snapshot);
    assert_eq!(
        db.list_history_gate_runs(&HistoryQuery::for_project(LOCAL))
            .expect("default page")
            .limit,
        DEFAULT_HISTORY_LIMIT,
        "a fresh connection uses the same page bounds"
    );
}

#[test]
fn a_query_naming_more_projects_than_sqlite_can_bind_fails_closed() {
    let (_file, db) = seeded("bind-limit");
    let oversized = HistoryQuery {
        project_ids: (0..1000).map(|index| format!("p{index}")).collect(),
        offset: 0,
        limit: 5,
        filter: None,
    };

    let builds = db.list_history_builds(&oversized);
    let runs = db.list_history_gate_runs(&oversized);
    let releases = db.list_history_releases(&oversized);

    let err = builds.expect_err("a thousand project ids must be refused for builds, not truncated");
    assert!(matches!(err, StorageError::Invariant { .. }), "got {err:?}");
    let err = runs.expect_err("and refused for Gate runs");
    assert!(matches!(err, StorageError::Invariant { .. }), "got {err:?}");
    let err = releases.expect_err("and refused for release records");
    assert!(matches!(err, StorageError::Invariant { .. }), "got {err:?}");
}

#[test]
fn a_default_query_reads_the_project_it_names_and_nothing_else() {
    let (_file, db) = seeded("default-query");
    let page = db
        .list_history_builds(&HistoryQuery::for_project(LOCAL))
        .expect("for_project page");
    assert_eq!(page.offset, 0);
    assert_eq!(page.limit, DEFAULT_HISTORY_LIMIT);
    assert_eq!(page.total, 2);
    assert!(
        page.rows
            .iter()
            .all(|row| row.build_id.starts_with("build-"))
    );
}

/// §49 asks History to stay usable with at least 100 builds, 100 Gate runs and 50 release records.
///
/// `import_snapshot` keys a build by its content, so 100 distinct real builds would need 100
/// distinct artifacts. The extra builds here are copies of the rows this crate's own importer wrote,
/// with only the identity, the stored path and the stored time changed; every Gate run and every
/// release record still goes through the real writers, so no invalid release fact is invented to
/// reach a count.
#[test]
fn history_stays_bounded_at_productization_scale() {
    let (_file, mut db) = seeded("scale");
    let builds = build_rows(&db, &[LOCAL], 0, 10, None);
    let mut all: Vec<(String, String)> = builds
        .rows
        .iter()
        .map(|row| (row.build_id.clone(), row.snapshot_id.clone()))
        .collect();
    // The seed wrote two real builds, so 98 copies reach the 100 §49 names.
    all.extend(extra_builds(&db, &builds.rows[0].build_id, 98));
    assert_eq!(all.len(), 100);

    let mut evaluations = Vec::new();
    for (index, (build, snapshot)) in all.iter().enumerate() {
        let id = run_id(&format!("r{index:04}"));
        let evaluation = passing_context(snapshot).evaluate(&id);
        db.persist_gate_run(&GateRunDraft {
            run_id: &id,
            build_id: build,
            baseline_build_id: None,
            policy_sha256: &hex64("policy"),
            evaluation: &evaluation,
            attachments: &[],
        })
        .unwrap_or_else(|err| panic!("persist run {index}: {err}"));
        evaluations.push(evaluation);
    }
    for index in 0..50 {
        persist_release(
            &mut db,
            &format!("s{index:04}"),
            &all[index].0,
            &evaluations[index],
            index as u32,
        );
    }

    let started = std::time::Instant::now();
    let builds = build_rows(&db, &[LOCAL], 0, DEFAULT_HISTORY_LIMIT, None);
    let build_read = started.elapsed();

    let started = std::time::Instant::now();
    let runs = run_rows(&db, &[LOCAL], 0, DEFAULT_HISTORY_LIMIT, None);
    let run_read = started.elapsed();

    let started = std::time::Instant::now();
    let releases = release_rows(&db, &[LOCAL], 0, DEFAULT_HISTORY_LIMIT, None);
    let release_read = started.elapsed();

    let started = std::time::Instant::now();
    // Fifteen characters reach into the digest body: the first five are the `gate-` prefix and the
    // next ten are the label bytes, which differ for every seeded run.
    let needle = runs.rows[0].run_id[..15].to_owned();
    let filtered = run_rows(&db, &[LOCAL], 0, DEFAULT_HISTORY_LIMIT, Some(&needle));
    let filter_read = started.elapsed();

    let second = run_rows(
        &db,
        &[LOCAL],
        DEFAULT_HISTORY_LIMIT,
        DEFAULT_HISTORY_LIMIT,
        None,
    );

    assert_eq!(builds.total, 100, "the build population §49 names");
    assert_eq!(runs.total, 100, "one Gate run per build");
    assert_eq!(releases.total, 50, "and 50 release records");
    assert_eq!(builds.rows.len(), DEFAULT_HISTORY_LIMIT as usize);
    assert_eq!(runs.rows.len(), DEFAULT_HISTORY_LIMIT as usize);
    assert_eq!(releases.rows.len(), DEFAULT_HISTORY_LIMIT as usize);
    assert_eq!(runs.next_offset, Some(DEFAULT_HISTORY_LIMIT));
    assert_eq!(
        filtered.total, 1,
        "a digest prefix identifies exactly one run"
    );
    assert_eq!(second.rows.len(), DEFAULT_HISTORY_LIMIT as usize);

    let first_ids: Vec<&str> = runs.rows.iter().map(|row| row.run_id.as_str()).collect();
    assert!(
        second
            .rows
            .iter()
            .all(|row| !first_ids.contains(&row.run_id.as_str())),
        "page two repeated a row from page one"
    );

    // The repository defines no latency SLA for History, so these are printed rather than asserted:
    // one command reproduces them, `cargo test -p firmwaresight-storage --test history_reads
    // --release -- --nocapture history_stays_bounded_at_productization_scale`.
    println!(
        "HISTORY SCALE builds={} gate={} releases={}: build page {build_read:?}, gate page \
         {run_read:?}, release page {release_read:?}, filtered gate page in {filter_read:?}",
        builds.total, runs.total, releases.total,
    );
}

/// Copy one real build's rows `count` times under new identities. Returns the added
/// `(build id, snapshot id)` pairs.
fn extra_builds(db: &Database, template: &str, added_count: usize) -> Vec<(String, String)> {
    let (project, normalization, created_by, state): (String, String, String, String) = db
        .connection()
        .query_row(
            "SELECT project_id, normalization_version, created_by_fwsight, state
               FROM builds WHERE id = ?1",
            params![template],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("the template build");
    let artifact = db
        .connection()
        .query_row(
            "SELECT kind, byte_size, parser_id, architecture, bitness, endianness, entry_point,
                    entry_unknown, build_id_note, build_id_unknown
               FROM artifacts WHERE id = ?1",
            params![format!("{template}#0")],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            },
        )
        .expect("the template artifact row");
    let footprint = db
        .connection()
        .query_row(
            "SELECT layout_source, weakest_basis, admissible_hard_block, nonvolatile_state,
                    nonvolatile_bytes, nonvolatile_class, runtime_state, runtime_bytes,
                    runtime_class, excluded_metadata
               FROM memory_footprints WHERE build_id = ?1",
            params![template],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, i64>(9)?,
                ))
            },
        )
        .expect("the template footprint row");

    let mut added = Vec::new();
    for index in 0..added_count {
        let snapshot = hex64(&format!("synthetic-{index:04}"));
        let build = format!("build-{snapshot}");

        db.connection()
            .execute(
                "INSERT INTO builds (id, project_id, snapshot_id, normalization_version,
                                     created_by_fwsight, state, created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![
                    build,
                    project,
                    snapshot,
                    normalization,
                    created_by,
                    state,
                    format!("2026-07-01T{:02}:{:02}:00Z", index / 60, index % 60)
                ],
            )
            .expect("synthetic build row");

        // The stored path keeps a parent directory on purpose: the read above must hide it.
        db.connection()
            .execute(
                "INSERT INTO artifacts (id, build_id, path, kind, sha256, byte_size, parser_id,
                                        architecture, bitness, endianness, entry_point,
                                        entry_unknown, build_id_note, build_id_unknown)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
                params![
                    format!("{build}#0"),
                    build,
                    format!("hidden-parent-scope/{index:04}/firmware.elf"),
                    artifact.0,
                    hex64(&format!("sha-{index:04}")),
                    artifact.1,
                    artifact.2,
                    artifact.3,
                    artifact.4,
                    artifact.5,
                    artifact.6,
                    artifact.7,
                    artifact.8,
                    artifact.9,
                ],
            )
            .expect("synthetic artifact row");

        db.connection()
            .execute(
                "INSERT INTO memory_footprints (build_id, layout_source, weakest_basis,
                        admissible_hard_block, nonvolatile_state, nonvolatile_bytes,
                        nonvolatile_class, runtime_state, runtime_bytes, runtime_class,
                        excluded_metadata)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    build,
                    footprint.0,
                    footprint.1,
                    footprint.2,
                    footprint.3,
                    footprint.4,
                    footprint.5,
                    footprint.6,
                    footprint.7,
                    footprint.8,
                    footprint.9,
                ],
            )
            .expect("synthetic footprint row");

        added.push((build, snapshot));
    }
    assert_eq!(added.len(), added_count, "every copy was written");
    added
}
