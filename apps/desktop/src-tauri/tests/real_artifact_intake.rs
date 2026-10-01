//! P1-A0 real artifact intake: the selection session, its IPC shape, and the typed errors around
//! both.
//!
//! These tests never open a dialog. The dialog belongs to the command layer (`src/intake.rs`), and
//! its only product is a path the user chose; every test here injects that path directly, which is
//! exactly what the Rust side would hand the session. That keeps the claims - opaque ids, no paths
//! crossing IPC, MAP presence changing evidence strength - testable without a window, and proves
//! the session has no dependence on the dialog implementation.
//!
//! The `desktop_parity.rs` file owns the CLI-vs-Desktop fact equality for the fixture path. This
//! file owns the user-path, so the same equality is asserted here once, from a file the catalog
//! never offered.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::Session;
use firmwaresight_desktop::intake::{on_artifact_picked, on_map_picked};
use firmwaresight_desktop::ipc::SelectionDto;
use firmwaresight_desktop::service::FixtureCatalog;
use firmwaresight_desktop::{LOCAL_PROJECT_ID, LOCAL_PROJECT_NAME};
use serde_json::Value;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // apps/desktop/src-tauri -> apps/desktop
        .and_then(Path::parent) // -> apps
        .and_then(Path::parent) // -> repo root
        .expect("the desktop shell lives at <root>/apps/desktop/src-tauri")
        .to_path_buf()
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        // The label suffix keeps two directories from ever colliding on the same nanosecond.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let unique = format!("fwsight-p1a0-{label}-{:x}-{:x}", std::process::id(), nanos);
        let path = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory creates");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Copy a committed fixture out of the catalog, the way a user's own build directory would hold
/// it. The copy is what gets "selected", so nothing here reads through `FixtureCatalog::analyze`.
fn copied_fixture(dir: &TempDir, name: &str, source: &Path) -> PathBuf {
    let target = dir.join(name);
    std::fs::copy(source, &target)
        .unwrap_or_else(|err| panic!("copy {} to {}: {err}", source.display(), target.display()));
    target
}

fn dual_region_elf(dir: &TempDir) -> PathBuf {
    let catalog = FixtureCatalog::from_environment();
    copied_fixture(
        dir,
        "app.elf",
        &catalog.artifact_path(firmwaresight_desktop::FixtureKey::P0DualRegion),
    )
}

fn dual_region_map(dir: &TempDir) -> PathBuf {
    let catalog = FixtureCatalog::from_environment();
    let map = catalog
        .artifact_path(firmwaresight_desktop::FixtureKey::P0DualRegion)
        .parent()
        .expect("the fixture lives in a directory")
        .join("firmware.map");
    copied_fixture(dir, "app.map", &map)
}

fn basic_elf(dir: &TempDir) -> PathBuf {
    let catalog = FixtureCatalog::from_environment();
    copied_fixture(
        dir,
        "basic.elf",
        &catalog.artifact_path(firmwaresight_desktop::FixtureKey::P0Basic),
    )
}

fn db_path(dir: &TempDir) -> PathBuf {
    dir.join("history.sqlite")
}

fn session(dir: &TempDir) -> Session {
    Session::open(FixtureCatalog::from_environment(), db_path(dir))
        .unwrap_or_else(|err| panic!("desktop session opens: {err}"))
}

fn golden(name: &str) -> Value {
    let path = repo_root().join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{} must be committed: {err}", path.display()));
    serde_json::from_str(&text).expect("golden is JSON")
}

fn json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).expect("the DTO serializes")
}

#[test]
fn a_selected_artifact_yields_an_opaque_id_and_a_file_name_only() {
    let dir = TempDir::new("opaque");
    let elf = dual_region_elf(&dir);
    let session = session(&dir);

    let selection: SelectionDto = session
        .stage_artifact(&elf, "op-stage")
        .expect("a real path stages");

    assert!(!selection.selection_id.is_empty());
    assert_eq!(selection.file_name, "app.elf");
    assert!(!selection.map_attached);
    assert_eq!(selection.map_file_name, None);

    // The id is a handle, not a location: it must not encode the path it stands for.
    let text = json(&selection);
    assert!(
        !text.contains(&elf.display().to_string()) && !text.contains(dir.path().to_str().unwrap()),
        "the selection DTO leaked a filesystem location: {text}"
    );
    assert!(
        !selection.selection_id.contains("app.elf"),
        "the selection id is derived from the file name, not opaque: {}",
        selection.selection_id
    );
}

#[test]
fn no_selection_or_summary_payload_carries_a_path() {
    let dir = TempDir::new("nopath");
    let elf = dual_region_elf(&dir);
    let map = dual_region_map(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-a").expect("stages");
    let with_map = session
        .stage_map(&selection.selection_id, &map, "op-b")
        .expect("the map attaches");
    let summary = session
        .analyze_selection(&selection.selection_id, "op-c")
        .expect("the selection analyzes");

    let root = dir.path().to_str().expect("a temp path is unicode");
    for (label, text) in [
        ("selection", json(&selection)),
        ("selection with map", json(&with_map)),
        ("summary", json(&summary)),
    ] {
        assert!(
            !text.contains(root),
            "{label} contains the directory the artifact was chosen from: {text}"
        );
        for forbidden in ["\"path\"", "\"filePath\"", "\"directory\"", "\"root\""] {
            assert!(
                !text.contains(forbidden),
                "{label} carries the key {forbidden}; only a name may cross IPC"
            );
        }
    }
    assert_eq!(with_map.map_file_name.as_deref(), Some("app.map"));
    assert!(with_map.map_attached);
}

#[test]
fn an_unknown_selection_id_is_a_typed_error_and_not_a_panic() {
    let dir = TempDir::new("unknown");
    let session = session(&dir);

    let err = session
        .analyze_selection("sel-does-not-exist", "op-unknown")
        .expect_err("an id the session never issued cannot analyze");

    assert_eq!(err.code, "ERR-INPUT-0001");
    assert_eq!(err.operation_id, "op-unknown");
    assert!(!err.message.is_empty());
    assert!(
        !err.message.contains("sel-does-not-exist"),
        "the message repeats a handle the user cannot act on: {}",
        err.message
    );
    assert!(err.remediation.is_some(), "every error needs a next step");

    // The same id on the other two surfaces must fail the same way rather than panic.
    for result in [
        session.stage_map("sel-does-not-exist", dual_region_map(&dir), "op-map"),
        session.clear_map("sel-does-not-exist", "op-clear"),
    ] {
        let err = result.expect_err("an unknown id cannot be mutated");
        assert_eq!(err.code, "ERR-INPUT-0001");
    }
}

#[test]
fn a_user_chosen_file_produces_the_same_core_facts_the_cli_reports() {
    let dir = TempDir::new("parity");
    let elf = dual_region_elf(&dir);
    let map = dual_region_map(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-p").expect("stages");
    session
        .stage_map(&selection.selection_id, &map, "op-p")
        .expect("the map attaches");
    let summary = session
        .analyze_selection(&selection.selection_id, "op-p")
        .expect("analyzes");
    let cli = golden("golden/cli/p0-dual-region-analyze.json");

    assert_eq!(
        summary.artifact.sha256,
        cli["artifact"]["sha256"].as_str().unwrap()
    );
    assert_eq!(
        summary.artifact.byte_size,
        cli["artifact"]["byteSize"].as_u64().unwrap()
    );
    assert_eq!(
        summary.artifact.architecture,
        cli["artifact"]["architecture"].as_str().unwrap()
    );
    assert_eq!(summary.section_count, 17);
    assert_eq!(summary.symbol_count, 32);
    assert_eq!(
        summary.memory.nonvolatile_image_footprint.bytes,
        cli["memory"]["nonvolatileImageFootprint"]["bytes"].as_u64()
    );
    assert_eq!(
        summary.memory.runtime_ram_footprint.bytes,
        cli["memory"]["runtimeRamFootprint"]["bytes"].as_u64()
    );
    assert_eq!(
        summary.capabilities.map,
        cli["capabilities"]["map"].as_str().unwrap()
    );
    // The name shown is the one the user chose, not the catalog's.
    assert_eq!(summary.artifact.file_name, "app.elf");
    assert_eq!(summary.source, "artifact");
}

#[test]
fn an_elf_without_a_map_degrades_the_map_capability_instead_of_failing() {
    let dir = TempDir::new("nomap");
    let elf = basic_elf(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-nomap").expect("stages");
    let summary = session
        .analyze_selection(&selection.selection_id, "op-nomap")
        .expect("an ELF alone still analyzes");

    assert_eq!(summary.capabilities.map, "not-provided");
    assert_eq!(summary.memory.layout_source, "none");
    assert_eq!(
        summary.memory.weakest_evidence_basis.as_deref(),
        Some("elf-address-and-flags")
    );
    assert!(!summary.memory.admissible_for_hard_block);
    assert_eq!(summary.source, "artifact");
}

#[test]
fn an_elf_only_summary_claims_no_map_provenance_and_carries_no_from_map_flag() {
    // Two halves of the same closure. `EvidenceSummaryDto.from_map` used to be true for an ELF that
    // was never given a MAP, because the pipeline labelled ELF-derived charges with map provenance
    // and the count simply followed the label. The label is now read off the accounting basis, and
    // the redundant boolean is gone: the fields that mean MAP are `capabilities.map`,
    // `memory.layout_source` and `memory.weakest_evidence_basis`, asserted here and in
    // `an_elf_without_a_map_degrades_the_map_capability_instead_of_failing`.
    let dir = TempDir::new("frommap");
    let elf = basic_elf(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-fm").expect("stages");
    let summary = session
        .analyze_selection(&selection.selection_id, "op-fm")
        .expect("analyzes");

    assert_eq!(summary.capabilities.map, "not-provided");
    assert_eq!(summary.memory.layout_source, "none");
    assert_eq!(
        summary.memory.weakest_evidence_basis.as_deref(),
        Some("elf-address-and-flags")
    );

    let counts = json(&summary.evidence_summary);
    assert!(
        !counts.contains("fromMap") && !counts.contains("map"),
        "the redundant MAP boolean is still on the wire: {counts}"
    );
    assert!(
        counts.contains("total"),
        "the class counts are the part worth sending: {counts}"
    );
}

#[test]
fn the_generated_typescript_binding_carries_no_from_map_field() {
    // The UI reads this file, so the boundary is only closed on both sides if the regenerated
    // contract lost the field too. CI regenerates it from the Rust and fails on drift, which is
    // what makes this a check of the committed artifact rather than of a hand edit.
    let path = repo_root().join("apps/desktop/ui/src/ipc/generated/EvidenceSummaryDto.ts");
    let ts = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{} must be committed: {err}", path.display()));

    assert!(
        !ts.contains("fromMap"),
        "the generated binding still promises a fromMap field:\n{ts}"
    );
    for field in ["total", "observed", "derived", "declared", "unknown"] {
        assert!(
            ts.contains(field),
            "the class counts must stay in the contract, `{field}` is missing:\n{ts}"
        );
    }
}

#[test]
fn attaching_a_map_changes_the_snapshot_id_and_removing_it_restores_it() {
    // The identity gap: ELF-only and ELF+MAP used to compose the same snapshot id, so the stronger
    // run looked like a duplicate of the weaker one.
    let dir = TempDir::new("snapshotid");
    let elf = dual_region_elf(&dir);
    let map = dual_region_map(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-i0").expect("stages");
    let without = session
        .analyze_selection(&selection.selection_id, "op-i1")
        .expect("analyzes");

    session
        .stage_map(&selection.selection_id, &map, "op-i2")
        .expect("attaches");
    let with = session
        .analyze_selection(&selection.selection_id, "op-i3")
        .expect("analyzes");

    assert_ne!(
        without.identity.snapshot_id, with.identity.snapshot_id,
        "different evidence input sets must not collide onto one snapshot"
    );

    let repeated = session
        .analyze_selection(&selection.selection_id, "op-i4")
        .expect("the same pair analyzes again");
    assert_eq!(
        repeated.identity.snapshot_id, with.identity.snapshot_id,
        "one ELF + one MAP is one snapshot, however often it is analyzed"
    );

    session
        .clear_map(&selection.selection_id, "op-i5")
        .expect("detaches");
    let detached = session
        .analyze_selection(&selection.selection_id, "op-i6")
        .expect("analyzes");
    assert_eq!(
        detached.identity.snapshot_id, without.identity.snapshot_id,
        "removing the MAP returns to the ELF-only snapshot"
    );
    assert_eq!(
        with.artifact.sha256, without.artifact.sha256,
        "the artifact being reported on never changed, only the evidence about it"
    );
}

#[test]
fn a_map_after_an_elf_only_analysis_is_stored_as_a_second_build() {
    // The persistence half of the gap, and the reason the identity changed: the UI was showing
    // MAP-backed truth while SQLite kept the weaker ELF-only record of the same file.
    let dir = TempDir::new("twostore");
    let elf = dual_region_elf(&dir);
    let map = dual_region_map(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-p0").expect("stages");
    let without = session
        .analyze_selection(&selection.selection_id, "op-p1")
        .expect("analyzes");
    session
        .stage_map(&selection.selection_id, &map, "op-p2")
        .expect("attaches");
    let with = session
        .analyze_selection(&selection.selection_id, "op-p3")
        .expect("analyzes");
    // The user clicks Analyze a third time on the same pair.
    let repeat = session
        .analyze_selection(&selection.selection_id, "op-p4")
        .expect("analyzes");
    drop(session);

    assert_eq!(repeat.identity.snapshot_id, with.identity.snapshot_id);

    let probe = firmwaresight_storage::Database::open(&db_path(&dir)).expect("reopen");
    let builds: i64 = probe
        .connection()
        .query_row("SELECT COUNT(*) FROM builds", [], |row| row.get(0))
        .expect("count");
    assert_eq!(
        builds, 2,
        "the weaker build stays as history beside the stronger one"
    );

    let rows = |snapshot_id: &str| -> Vec<String> {
        let build_id: String = probe
            .connection()
            .query_row(
                "SELECT id FROM builds WHERE snapshot_id = ?1",
                (snapshot_id,),
                |row| row.get(0),
            )
            .expect("the build is stored");
        let mut stmt = probe
            .connection()
            .prepare("SELECT kind FROM artifacts WHERE build_id = ?1 ORDER BY id")
            .expect("select kinds");
        stmt.query_map((build_id,), |row| row.get::<_, String>(0))
            .expect("kinds")
            .map(|r| r.expect("row"))
            .collect()
    };

    assert_eq!(rows(&without.identity.snapshot_id), vec!["Elf".to_owned()]);
    assert_eq!(
        rows(&with.identity.snapshot_id),
        vec!["Elf".to_owned(), "Map".to_owned()],
        "the companion artifact is persisted, not just displayed"
    );

    let layout: (String, String) = (
        probe
            .connection()
            .query_row(
                "SELECT m.layout_source FROM memory_footprints m JOIN builds b ON b.id = m.build_id
                  WHERE b.snapshot_id = ?1",
                (without.identity.snapshot_id.as_str(),),
                |row| row.get(0),
            )
            .expect("weaker layout"),
        probe
            .connection()
            .query_row(
                "SELECT m.layout_source FROM memory_footprints m JOIN builds b ON b.id = m.build_id
                  WHERE b.snapshot_id = ?1",
                (with.identity.snapshot_id.as_str(),),
                |row| row.get(0),
            )
            .expect("stronger layout"),
    );
    assert_eq!(
        layout.0, "none",
        "the ELF-only build kept its own weaker attribution"
    );
    assert_eq!(
        layout.1, "map",
        "the MAP-backed attribution is what SQLite now holds"
    );

    let stronger_evidence: i64 = probe
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM evidence e JOIN builds b ON b.id = e.build_id
              WHERE b.snapshot_id = ?1",
            (with.identity.snapshot_id.as_str(),),
            |row| row.get(0),
        )
        .expect("count stronger evidence");
    let weaker_evidence: i64 = probe
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM evidence e JOIN builds b ON b.id = e.build_id
              WHERE b.snapshot_id = ?1",
            (without.identity.snapshot_id.as_str(),),
            |row| row.get(0),
        )
        .expect("count weaker evidence");
    assert!(
        stronger_evidence > weaker_evidence,
        "the stronger record must hold more than the weaker one: {weaker_evidence} vs \
         {stronger_evidence}"
    );
    assert_eq!(
        with.evidence_summary.total as i64, stronger_evidence,
        "what the UI counted is what storage kept"
    );
    assert_eq!(
        without.evidence_summary.total as i64, weaker_evidence,
        "the ELF-only summary matches its own build"
    );
}

#[test]
fn attaching_a_map_strrengthens_the_evidence_and_detach_takes_it_back() {
    let dir = TempDir::new("map");
    let elf = dual_region_elf(&dir);
    let map = dual_region_map(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-m0").expect("stages");

    let without = session
        .analyze_selection(&selection.selection_id, "op-m1")
        .expect("the ELF alone analyzes");
    assert_eq!(without.capabilities.map, "not-provided");
    assert!(!without.memory.admissible_for_hard_block);

    session
        .stage_map(&selection.selection_id, &map, "op-m2")
        .expect("the map attaches");
    let with = session
        .analyze_selection(&selection.selection_id, "op-m3")
        .expect("the pair analyzes");
    assert_eq!(with.capabilities.map, "provided");
    assert!(with.memory.admissible_for_hard_block);
    assert_eq!(
        with.memory.weakest_evidence_basis.as_deref(),
        Some("map-memory-configuration+elf-load")
    );
    assert!(
        with.evidence_summary.total > without.evidence_summary.total,
        "a MAP that changes nothing would not be worth a dialog"
    );

    session
        .clear_map(&selection.selection_id, "op-m4")
        .expect("the map detaches");
    let detached = session
        .analyze_selection(&selection.selection_id, "op-m5")
        .expect("the ELF alone analyzes again");
    assert_eq!(detached.capabilities.map, "not-provided");
    assert_eq!(
        detached.artifact.sha256, with.artifact.sha256,
        "detaching a MAP must not change which artifact is being reported on"
    );
}

#[test]
fn an_artifact_deleted_after_selection_is_a_typed_error_not_a_panic() {
    let dir = TempDir::new("deleted");
    let elf = dual_region_elf(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-d0").expect("stages");
    std::fs::remove_file(&elf).expect("the copy is removed");

    let err = session
        .analyze_selection(&selection.selection_id, "op-d1")
        .expect_err("a deleted file cannot analyze");
    assert_eq!(err.code, "ERR-INPUT-0001");
    assert_eq!(err.operation_id, "op-d1");
    assert!(err.remediation.is_some());
    assert!(
        !err.message.contains(&elf.display().to_string()),
        "the message quotes a path the user cannot do anything with: {}",
        err.message
    );
}

#[test]
fn format_truth_comes_from_the_bytes_not_from_an_elf_extension() {
    let dir = TempDir::new("malformed");
    // Named like a linker output, but the bytes are not ELF: the magic check is what decides.
    let fake = dir.join("firmware.elf");
    std::fs::write(&fake, b"this is a text file wearing an .elf name\n").expect("writes");
    let session = session(&dir);

    let selection = session
        .stage_artifact(&fake, "op-f0")
        .expect("staging does not guess the format from a name");
    let err = session
        .analyze_selection(&selection.selection_id, "op-f1")
        .expect_err("the pipeline refuses non-ELF bytes");

    assert_eq!(err.code, "ERR-FORMAT-0001");
    assert_eq!(
        err.message,
        "That file is not a format FirmwareSight can analyze yet."
    );
    assert!(err.remediation.is_some());
}

#[test]
fn a_map_that_is_not_a_gnu_ld_map_is_a_typed_error_and_is_never_silently_dropped() {
    let dir = TempDir::new("badmap");
    let elf = dual_region_elf(&dir);
    let fake_map = dir.join("notes.map");
    std::fs::write(&fake_map, b"not a linker map at all\n").expect("writes");
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-bm0").expect("stages");
    session
        .stage_map(&selection.selection_id, &fake_map, "op-bm1")
        .expect("staging does not parse the MAP early");

    let err = session
        .analyze_selection(&selection.selection_id, "op-bm2")
        .expect_err("a MAP that no adapter reads must fail, not degrade quietly");
    assert!(
        err.code == "ERR-MAP-3001" || err.code == "ERR-MAP-3002",
        "a rejected MAP must report a MAP code, got {}: {}",
        err.code,
        err.message
    );
    assert!(err.remediation.is_some());
}

#[test]
fn a_map_attached_to_one_selection_does_not_reach_another() {
    let dir = TempDir::new("isolate");
    let dual = dual_region_elf(&dir);
    let basic = basic_elf(&dir);
    let map = dual_region_map(&dir);
    let session = session(&dir);

    let with_map = session.stage_artifact(&dual, "op-i0").expect("stages");
    let other = session.stage_artifact(&basic, "op-i1").expect("stages");
    let attached = session
        .stage_map(&with_map.selection_id, &map, "op-i2")
        .expect("the map attaches to the first selection");

    assert!(attached.map_attached);
    assert!(
        !session
            .selection(&other.selection_id)
            .expect("the second selection still exists")
            .map_attached,
        "the MAP followed the selection id, not the session"
    );

    let summary = session
        .analyze_selection(&other.selection_id, "op-i3")
        .expect("the second selection analyzes");
    assert_eq!(summary.capabilities.map, "not-provided");
    assert_eq!(summary.artifact.file_name, "basic.elf");
}

#[test]
fn a_selection_is_recorded_under_the_local_project_not_the_p0_demo_identity() {
    let dir = TempDir::new("project");
    let elf = dual_region_elf(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-pr0").expect("stages");
    session
        .analyze_selection(&selection.selection_id, "op-pr1")
        .expect("analyzes");
    drop(session);

    let probe = firmwaresight_storage::Database::open(&db_path(&dir)).expect("reopen");
    let local_builds: i64 = probe
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM builds WHERE project_id = ?1",
            (LOCAL_PROJECT_ID,),
            |row| row.get(0),
        )
        .expect("count local builds");
    let p0_builds: i64 = probe
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM builds WHERE project_id = 'p0-desktop'",
            [],
            |row| row.get(0),
        )
        .expect("count p0 builds");

    assert_eq!(local_builds, 1, "the selection is recorded once");
    assert_eq!(
        p0_builds, 0,
        "a file the user chose is not filed under the P0 demo project"
    );

    let name: String = probe
        .connection()
        .query_row(
            "SELECT name FROM projects WHERE id = ?1",
            (LOCAL_PROJECT_ID,),
            |row| row.get(0),
        )
        .expect("project name");
    assert_eq!(name, LOCAL_PROJECT_NAME);
}

#[test]
fn selecting_the_same_file_twice_does_not_duplicate_history() {
    let dir = TempDir::new("dedupe");
    let elf = dual_region_elf(&dir);
    let session = session(&dir);

    for _ in 0..3 {
        let selection = session.stage_artifact(&elf, "op-r").expect("stages");
        session
            .analyze_selection(&selection.selection_id, "op-r")
            .expect("analyzes");
    }

    let probe = firmwaresight_storage::Database::open(&db_path(&dir)).expect("reopen");
    let builds: i64 = probe
        .connection()
        .query_row("SELECT COUNT(*) FROM builds", [], |row| row.get(0))
        .expect("count");
    assert_eq!(builds, 1, "three analyses of one file is one build");
}

#[test]
fn the_selection_path_adds_no_schema_migration() {
    let dir = TempDir::new("schema");
    let elf = dual_region_elf(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-s0").expect("stages");
    session
        .analyze_selection(&selection.selection_id, "op-s1")
        .expect("analyzes");

    let probe = firmwaresight_storage::Database::open(&db_path(&dir)).expect("reopen");
    let version: i64 = probe
        .connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .expect("schema version");
    assert_eq!(
        version,
        firmwaresight_storage::SCHEMA_VERSION,
        "P1-A0 must not move the schema on its own"
    );
    let names: Vec<String> = probe
        .connection()
        .prepare("SELECT name FROM schema_migrations ORDER BY version")
        .expect("select")
        .query_map([], |row| row.get(0))
        .expect("migrations")
        .map(|row| row.expect("name"))
        .collect();
    assert_eq!(
        names,
        vec![
            "0001_initial".to_owned(),
            "0002_evidence_keyed_by_build".to_owned(),
            "0003_gate_history".to_owned(),
            "0004_release_records".to_owned(),
        ],
        "every migration belongs to a named stage, and none of them is the selection path's"
    );
}

#[test]
fn a_cancelled_dialog_is_not_an_error() {
    let dir = TempDir::new("cancel");
    let session = Session::open(FixtureCatalog::from_environment(), db_path(&dir)).expect("opens");
    let session = std::sync::Arc::new(session);

    // `None` is what the native dialog reports when the user presses Cancel.
    let outcome =
        on_artifact_picked(&session, None, "op-cancel").expect("cancelling is not an error");
    assert!(
        outcome.is_none(),
        "a cancelled artifact pick yields no selection"
    );

    let selection = session
        .stage_artifact(dual_region_elf(&dir), "op-cancel2")
        .expect("stages");
    let map_outcome = on_map_picked(&session, &selection.selection_id, None, "op-cancel3")
        .expect("cancelling a MAP is not an error");
    assert!(map_outcome.is_none());
    assert!(
        !session
            .selection(&selection.selection_id)
            .expect("the selection survives")
            .map_attached,
        "a cancelled MAP dialog must leave the artifact selection intact"
    );
}

#[test]
fn the_summary_of_a_real_artifact_stays_bounded() {
    let dir = TempDir::new("bounded");
    let elf = dual_region_elf(&dir);
    let map = dual_region_map(&dir);
    let session = session(&dir);

    let selection = session.stage_artifact(&elf, "op-b0").expect("stages");
    session
        .stage_map(&selection.selection_id, &map, "op-b1")
        .expect("attaches");
    let summary = session
        .analyze_selection(&selection.selection_id, "op-b2")
        .expect("analyzes");

    let text = json(&summary);
    let payload: Value = serde_json::from_str(&text).expect("round trip");
    for forbidden in ["symbols", "sections", "evidence"] {
        assert!(
            payload.get(forbidden).is_none(),
            "a {forbidden} list crossed the IPC boundary"
        );
    }
    assert!(
        text.len() < 8_192,
        "summary payload grew to {} bytes",
        text.len()
    );
}

#[test]
fn selection_ids_are_unique_within_a_session() {
    let dir = TempDir::new("unique");
    let elf = dual_region_elf(&dir);
    let basic = basic_elf(&dir);
    let session = session(&dir);

    let first = session.stage_artifact(&elf, "op-u0").expect("stages");
    let second = session.stage_artifact(&elf, "op-u1").expect("stages");
    let third = session.stage_artifact(&basic, "op-u2").expect("stages");

    let ids = [
        &first.selection_id,
        &second.selection_id,
        &third.selection_id,
    ];
    let unique: std::collections::HashSet<&String> = ids.iter().copied().collect();
    assert_eq!(unique.len(), 3, "two clicks must not collide on one handle");
}

#[test]
fn the_session_can_be_used_from_a_worker_thread() {
    // The commands move both dialog results and analysis off the WebView event loop, which only
    // works if the session and its selection store cross a thread boundary.
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Session>();
    assert_send_sync::<SelectionDto>();

    let dir = TempDir::new("thread");
    let elf = dual_region_elf(&dir);
    let db = db_path(&dir);
    let worker = std::thread::spawn(move || {
        let session = Session::open(FixtureCatalog::from_environment(), &db).expect("opens");
        let selection = session.stage_artifact(&elf, "op-t").expect("stages");
        session
            .analyze_selection(&selection.selection_id, "op-t")
            .expect("analyzes off the main thread")
    });

    let summary = worker.join().expect("the worker must not panic");
    let cli = golden("golden/cli/p0-dual-region-analyze.json");
    assert_eq!(
        summary.artifact.sha256,
        cli["artifact"]["sha256"].as_str().expect("sha")
    );
}
