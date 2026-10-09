//! The Compare read path: which builds may be picked, and what one picked snapshot hydrates into.
//!
//! P2 promises two things that only a storage test can prove. First, a comparison runs on
//! *persisted* facts: an old build stays comparable after its ELF and MAP have been moved or
//! deleted, because nothing in this path opens a file. Second, the candidate list is scoped,
//! bounded and ordered by a rule rather than by SQLite's visit order, and the import timestamp is
//! never allowed to read as a build time (`04_TECH/24` Time).
//!
//! Rows created here are real: each snapshot comes from a committed fixture through the same
//! `pipeline::analyze` + `import_snapshot` path the desktop uses. Where a test needs a timestamp it
//! cannot wait for, it sets the column directly and says so.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_core::domain::diff::{BudgetState, DiffSnapshotInput};
use firmwaresight_core::domain::identity::Fact;
use firmwaresight_storage::{
    CandidateQuery, CompareCandidate, DEFAULT_CANDIDATE_LIMIT, DEFAULT_QUERY_LIMIT, Database,
    MAX_CANDIDATE_LIMIT, SCHEMA_VERSION, StorageError,
};

const LOCAL: &str = "local-desktop";
const P0_DEMO: &str = "p0-desktop";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/<name>")
        .to_path_buf()
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-compare-{label}-{}-{:x}",
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
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-compare-db-{label}-{}-{:x}.sqlite",
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
        Database::open(self.0.as_path()).expect("database opens")
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

/// Copy a committed fixture pair into a scratch directory so a test can delete or corrupt the
/// originals' replacements afterwards. Returns (elf, map).
fn copy_fixture(dir: &TempDir, side: &str) -> (PathBuf, PathBuf) {
    let source = repo_root().join(format!("fixtures/elf/p2-diff/{side}"));
    let elf = dir.path().join(format!("{side}.elf"));
    let map = dir.path().join(format!("{side}.map"));
    std::fs::copy(source.join("firmware.elf"), &elf).expect("copy elf");
    std::fs::copy(source.join("firmware.map"), &map).expect("copy map");
    (elf, map)
}

fn analyze_side(dir: &TempDir, side: &str) -> BuildSnapshot {
    let (elf, map) = copy_fixture(dir, side);
    pipeline::analyze(&AnalysisRequest::new(elf).with_map(map))
        .unwrap_or_else(|err| panic!("the {side} fixture must analyze: {err}"))
        .snapshot
        .clone()
}

fn analyze_p0_dual_region() -> BuildSnapshot {
    let root = repo_root();
    pipeline::analyze(
        &AnalysisRequest::new(root.join("fixtures/elf/p0-dual-region/firmware.elf"))
            .with_map(root.join("fixtures/elf/p0-dual-region/firmware.map")),
    )
    .expect("the P0 fixture must analyze")
    .snapshot
    .clone()
}

fn import(db: &mut Database, project: &str, name: &str, snapshot: &BuildSnapshot) -> String {
    db.import_snapshot(project, name, snapshot)
        .unwrap_or_else(|err| panic!("import into {project}: {err}"))
}

fn candidates(db: &Database, projects: &[&str]) -> Vec<CompareCandidate> {
    let query = CandidateQuery {
        project_ids: projects.iter().map(|id| (*id).to_owned()).collect(),
        offset: 0,
        limit: DEFAULT_CANDIDATE_LIMIT,
        filter: None,
    };
    db.list_compare_candidates(&query)
        .expect("candidate list")
        .rows
}

/// The hash `fixtures/manifest.json` records for one committed P2 fixture file.
///
/// Asserting a literal here would go stale the moment the pair is regenerated, which is exactly
/// what happened to an earlier draft of this file: it quoted a hash from before the settings page
/// was added and passed nothing. The manifest is the record the parser tests already trust.
fn recorded_hash(side: &str, name: &str) -> String {
    let raw =
        std::fs::read_to_string(repo_root().join("fixtures/manifest.json")).expect("manifest");
    let relative = format!("fixtures/elf/p2-diff/{side}/{name}");
    // The manifest is written by the generator two lines at a time, so a line scan is enough and
    // keeps this test free of a JSON dependency it does not need.
    let lines: Vec<&str> = raw.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.contains(&format!("\"{relative}\"")))
        .unwrap_or_else(|| panic!("{relative} is not in fixtures/manifest.json"));
    let hash = lines[at..at + 4]
        .iter()
        .find(|line| line.contains("\"sha256\""))
        .and_then(|line| line.split_once(':'))
        .map(|(_, value)| value.trim().trim_end_matches(',').trim_matches('\"'))
        .unwrap_or_else(|| panic!("no sha256 recorded next to {relative}"));
    assert_eq!(hash.len(), 64, "{relative} records a short hash: {hash}");
    hash.to_owned()
}

fn set_imported_at(db: &Database, build_id: &str, stamp: &str) {
    // A test cannot wait a minute between two imports, so the column that records import time is
    // written directly. Nothing else about the row changes.
    db.connection()
        .execute(
            "UPDATE builds SET created_at = ?2 WHERE id = ?1",
            rusqlite::params![build_id, stamp],
        )
        .expect("stamp import time");
}

fn section<'a>(
    input: &'a DiffSnapshotInput,
    name: &str,
) -> Option<&'a firmwaresight_core::domain::diff::DiffSection> {
    input
        .sections
        .iter()
        .find(|row| row.name.value().is_some_and(|n| n == name))
}

#[test]
fn only_the_callers_own_projects_are_offered() {
    let dir = TempDir::new("scope");
    let mut db = TempDb::new("scope").open();
    import(
        &mut db,
        LOCAL,
        "Local analyses",
        &analyze_side(&dir, "base"),
    );
    import(
        &mut db,
        P0_DEMO,
        "FirmwareSight P0",
        &analyze_p0_dual_region(),
    );

    let local = candidates(&db, &[LOCAL]);
    assert_eq!(
        local.len(),
        1,
        "the caller asked for its own project and got someone else's build"
    );
    assert_eq!(local[0].file_name, "base.elf");

    let scoped = candidates(&db, &[P0_DEMO]);
    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].file_name, "firmware.elf");

    let both = candidates(&db, &[LOCAL, P0_DEMO]);
    assert_eq!(both.len(), 2);
}

#[test]
fn an_empty_project_list_offers_nothing_rather_than_everything() {
    let dir = TempDir::new("empty-scope");
    let mut db = TempDb::new("empty-scope").open();
    import(
        &mut db,
        LOCAL,
        "Local analyses",
        &analyze_side(&dir, "base"),
    );

    let page = db
        .list_compare_candidates(&CandidateQuery::default())
        .expect("an unscoped query is answered, not refused");
    assert!(
        page.rows.is_empty(),
        "a caller that named no project must not receive the P0 demo build or anyone else's history"
    );
    assert_eq!(page.total, 0);
    assert_eq!(page.limit, DEFAULT_CANDIDATE_LIMIT);
}

#[test]
fn the_p0_demo_build_is_never_a_compare_candidate_for_a_user() {
    let dir = TempDir::new("p0-excluded");
    let mut db = TempDb::new("p0-excluded").open();
    import(
        &mut db,
        P0_DEMO,
        "FirmwareSight P0",
        &analyze_p0_dual_region(),
    );
    import(
        &mut db,
        LOCAL,
        "Local analyses",
        &analyze_side(&dir, "target"),
    );

    let names: Vec<_> = candidates(&db, &[LOCAL])
        .iter()
        .map(|row| row.snapshot_id.clone())
        .collect();
    let demo: Vec<_> = candidates(&db, &[P0_DEMO])
        .iter()
        .map(|row| row.snapshot_id.clone())
        .collect();

    assert_eq!(names.len(), 1);
    assert!(
        names.iter().all(|id| !demo.contains(id)),
        "the demo project's build leaked into the user's selector"
    );
}

#[test]
fn newest_imported_first_with_the_snapshot_id_as_tie_break() {
    let dir = TempDir::new("order");
    let mut db = TempDb::new("order").open();
    let older = import(
        &mut db,
        LOCAL,
        "Local analyses",
        &analyze_side(&dir, "base"),
    );
    let newer = import(
        &mut db,
        LOCAL,
        "Local analyses",
        &analyze_side(&dir, "target"),
    );

    set_imported_at(&db, &older, "2026-01-01T00:00:00Z");
    set_imported_at(&db, &newer, "2026-02-01T00:00:00Z");

    let rows = candidates(&db, &[LOCAL]);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].build_id, newer, "the newest import comes first");
    assert_eq!(rows[0].imported_at, "2026-02-01T00:00:00Z");

    // Two imports stamped identically must still produce one stable order.
    set_imported_at(&db, &older, "2026-02-01T00:00:00Z");
    let tied = candidates(&db, &[LOCAL]);
    let again = candidates(&db, &[LOCAL]);
    assert_eq!(
        tied.iter().map(|r| r.build_id.clone()).collect::<Vec<_>>(),
        again.iter().map(|r| r.build_id.clone()).collect::<Vec<_>>(),
        "a tie broken by visit order is not a stable order"
    );
    let mut by_snapshot = tied.clone();
    by_snapshot.sort_by(|a, b| a.snapshot_id.cmp(&b.snapshot_id));
    assert_eq!(
        tied.iter()
            .map(|r| r.snapshot_id.clone())
            .collect::<Vec<_>>(),
        by_snapshot
            .iter()
            .map(|r| r.snapshot_id.clone())
            .collect::<Vec<_>>(),
        "the documented tie-break is snapshot id ascending"
    );
}

#[test]
fn the_candidate_page_is_bounded_at_one_hundred_rows() {
    let dir = TempDir::new("bound");
    let mut db = TempDb::new("bound").open();
    import(
        &mut db,
        LOCAL,
        "Local analyses",
        &analyze_side(&dir, "base"),
    );

    let page = db
        .list_compare_candidates(&CandidateQuery {
            project_ids: vec![LOCAL.to_owned()],
            offset: 0,
            limit: 100_000,
            filter: None,
        })
        .expect("a request above the ceiling is clamped, not refused");
    assert_eq!(
        page.limit, MAX_CANDIDATE_LIMIT,
        "the ceiling is enforced in Rust, whatever the caller asked for"
    );

    let defaulted = db
        .list_compare_candidates(&CandidateQuery {
            project_ids: vec![LOCAL.to_owned()],
            offset: 0,
            limit: 0,
            filter: None,
        })
        .expect("a limit that is not a page size is treated as not asked");
    assert_eq!(defaulted.limit, DEFAULT_CANDIDATE_LIMIT);
    assert_ne!(defaulted.limit, DEFAULT_QUERY_LIMIT);
}

#[test]
fn a_candidate_names_a_file_never_a_host_path() {
    let dir = TempDir::new("no-path");
    let mut db = TempDb::new("no-path").open();
    let snapshot = analyze_side(&dir, "base");
    let build_id = import(&mut db, LOCAL, "Local analyses", &snapshot);

    let stored: String = db
        .connection()
        .query_row(
            "SELECT path FROM artifacts WHERE id = ?1",
            rusqlite::params![format!("{build_id}#0")],
            |row| row.get(0),
        )
        .expect("the stored row keeps the path the user chose");
    assert!(
        stored.contains('/') || stored.contains('\\'),
        "the harness expects a real directory in the stored path, got {stored}"
    );

    let rows = candidates(&db, &[LOCAL]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].file_name, "base.elf");
    let rendered = format!("{rows:?}");
    for marker in ["C:\\", "D:\\", "/home/", "/Users/", "/tmp/"] {
        assert!(
            !rendered.contains(marker),
            "a candidate row carried {marker}: {rendered}"
        );
    }
}

#[test]
fn a_failed_import_is_not_offered_as_a_comparable_build() {
    let dir = TempDir::new("failed");
    let mut db = TempDb::new("failed").open();
    let build_id = import(
        &mut db,
        LOCAL,
        "Local analyses",
        &analyze_side(&dir, "base"),
    );
    db.connection()
        .execute(
            "UPDATE builds SET state = 'FAILED' WHERE id = ?1",
            rusqlite::params![build_id],
        )
        .expect("mark the import failed");

    assert!(
        candidates(&db, &[LOCAL]).is_empty(),
        "a build that never completed must not be comparable"
    );
}

#[test]
fn hydration_reads_the_stored_footprint_and_both_row_sets() {
    let dir = TempDir::new("hydrate");
    let mut db = TempDb::new("hydrate").open();
    let snapshot = analyze_side(&dir, "base");
    let expected_id = snapshot.id().as_str().to_owned();
    import(&mut db, LOCAL, "Local analyses", &snapshot);

    let input = db.load_diff_input(&expected_id).expect("hydrate base");

    assert_eq!(input.snapshot_id, expected_id);
    assert_eq!(input.artifact.file_name, "base.elf");
    assert_eq!(
        input.artifact.sha256,
        recorded_hash("base", "firmware.elf").as_str(),
        "the identity is the one stored at import time"
    );
    let memory = input.memory.as_ref().expect("a footprint row exists");
    assert_eq!(memory.nonvolatile.state, BudgetState::Exact);
    assert_eq!(memory.nonvolatile.bytes, Some(256));
    assert_eq!(memory.runtime_ram.bytes, Some(8));
    assert!(
        memory.evidence.map_backed,
        "the MAP is stored as a companion artifact"
    );
    assert_eq!(memory.evidence.layout_source, "map");
    assert_eq!(
        memory.evidence.weakest_basis.as_deref(),
        Some("MapRegionAndElfLoad")
    );

    let text = section(&input, ".text").expect(".text is stored");
    assert_eq!(text.file_size, 124);
    assert_eq!(text.role, "Code");
    assert!(
        input
            .symbols
            .iter()
            .any(|row| row.name.value().map(String::as_str) == Some("mqtt_task")),
        "the symbol rows hydrate"
    );
}

#[test]
fn an_unknown_stored_fact_stays_unknown_after_hydration() {
    let mut db = TempDb::new("unknown").open();
    let snapshot = analyze_p0_dual_region();
    let id = snapshot.id().as_str().to_owned();
    import(&mut db, LOCAL, "Local analyses", &snapshot);

    let input = db.load_diff_input(&id).expect("hydrate the P0 fixture");

    let debug_info = section(&input, ".debug_info").expect("the fixture has debug sections");
    assert!(
        matches!(&debug_info.virtual_address, Fact::Unknown { reason } if !reason.is_empty()),
        "a section with no runtime address must not hydrate as address 0"
    );
    assert!(
        input.sections.iter().any(|row| row.file_offset.is_none()),
        "at least one stored section has no file offset; `file_offset` has no reason column, so it          must read back as absent rather than as 0"
    );
    for row in input
        .sections
        .iter()
        .filter(|row| row.file_offset.is_none())
    {
        assert_eq!(
            row.file_offset,
            None,
            "a section named {:?} with no offset stays absent",
            row.name.value()
        );
    }

    let bss = section(&input, ".bss").expect(".bss is stored");
    assert_eq!(bss.file_size, 0);
    assert!(bss.memory_size.is_known(), ".bss has a real runtime size");

    assert!(
        input
            .symbols
            .iter()
            .any(|row| matches!(&row.size, Fact::Unknown { .. })),
        "a sizeless symbol must stay unknown rather than become a zero-sized one"
    );
}

#[test]
fn comparison_works_after_the_source_files_are_gone() {
    let dir = TempDir::new("moved");
    let mut db = TempDb::new("moved").open();
    let mut inputs = Vec::new();
    for side in ["base", "target"] {
        let snapshot = analyze_side(&dir, side);
        import(&mut db, LOCAL, "Local analyses", &snapshot);
        inputs.push(snapshot.id().as_str().to_owned());
    }

    // Delete every copy this test made. The originals under fixtures/ are untouched, and Compare
    // must not care either way.
    for entry in std::fs::read_dir(dir.path()).expect("temp dir reads") {
        let path = entry.expect("dir entry").path();
        std::fs::remove_file(&path).expect("remove the source file");
    }

    let base = db
        .load_diff_input(&inputs[0])
        .expect("base hydrates with no file");
    let target = db
        .load_diff_input(&inputs[1])
        .expect("target hydrates with no file");
    let diff = firmwaresight_core::domain::diff::compare(&base, &target).expect("compare runs");

    assert_eq!(diff.memory.nonvolatile.delta, Some(120));
    assert_eq!(diff.memory.runtime_ram.delta, Some(68));
    assert!(
        diff.section_changes.iter().any(|row| row.key == ".calib"
            && row.change == firmwaresight_core::domain::diff::ChangeKind::Removed),
        "the stored rows still carry the removal"
    );
}

#[test]
fn rewriting_the_source_file_does_not_change_what_the_snapshot_reports() {
    let dir = TempDir::new("rewrite");
    let mut db = TempDb::new("rewrite").open();
    let (elf, _) = copy_fixture(&dir, "base");
    let snapshot =
        pipeline::analyze(&AnalysisRequest::new(&elf).with_map(dir.path().join("base.map")))
            .expect("analyze")
            .snapshot
            .clone();
    let id = snapshot.id().as_str().to_owned();
    let stored_size = snapshot
        .primary_artifact()
        .expect("the snapshot has a primary artifact")
        .byte_size;
    import(&mut db, LOCAL, "Local analyses", &snapshot);

    // The file on disk is now a different binary. The stored snapshot must keep reporting the bytes
    // it was built from, which is the whole reason Compare does not re-read anything.
    let mut hostile = std::fs::read(&elf).expect("read the copy");
    let length = hostile.len();
    for byte in hostile.iter_mut() {
        *byte = byte.wrapping_add(1);
    }
    std::fs::write(&elf, &hostile).expect("overwrite the copy");
    assert_eq!(hostile.len(), length);

    let input = db.load_diff_input(&id).expect("hydration ignores the file");
    assert_eq!(
        input.artifact.sha256,
        recorded_hash("base", "firmware.elf").as_str(),
        "the identity is the stored one, not a re-hash of whatever is on disk now"
    );
    assert_eq!(
        input.artifact.byte_size, stored_size,
        "the size is the one recorded at import, not a stat of whatever is on disk now"
    );
    assert!(
        section(&input, ".text").is_some(),
        "sections come from rows, not from a second parse"
    );
}

#[test]
fn the_same_snapshot_hydrates_identically_every_time() {
    let dir = TempDir::new("stable");
    let mut db = TempDb::new("stable").open();
    let snapshot = analyze_side(&dir, "target");
    let id = snapshot.id().as_str().to_owned();
    import(&mut db, LOCAL, "Local analyses", &snapshot);

    let first = db.load_diff_input(&id).expect("first hydration");
    let second = db.load_diff_input(&id).expect("second hydration");
    assert_eq!(first, second, "a diff input must be reproducible");
    assert_eq!(
        first.sections.iter().map(|s| s.index).collect::<Vec<_>>(),
        second.sections.iter().map(|s| s.index).collect::<Vec<_>>()
    );
}

#[test]
fn an_unknown_snapshot_id_is_a_typed_error_not_a_panic() {
    let db = TempDb::new("missing").open();
    let err = db
        .load_diff_input("snap-does-not-exist")
        .expect_err("an unknown snapshot cannot hydrate");
    assert!(
        matches!(err, StorageError::NotFound { ref id } if id == "snap-does-not-exist"),
        "expected NotFound, got {err:?}"
    );
    assert_eq!(err.stable_code(), "ERR-STORAGE-4005");
}

#[test]
fn compare_added_no_schema_change() {
    // The prompt's §44 promise, checked against the files rather than against this sentence. P3 added
    // 0003 for the Gate and P4 added 0004 for the Release Bundle, so the check is that neither numbered
    // step belongs to Compare and that the migration count still equals the schema version.
    //
    // `0005_unknown_reasons` is P5's, and it is here because the guard is a *named-stage* check, not a
    // frozen count: a migration that appeared without belonging to a stage is exactly what this list
    // would catch. P5 added it to store the Unknown reason that `optional_fact_u64` used to discard
    // (L6/L7), never to give History a table - see P5_VALIDATION/P5_MIGRATION_DECISION.md.
    // `0006_release_attachments` is C1-U1's: the additive `gate_run_attachments` table of
    // `04_TECH/28` §7.7, named for the unit that wrote it rather than added to a frozen count.
    let migrations = repo_root().join("crates/firmwaresight-storage/migrations");
    let mut names: Vec<_> = std::fs::read_dir(migrations)
        .expect("migrations dir")
        .filter_map(|entry| {
            entry
                .ok()
                .map(|e| e.file_name().to_string_lossy().into_owned())
        })
        .filter(|name| name.ends_with(".sql"))
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "0001_initial.sql",
            "0002_evidence_keyed_by_build.sql",
            "0003_gate_history.sql",
            "0004_release_records.sql",
            "0005_unknown_reasons.sql",
            "0006_release_attachments.sql",
        ],
        "every migration belongs to a named stage"
    );
    assert_eq!(
        names.len(),
        usize::try_from(SCHEMA_VERSION).expect("a schema version that fits a count"),
        "one numbered migration per schema version"
    );
    assert!(
        names
            .iter()
            .all(|name| !name.contains("diff") && !name.contains("compare")),
        "P2 wrote a migration: {names:?}"
    );
}
