//! P2 Compare over IPC: what the boundary is allowed to say about a computed diff.
//!
//! Core owns the diff and the storage layer owns what is persisted, so this file is about the
//! *boundary* only:
//!
//! - the candidate list is project-scoped, bounded, and never carries a host path;
//! - `compare_snapshots` answers with a summary whose arrays are capped, never with the change tables
//!   themselves (prompt §20, §22);
//! - rows are paged, filtered and ordered through a session-local handle, and a handle the session
//!   never issued or has already dropped is a typed error rather than an empty page;
//! - export paths come from a Rust-side dialog, cancel and "keep my file" are normal outcomes, and a
//!   refused or failed overwrite leaves the existing bytes alone (prompt §35, §36);
//! - and the capability still grants the WebView no filesystem, shell or dialog authority.
//!
//! As in the P1 detail file, these call the same `Session` methods the commands do, with no window
//! and no runtime.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::compare::{ExportFormat, write_export};
use firmwaresight_desktop::ipc::{
    CandidatePageRequestDto, ChangeKindFilterDto, CompareRequestDto, SectionChangeQueryDto,
    SectionChangeSortDto, SortDirDto, SymbolChangeQueryDto, SymbolChangeSortDto,
};
use firmwaresight_desktop::{FixtureKey, Session, service::FixtureCatalog};
use serde_json::Value;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // apps/desktop/src-tauri -> apps/desktop
        .and_then(Path::parent) // -> apps
        .and_then(Path::parent) // -> repo root
        .expect("the desktop shell lives at <root>/apps/desktop/src-tauri")
        .to_path_buf()
}

fn fixture(side: &str) -> (PathBuf, PathBuf) {
    let root = repo_root();
    (
        root.join(format!("fixtures/elf/p2-diff/{side}/firmware.elf")),
        root.join(format!("fixtures/elf/p2-diff/{side}/firmware.map")),
    )
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-p2-compare-{label}-{}-{:x}.sqlite",
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

/// Analyze and store one fixture side the way the product path does: a staged file, a staged MAP,
/// then the same pipeline the CLI runs. The snapshot id is what the UI would hold.
fn store_side(session: &Session, side: &str) -> String {
    let (artifact, map) = fixture(side);
    let selection = session
        .stage_artifact(&artifact, "op-stage")
        .expect("the committed fixture stages");
    session
        .stage_map(&selection.selection_id, &map, "op-map")
        .expect("the fixture MAP stages");
    let summary = session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("the fixture analyzes and stores");
    summary.identity.snapshot_id
}

fn session_with_pair(label: &str) -> (Session, TempDb, String, String) {
    let db = TempDb::new(label);
    let session =
        Session::open(FixtureCatalog::from_environment(), db.path()).expect("a session opens");
    let base = store_side(&session, "base");
    let target = store_side(&session, "target");
    (session, db, base, target)
}

fn compare(
    session: &Session,
    base: &str,
    target: &str,
) -> firmwaresight_desktop::ipc::CompareSummaryDto {
    session
        .compare_snapshots(
            &CompareRequestDto {
                base_snapshot_id: base.to_owned(),
                target_snapshot_id: target.to_owned(),
            },
            "op-compare",
        )
        .expect("the fixture pair compares")
}

fn json(value: &impl serde::Serialize) -> Value {
    serde_json::from_str(&serde_json::to_string(value).expect("serializes")).expect("round trip")
}

// ---------------------------------------------------------------- candidate list

#[test]
fn the_candidate_list_offers_builds_the_user_analyzed_and_no_demo_history() {
    let db = TempDb::new("scope");
    let session =
        Session::open(FixtureCatalog::from_environment(), db.path()).expect("a session opens");
    let local = store_side(&session, "base");
    // A P0 demo build is stored in the same database, under its own project label.
    session
        .analyze_and_store(FixtureKey::P0DualRegion, "op-demo")
        .expect("the demo fixture stores");

    let page = session
        .list_compare_candidates(&CandidatePageRequestDto::default(), "op-list")
        .expect("the list reads");

    assert_eq!(page.total, 1, "user history is the local project only");
    assert_eq!(page.rows[0].snapshot_id, local);
    assert_eq!(page.rows[0].file_name, "firmware.elf");
    assert_eq!(page.rows[0].architecture, "Arm");
}

#[test]
fn a_candidate_page_never_exceeds_the_bound_and_never_names_a_host_path() {
    let (session, _db, _, _) = session_with_pair("bounds");
    let page = session
        .list_compare_candidates(
            &CandidatePageRequestDto {
                offset: None,
                limit: Some(10_000),
            },
            "op-list",
        )
        .expect("the list reads");
    assert!(
        page.limit <= 100,
        "the boundary clamps an oversized page, got {}",
        page.limit
    );

    let text = serde_json::to_string(&page.rows).expect("serializes");
    for marker in ["C:\\", "D:\\", "/home/", "/Users/", "/tmp/", "fixtures/elf"] {
        assert!(
            !text.contains(marker),
            "a candidate carries {marker}: {text}"
        );
    }
}

#[test]
fn a_candidate_carries_its_recorded_budgets_so_the_selector_can_show_them() {
    let (session, _db, _, _) = session_with_pair("budgets");
    let page = session
        .list_compare_candidates(&CandidatePageRequestDto::default(), "op-list")
        .expect("the list reads");
    let base = page
        .rows
        .iter()
        .find(|row| row.file_name == "firmware.elf")
        .expect("both sides share a file name, so any row proves the shape");

    assert_eq!(base.nonvolatile.state, "exact");
    assert!(
        base.nonvolatile.bytes.is_some(),
        "a MAP-backed build reports a number, not a blank"
    );
    assert_eq!(base.runtime_ram.state, "exact");
    assert!(
        !base.imported_at.is_empty(),
        "the record says when it entered history"
    );
}

// ---------------------------------------------------------------- summary

#[test]
fn a_summary_is_bounded_and_carries_no_change_table() {
    let (session, _db, base, target) = session_with_pair("summary");
    let summary = compare(&session, &base, &target);
    let document = json(&summary);

    let mut keys: Vec<&str> = document
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "base",
            "counts",
            "diffId",
            "memory",
            "objectChanges",
            "target",
            "topSections",
            "topSymbols",
            "warnings"
        ],
        "the summary shape is the bounded one prompt §22 names"
    );
    assert!(summary.top_sections.len() <= 5);
    assert!(summary.top_symbols.len() <= 10);
    assert_eq!(summary.base.snapshot_id, base);
    assert_eq!(summary.target.snapshot_id, target);
    assert_eq!(summary.base.file_name, "firmware.elf");
    assert_ne!(summary.base.sha256, summary.target.sha256);

    // The two ADR-0021 budgets, each with old, new and a signed delta.
    assert_eq!(summary.memory.nonvolatile.base, Some(256));
    assert_eq!(summary.memory.nonvolatile.target, Some(376));
    assert_eq!(summary.memory.nonvolatile.delta, Some(120));
    assert_eq!(summary.memory.runtime_ram.delta, Some(68));
    assert_eq!(summary.memory.comparability, "exact");
    assert!(summary.memory.base.map_backed);
    assert_eq!(
        summary.memory.base.weakest_evidence_basis.as_deref(),
        Some("MapRegionAndElfLoad")
    );
    assert_eq!(
        summary.memory.evidence_warning, None,
        "both sides have a MAP"
    );
    assert!(!summary.object_changes.available);
    assert_eq!(summary.counts.sections.added, 1);
    assert_eq!(summary.counts.sections.removed, 1);
    assert_eq!(summary.counts.sections.ambiguous, 0);
}

#[test]
fn the_diff_handle_is_opaque_and_session_local() {
    let (session, _db, base, target) = session_with_pair("handle");
    let first = compare(&session, &base, &target);
    let second = compare(&session, &base, &target);

    assert!(first.diff_id.starts_with("cmp-"), "{}", first.diff_id);
    assert_ne!(
        first.diff_id, second.diff_id,
        "each computation gets its own handle"
    );
    assert!(
        !first.diff_id.contains('/') && !first.diff_id.contains('\\'),
        "a handle is not a path"
    );
    let text = serde_json::to_string(&first).expect("serializes");
    for marker in ["C:\\", "D:\\", "/home/", "/Users/", "fixtures/elf"] {
        assert!(!text.contains(marker), "the summary carries {marker}");
    }
}

#[test]
fn the_same_build_on_both_sides_is_refused_not_answered_with_empty_tables() {
    let (session, _db, base, _) = session_with_pair("same");
    let err = session
        .compare_snapshots(
            &CompareRequestDto {
                base_snapshot_id: base.clone(),
                target_snapshot_id: base,
            },
            "op-same",
        )
        .expect_err("a build is not its own comparison");
    assert_eq!(err.code, "ERR-DIFF-5001");
    assert!(!err.message.is_empty());
    assert!(err.remediation.is_some());
}

#[test]
fn an_unknown_snapshot_id_is_a_typed_error() {
    let (session, _db, base, _) = session_with_pair("unknown-snapshot");
    let err = session
        .compare_snapshots(
            &CompareRequestDto {
                base_snapshot_id: base,
                target_snapshot_id: "snap-does-not-exist".to_owned(),
            },
            "op-unknown",
        )
        .expect_err("an id storage has never seen cannot be compared");
    assert_eq!(err.code, "ERR-STORAGE-4005");
    assert!(
        !err.message.contains("SELECT") && !err.message.contains("sqlite"),
        "an internal detail must not become the user's message: {}",
        err.message
    );
}

#[test]
fn a_dropped_diff_handle_is_a_typed_error() {
    let (session, _db, base, target) = session_with_pair("evict");
    let first = compare(&session, &base, &target);
    // The documented cap: enough newer comparisons push the oldest handle out.
    for _ in 0..8 {
        compare(&session, &base, &target);
    }

    let err = session
        .query_section_changes(
            &SectionChangeQueryDto {
                diff_id: first.diff_id.clone(),
                ..Default::default()
            },
            "op-evicted",
        )
        .expect_err("an evicted handle cannot page anything");
    assert_eq!(err.code, "ERR-DIFF-5002");
    assert!(
        err.remediation
            .as_deref()
            .is_some_and(|text| text.contains("again")),
        "the reader is told what to do next"
    );
}

// ---------------------------------------------------------------- change pages

#[test]
fn section_changes_page_filter_and_order_without_hiding_a_row() {
    let (session, _db, base, target) = session_with_pair("sections");
    let summary = compare(&session, &base, &target);

    let all = session
        .query_section_changes(
            &SectionChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                ..Default::default()
            },
            "op-all",
        )
        .expect("the first page reads");
    assert_eq!(
        all.total,
        summary.counts.sections.added
            + summary.counts.sections.removed
            + summary.counts.sections.changed,
        "the page total is every changed row, not a truncated summary"
    );
    assert!(all.rows.iter().any(|row| row.change_kind == "Added"));
    assert!(all.rows.iter().any(|row| row.change_kind == "Removed"));
    assert!(all.rows.iter().any(|row| row.change_kind == "Changed"));

    let added_only = session
        .query_section_changes(
            &SectionChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                change_kind: Some(ChangeKindFilterDto::Added),
                ..Default::default()
            },
            "op-added",
        )
        .expect("the filter reads");
    assert_eq!(added_only.total, 1);
    assert_eq!(added_only.rows[0].key, ".ota");
    assert!(
        added_only.rows[0].base.is_none(),
        "an addition has no base row"
    );
    assert_eq!(added_only.rows[0].file_size.base, None);
    assert_eq!(added_only.rows[0].file_size.delta, None);

    let text_filter = session
        .query_section_changes(
            &SectionChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                filter: Some("TEXT".to_owned()),
                ..Default::default()
            },
            "op-filter",
        )
        .expect("a case-insensitive filter reads");
    assert_eq!(text_filter.total, 1);
    assert_eq!(text_filter.rows[0].key, ".text");
}

#[test]
fn a_removed_section_keeps_absence_on_the_target_side() {
    let (session, _db, base, target) = session_with_pair("removed");
    let summary = compare(&session, &base, &target);
    let page = session
        .query_section_changes(
            &SectionChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                change_kind: Some(ChangeKindFilterDto::Removed),
                ..Default::default()
            },
            "op-removed",
        )
        .expect("the filter reads");

    let row = &page.rows[0];
    assert_eq!(row.key, ".calib");
    assert!(
        row.target.is_none(),
        "a removal leaves no target row behind"
    );
    assert_eq!(row.file_size.target, None);
    assert_eq!(row.file_size.delta, None);
    assert!(
        row.file_size
            .reason
            .as_deref()
            .is_some_and(|text| !text.is_empty()),
        "the absence is explained, not displayed as zero"
    );
    assert!(!row.ambiguous, "a name that occurs once is a clean removal");
}

#[test]
fn sorting_by_delta_leaves_rows_without_a_number_at_the_end() {
    let (session, _db, base, target) = session_with_pair("sort");
    let summary = compare(&session, &base, &target);

    let descending = session
        .query_section_changes(
            &SectionChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                sort: SectionChangeSortDto::Delta,
                direction: SortDirDto::Desc,
                ..Default::default()
            },
            "op-desc",
        )
        .expect("the sorted page reads");
    let measured: Vec<i64> = descending
        .rows
        .iter()
        .filter_map(|row| row.file_size.delta)
        .collect();
    let mut expected = measured.clone();
    expected.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(measured, expected, "the biggest growth leads");

    // The Added and Removed rows have no delta at all; they must sit behind every measured row in
    // both directions, because an absent number is not a small number.
    let unmeasured_last = descending
        .rows
        .iter()
        .rev()
        .take_while(|row| row.file_size.delta.is_none())
        .count();
    assert!(
        unmeasured_last >= 2,
        "added and removed rows stay at the end"
    );

    let ascending = session
        .query_section_changes(
            &SectionChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                sort: SectionChangeSortDto::Delta,
                ..Default::default()
            },
            "op-asc",
        )
        .expect("the other direction reads");
    let tail_unmeasured = ascending
        .rows
        .iter()
        .rev()
        .take_while(|row| row.file_size.delta.is_none())
        .count();
    assert_eq!(
        tail_unmeasured, unmeasured_last,
        "flipping the column must not promote an unknown"
    );
}

#[test]
fn the_additions_page_ranks_on_the_quantity_the_growth_list_uses() {
    let (session, _db, base, target) = session_with_pair("additions");
    let summary = compare(&session, &base, &target);

    // The Compare screen builds its "largest additions" list from this query, and its "top growth"
    // list comes from Core's memory-size ranking. Two adjacent rankings of different quantities
    // would invite a reader to compare them, so the ordering field here is the same one.
    let page = session
        .query_section_changes(
            &SectionChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                change_kind: Some(ChangeKindFilterDto::Added),
                sort: SectionChangeSortDto::MemorySize,
                direction: SortDirDto::Desc,
                ..Default::default()
            },
            "op-added",
        )
        .expect("the added page reads");

    assert!(page.total > 0, "the fixture pair adds sections");
    let sizes: Vec<Option<u64>> = page
        .rows
        .iter()
        .map(|row| row.memory_size.target.or(row.memory_size.base))
        .collect();
    let measured: Vec<u64> = sizes.iter().flatten().copied().collect();
    let mut expected = measured.clone();
    expected.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(measured, expected, "the largest addition leads");
    assert!(
        sizes.iter().rev().take_while(|size| size.is_none()).count()
            == sizes.iter().filter(|size| size.is_none()).count(),
        "a row with no number stays behind every row that has one"
    );
    for row in &page.rows {
        assert_eq!(row.change_kind, "Added", "the filter is the filter");
    }
}

#[test]
fn the_same_query_twice_returns_the_same_page() {
    let (session, _db, base, target) = session_with_pair("stable");
    let summary = compare(&session, &base, &target);
    let request = SectionChangeQueryDto {
        diff_id: summary.diff_id.clone(),
        sort: SectionChangeSortDto::FileSize,
        direction: SortDirDto::Desc,
        offset: 2,
        limit: Some(3),
        ..Default::default()
    };

    let first = session
        .query_section_changes(&request, "op-a")
        .expect("page reads");
    let second = session
        .query_section_changes(&request, "op-b")
        .expect("page reads again");
    assert_eq!(json(&first), json(&second));
    assert_eq!(first.offset, 2);
    assert_eq!(first.rows.len(), 3);
    assert!(
        first.next_offset.is_some(),
        "there is a page after this one"
    );
}

#[test]
fn a_requested_page_size_is_clamped_rather_than_honoured() {
    let (session, _db, base, target) = session_with_pair("clamp");
    let summary = compare(&session, &base, &target);
    let page = session
        .query_symbol_changes(
            &SymbolChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                limit: Some(100_000),
                ..Default::default()
            },
            "op-clamp",
        )
        .expect("the page reads");
    assert_eq!(
        page.limit, 500,
        "the boundary decides how much a payload may hold"
    );
    assert!(page.rows.len() <= 500);
}

#[test]
fn symbol_changes_cross_with_both_sides_and_a_signed_delta() {
    let (session, _db, base, target) = session_with_pair("symbols");
    let summary = compare(&session, &base, &target);
    let page = session
        .query_symbol_changes(
            &SymbolChangeQueryDto {
                diff_id: summary.diff_id.clone(),
                filter: Some("mqtt_task".to_owned()),
                sort: SymbolChangeSortDto::SizeDelta,
                direction: SortDirDto::Desc,
                ..Default::default()
            },
            "op-symbol",
        )
        .expect("the page reads");

    assert_eq!(page.total, 1);
    let row = &page.rows[0];
    assert_eq!(row.name, "mqtt_task");
    assert_eq!(row.change_kind, "Changed");
    assert_eq!(row.size.base, Some(4));
    assert_eq!(row.size.target, Some(46));
    assert_eq!(row.size.delta, Some(42));
    let base_side = row.base.as_ref().expect("both sides exist");
    let target_side = row.target.as_ref().expect("both sides exist");
    assert!(
        base_side.address != target_side.address || base_side.size != target_side.size,
        "a Changed row shows what differs"
    );
    assert!(
        row.differing_fields.contains(&"size".to_owned()),
        "{:?}",
        row.differing_fields
    );
}

#[test]
fn an_unknown_diff_id_pages_nothing_and_says_why() {
    let (session, _db, _, _) = session_with_pair("bad-handle");
    let err = session
        .query_symbol_changes(
            &SymbolChangeQueryDto {
                diff_id: "cmp-dead-beef".to_owned(),
                ..Default::default()
            },
            "op-bad",
        )
        .expect_err("the session never issued that handle");
    assert_eq!(err.code, "ERR-DIFF-5002");
}

// ---------------------------------------------------------------- export

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("fwsight-p2-export-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch directory");
        Self(dir)
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_cancelled_export_is_a_normal_outcome_and_writes_nothing() {
    let scratch = Scratch::new("cancel");
    let untouched = scratch.file("never.txt");
    let outcome = write_export(None, "text", ExportFormat::Json, |_| true)
        .expect("cancelling is not an error");
    assert_eq!(outcome.status, "cancelled");
    assert_eq!(outcome.file_name, None);
    assert_eq!(outcome.format, "json");
    assert!(!untouched.exists());
}

#[test]
fn a_refused_overwrite_leaves_the_existing_file_byte_for_byte() {
    let scratch = Scratch::new("overwrite");
    let target = scratch.file("diff.json");
    std::fs::write(&target, "someone elses report").expect("seed");

    let outcome = write_export(Some(target.clone()), "fresh", ExportFormat::Json, |_| false)
        .expect("keeping a file is not an error");
    assert_eq!(outcome.status, "kept-existing");
    assert_eq!(
        std::fs::read_to_string(&target).expect("read"),
        "someone elses report"
    );

    let approved = write_export(Some(target.clone()), "fresh", ExportFormat::Json, |_| true)
        .expect("the replacement was approved");
    assert_eq!(approved.status, "written");
    assert_eq!(approved.file_name.as_deref(), Some("diff.json"));
    assert_eq!(std::fs::read_to_string(&target).expect("read"), "fresh");
}

#[test]
fn an_export_that_cannot_be_written_reports_a_typed_error_without_leaking_the_directory() {
    let missing = PathBuf::from("fwsight-no-such-folder-p2").join("diff.json");
    let err = write_export(Some(missing), "text", ExportFormat::Html, |_| true)
        .expect_err("writing into a folder that does not exist cannot succeed");
    assert_eq!(err.code, "ERR-EXPORT-6001");
    let details = err
        .details
        .expect("the failure names the file it could not write");
    assert!(details.starts_with("diff.json:"), "{details}");
    assert!(
        !details.contains("fwsight-no-such-folder-p2"),
        "the unreachable directory stays out of the payload: {details}"
    );
    assert!(
        err.remediation
            .as_deref()
            .is_some_and(|text| text.contains("unchanged")),
        "the reader is told the existing file is safe"
    );
}

#[test]
fn the_json_an_export_writes_is_the_portable_diff_document() {
    let (session, _db, base, target) = session_with_pair("export-json");
    let summary = compare(&session, &base, &target);
    let (text, suggested) = session
        .render_export(&summary.diff_id, ExportFormat::Json, "op-export")
        .expect("the diff renders");
    assert!(suggested.ends_with(".json"), "{suggested}");
    assert!(!suggested.contains('/') && !suggested.contains('\\'));

    let scratch = Scratch::new("json");
    let file = scratch.file(&suggested);
    let outcome = write_export(Some(file.clone()), &text, ExportFormat::Json, |_| true)
        .expect("the write succeeds");
    assert_eq!(outcome.status, "written");
    assert_eq!(
        std::fs::read_to_string(&file).expect("read"),
        text,
        "the file holds exactly what the shell rendered"
    );
    // The write went through a sibling temporary; nothing of that machinery may remain.
    let leftovers = std::fs::read_dir(&scratch.0)
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with('.'))
        .count();
    assert_eq!(leftovers, 0, "a finished export leaves no scratch file");

    let document: Value = serde_json::from_str(&text).expect("one JSON document");
    assert_eq!(document["schema"], "urn:firmwaresight:schema:diff:1");
    assert_eq!(document["schemaVersion"], 1);
    assert_eq!(document["base"]["snapshotId"], base);
    assert_eq!(document["memory"]["nonvolatile"]["delta"], 120);
    for marker in ["C:\\", "D:\\", "/home/", "/Users/", "fixtures/elf"] {
        assert!(!text.contains(marker), "the export carries {marker}");
    }
}

#[test]
fn the_html_an_export_writes_is_one_self_contained_file() {
    let (session, _db, base, target) = session_with_pair("export-html");
    let summary = compare(&session, &base, &target);
    let (text, suggested) = session
        .render_export(&summary.diff_id, ExportFormat::Html, "op-export")
        .expect("the diff renders");
    assert!(suggested.ends_with(".html"), "{suggested}");

    let scratch = Scratch::new("html");
    let file = scratch.file(&suggested);
    write_export(Some(file.clone()), &text, ExportFormat::Html, |_| true).expect("written");
    let html = std::fs::read_to_string(&file).expect("read");
    assert_eq!(html, text);
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<style>"));
    for forbidden in ["<script", "http://", "https://", "<link", "@import"] {
        assert!(
            !html.to_ascii_lowercase().contains(forbidden),
            "the export references {forbidden}"
        );
    }
    assert!(html.contains("Delta = target"));
    assert!(
        html.contains("3f615b624717"),
        "both sides are identified by hash, not only by a shared file name"
    );
    assert!(
        html.matches("firmware.elf").count() >= 2,
        "the base and the target are each named"
    );
}

#[test]
fn an_export_payload_names_no_path_and_offers_the_ui_only_a_file_name() {
    let outcome = write_export(None, "text", ExportFormat::Html, |_| true).expect("cancelled");
    let document = json(&outcome);
    let mut keys: Vec<&str> = document
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, vec!["fileName", "format", "status"]);
}

#[test]
fn the_webview_still_holds_no_filesystem_shell_or_dialog_authority() {
    // The save dialog is opened by Rust. If the capability ever started naming a dialog, fs or shell
    // permission, exports would be a WebView with the machine's filesystem behind them.
    let capability =
        std::fs::read_to_string(repo_root().join("apps/desktop/src-tauri/capabilities/main.json"))
            .expect("the capability file is part of the shell");
    let document: Value = serde_json::from_str(&capability).expect("valid JSON");
    let permissions: Vec<String> = document["permissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .map(|entry| match entry {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        })
        .collect();
    for forbidden in ["dialog", "fs", "shell", "http", "open", "save", "path"] {
        assert!(
            !permissions
                .iter()
                .any(|granted| granted.contains(forbidden)),
            "capability grants {forbidden}: {permissions:?}"
        );
    }
    assert!(
        document["windows"]
            .as_array()
            .expect("windows")
            .iter()
            .any(|window| window.as_str() == Some("main")),
        "the capability is scoped to the main window"
    );
}

#[test]
fn no_compare_command_accepts_a_path_a_table_name_or_a_statement() {
    // The requests are the surface. If any of them grew a free-form field this would fail, which is
    // the point: the shape is the permission boundary.
    let request = json(&SectionChangeQueryDto {
        diff_id: "cmp-1-1".to_owned(),
        ..Default::default()
    });
    let mut keys: Vec<&str> = request
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "changeKind",
            "diffId",
            "direction",
            "filter",
            "limit",
            "offset",
            "sort"
        ]
    );

    let compare_request = json(&CompareRequestDto {
        base_snapshot_id: "snap-a".to_owned(),
        target_snapshot_id: "snap-b".to_owned(),
    });
    let mut keys: Vec<&str> = compare_request
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, vec!["baseSnapshotId", "targetSnapshotId"]);
}
