//! P5 History over the shell boundary: what three bounded reads are allowed to say about rows this
//! application stored earlier, possibly in an earlier run.
//!
//! The storage layer owns paging, scoping and the path reduction, and Core owns every fact, so this
//! file is about the *boundary* only:
//!
//! - a page is scoped to the one project the shell files user artifacts under, and the WebView cannot
//!   widen it (prompt §44, `AGENTS.md` 7);
//! - a page is bounded whatever the caller asked for, and the limit reported is the limit applied
//!   (prompt §17);
//! - no host path crosses, in any field or in the JSON a WebView would receive (prompt §16);
//! - a stored verdict is surfaced, never recomputed: the disposition and the counts a History row
//!   carries are the ones the Gate run itself reported;
//! - and the window title is Rust's, from a closed page enum, with the capability list unchanged.
//!
//! As in the Compare and Gate files, these call the same `Session` methods the commands do, with no
//! window and no runtime. Where a release record is needed, the row is written by
//! `firmwaresight-storage`'s own writer against the same file, which is how `bundle_ipc.rs` reads a
//! record back too: the point here is what a read may say about a persisted row, not how the Bundle
//! export produced it.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::ipc::{GateRunRequestDto, HistoryPageRequestDto, MainWindowPage};
use firmwaresight_desktop::{LOCAL_PROJECT_ID, Session, service::FixtureCatalog};
use firmwaresight_storage::{Database, ReleaseRecordDraft};
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
            "fwsight-p5-history-{label}-{}-{:x}.sqlite",
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

fn session(db: &TempDb) -> Session {
    Session::open(FixtureCatalog::from_environment(), db.path()).expect("a session opens")
}

/// Analyze and store one fixture side through the product path: stage, attach the MAP, analyze.
fn store_side(session: &Session, side: &str) -> String {
    let (artifact, map) = fixture(side);
    let selection = session
        .stage_artifact(&artifact, "op-stage")
        .expect("the committed fixture stages");
    session
        .stage_map(&selection.selection_id, &map, "op-map")
        .expect("the fixture MAP stages");
    session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("the fixture analyzes and stores")
        .identity
        .snapshot_id
}

fn request(
    offset: Option<usize>,
    limit: Option<usize>,
    filter: Option<&str>,
) -> HistoryPageRequestDto {
    HistoryPageRequestDto {
        offset,
        limit,
        filter: filter.map(str::to_owned),
    }
}

/// A 64-character lowercase hex value shaped like a real digest, so the schema's CHECKs are
/// exercised by name rather than skipped.
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

/// Write one release record through the storage writer, against the same file the session holds.
fn record_release(db: &TempDb, build_id: &str, gate_run_id: &str, tag: &str) -> String {
    let release_id = format!("release-{}", hex64(tag));
    Database::open(db.path())
        .expect("the same database reopens")
        .persist_release_record(&ReleaseRecordDraft {
            release_id: &release_id,
            build_id,
            baseline_build_id: None,
            gate_run_id,
            release_version: "1.2.3",
            manifest_sha256: &hex64(&format!("manifest-{tag}")),
        })
        .expect("a release record for a stored build and a stored run is writable");
    release_id
}

#[test]
fn an_empty_store_reads_three_empty_pages() {
    let db = TempDb::new("empty");
    let session = session(&db);

    let builds = session
        .list_history_builds(&request(None, None, None), "op-builds")
        .expect("an empty history is a page, not an error");
    let runs = session
        .list_history_gate_runs(&request(None, None, None), "op-runs")
        .expect("an empty gate history is a page");
    let releases = session
        .list_history_releases(&request(None, None, None), "op-releases")
        .expect("an empty release history is a page");

    for (label, total, rows, next) in [
        (
            "builds",
            builds.total,
            builds.rows.len(),
            builds.next_offset,
        ),
        ("gate runs", runs.total, runs.rows.len(), runs.next_offset),
        (
            "releases",
            releases.total,
            releases.rows.len(),
            releases.next_offset,
        ),
    ] {
        assert_eq!(total, 0, "{label}");
        assert_eq!(rows, 0, "{label}");
        assert_eq!(next, None, "{label} must say there is no next page");
    }
}

#[test]
fn history_pages_the_builds_this_application_stored() {
    let db = TempDb::new("builds");
    let session = session(&db);
    let base = store_side(&session, "base");
    let target = store_side(&session, "target");

    let page = session
        .list_history_builds(&request(None, None, None), "op-builds")
        .expect("two stored builds read back");
    assert_eq!(page.total, 2);
    assert_eq!(page.rows.len(), 2);
    assert_eq!(page.offset, 0);
    // The two sides were stored inside the same second, so the tie-break is the snapshot id; either
    // way the page must say which build is which, and never by a path.
    let ids: Vec<&str> = page
        .rows
        .iter()
        .map(|row| row.snapshot_id.as_str())
        .collect();
    assert!(
        ids.contains(&base.as_str()) && ids.contains(&target.as_str()),
        "ids: {ids:?}"
    );
    assert_eq!(
        page.next_offset, None,
        "everything that exists fit the page"
    );

    for row in &page.rows {
        // Both sides are analyzed where the committed fixtures lie, so both are stored under the
        // same file name. The snapshot id is what tells the two rows apart, which is exactly why a
        // name is a safe thing to show and a path is not.
        assert_eq!(
            row.file_name, "firmware.elf",
            "name only: {}",
            row.file_name
        );
        assert_ne!(row.build_id, row.snapshot_id, "two different identities");
        assert_eq!(row.sha256.len(), 64);
        assert!(row.byte_size > 0);
        assert!(!row.architecture.is_empty());
        assert!(
            !row.imported_at.is_empty(),
            "a stored time is surfaced as stored"
        );
    }
}

#[test]
fn a_stored_gate_run_is_surfaced_exactly_as_the_gate_reported_it() {
    let db = TempDb::new("gate");
    let session = session(&db);
    let target = store_side(&session, "target");

    let run = session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: target.clone(),
                baseline_snapshot_id: None,
            },
            "op-gate",
        )
        .expect("the run completes");

    let page = session
        .list_history_gate_runs(&request(None, None, None), "op-runs")
        .expect("the run is in history");
    assert_eq!(page.total, 1);
    let row = &page.rows[0];
    assert_eq!(row.run_id, run.run_id);
    assert_eq!(
        row.disposition, run.overall_effective_severity,
        "not recomputed"
    );
    assert_eq!(row.counts, run.counts, "the stored counts, unchanged");
    assert_eq!(
        row.counts.pass
            + row.counts.review
            + row.counts.block
            + row.counts.unknown
            + row.counts.not_applicable,
        run.findings.len(),
        "every finding is counted once"
    );
    assert_eq!(row.stored_at, run.created_at, "the run's own audit time");
    assert_eq!(
        row.file_name, "firmware.elf",
        "the build's name, never its directory"
    );
    assert_eq!(row.baseline_build_id, None, "this run named no baseline");
}

#[test]
fn a_release_record_reads_back_with_the_digest_that_ties_it() {
    let db = TempDb::new("release");
    let session = session(&db);
    let target = store_side(&session, "target");
    let run = session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: target.clone(),
                baseline_snapshot_id: None,
            },
            "op-gate",
        )
        .expect("the run completes");

    let stored = Database::open(db.path())
        .expect("reopen")
        .build_id_for_snapshot(&target)
        .expect("build lookup")
        .expect("the analyzed build is stored");
    let release_id = record_release(&db, &stored, &run.run_id, "one");

    let page = session
        .list_history_releases(&request(None, None, None), "op-releases")
        .expect("the record is in history");
    assert_eq!(page.total, 1);
    let row = &page.rows[0];
    assert_eq!(row.release_id, release_id);
    assert_eq!(row.release_version, "1.2.3");
    assert_eq!(row.build_id, stored);
    assert_eq!(row.gate_run_id, run.run_id, "the run it was qualified by");
    assert_eq!(row.manifest_sha256, hex64("manifest-one"));
    assert_eq!(row.file_name, "firmware.elf");
}

#[test]
fn no_history_page_ever_carries_a_host_path() {
    let db = TempDb::new("path");
    let session = session(&db);
    let target = store_side(&session, "target");
    let base = store_side(&session, "base");
    let run = session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: target.clone(),
                baseline_snapshot_id: None,
            },
            "op-gate",
        )
        .expect("the run completes");
    let stored = Database::open(db.path())
        .expect("reopen")
        .build_id_for_snapshot(&base)
        .expect("build lookup")
        .expect("the base build is stored");
    record_release(&db, &stored, &run.run_id, "two");

    // What the WebView would actually receive: the serialized page, not the Rust debug form.
    let pages = [
        serde_json::to_value(
            session
                .list_history_builds(&request(None, None, None), "op-1")
                .expect("builds"),
        )
        .expect("json"),
        serde_json::to_value(
            session
                .list_history_gate_runs(&request(None, None, None), "op-2")
                .expect("runs"),
        )
        .expect("json"),
        serde_json::to_value(
            session
                .list_history_releases(&request(None, None, None), "op-3")
                .expect("releases"),
        )
        .expect("json"),
    ];
    // A fragment of the directory the artifacts were really read from. If it appears anywhere in
    // what the WebView would receive, the boundary has leaked a path.
    let directory = "p2-diff";
    for page in &pages {
        let text = page.to_string();
        assert!(
            !text.contains("fixtures"),
            "a stored directory crossed: {text}"
        );
        assert!(
            !text.contains(directory),
            "a stored directory crossed: {text}"
        );
        for (key, value) in string_fields(page) {
            assert!(
                !value.contains('/') && !value.contains('\\'),
                "{key} carried a separator: {value}"
            );
            assert!(
                !value.contains(directory) && !value.contains("fixtures"),
                "{key} carried a stored directory: {value}"
            );
        }
    }
}

/// Every string value in a serialized page, with the field name it arrived under.
fn string_fields(value: &Value) -> Vec<(String, String)> {
    fn walk(prefix: &str, value: &Value, out: &mut Vec<(String, String)>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    let name = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    walk(&name, child, out);
                }
            }
            Value::Array(items) => {
                for item in items {
                    walk(prefix, item, out);
                }
            }
            Value::String(text) => out.push((prefix.to_owned(), text.clone())),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk("", value, &mut out);
    out
}

#[test]
fn a_history_page_is_bounded_by_rust_not_by_the_caller() {
    let db = TempDb::new("clamp");
    let session = session(&db);
    store_side(&session, "target");

    let asked = session
        .list_history_builds(&request(Some(0), Some(usize::MAX), None), "op-big")
        .expect("an absurd page is clamped, not honoured");
    assert_eq!(asked.limit, 100, "the store's ceiling, reported as applied");
    assert_eq!(asked.rows.len(), 1);

    let defaulted = session
        .list_history_builds(&request(None, None, None), "op-default")
        .expect("no page size means the default");
    assert_eq!(defaulted.limit, 25);

    let beyond = session
        .list_history_builds(&request(Some(500), Some(25), None), "op-beyond")
        .expect("an offset past the end is an empty page, not an error");
    assert_eq!(beyond.rows.len(), 0);
    assert_eq!(beyond.total, 1, "and the total is still the truth");
    assert_eq!(beyond.next_offset, None);

    // The same ceilings answer for the record tables, so one screen cannot page three tables by
    // three different amounts.
    let runs = session
        .list_history_gate_runs(&request(None, Some(usize::MAX), None), "op-runs")
        .expect("clamped");
    assert_eq!(runs.limit, 100);
    let releases = session
        .list_history_releases(&request(None, None, None), "op-releases")
        .expect("defaulted");
    assert_eq!(releases.limit, 25);
}

#[test]
fn a_filter_narrows_a_page_and_never_widens_the_scope() {
    let db = TempDb::new("filter");
    let session = session(&db);
    let base = store_side(&session, "base");
    let target = store_side(&session, "target");

    let one = session
        .list_history_builds(&request(None, None, Some(&target[..16])), "op-filter")
        .expect("a snapshot prefix filters the page");
    assert_eq!(one.total, 1);
    assert_eq!(one.rows[0].snapshot_id, target);

    let none = session
        .list_history_builds(
            &request(None, None, Some("no-such-build-anywhere")),
            "op-absent",
        )
        .expect("a filter that matches nothing is an empty page, not an error");
    assert_eq!(none.total, 0);
    assert_eq!(none.next_offset, None);

    // The directory the artifacts were really read from is not a searchable column, so a filter
    // cannot be used to ask whether some path exists.
    let probe = session
        .list_history_builds(&request(None, None, Some("fixtures")), "op-probe")
        .expect("a path fragment is refused by the read, not by an error");
    assert_eq!(probe.total, 0, "a stored directory was searchable");
    let by_parent = session
        .list_history_builds(&request(None, None, Some("p2-diff")), "op-probe-2")
        .expect("the intake directory is not searchable either");
    assert_eq!(
        by_parent.total, 0,
        "a stored parent directory was searchable"
    );

    // An empty filter is no filter, so the empty string cannot become "match everything but the
    // rows that have no text".
    let empty = session
        .list_history_builds(&request(None, None, Some("")), "op-empty")
        .expect("an empty filter is not a filter");
    assert_eq!(empty.total, 2);
    assert_eq!(empty.rows[0].snapshot_id.len(), base.len());
}

#[test]
fn a_reopened_session_reads_the_same_history() {
    let db = TempDb::new("reopen");
    let (run_id, release_id, snapshot) = {
        let session = session(&db);
        let target = store_side(&session, "target");
        let run = session
            .run_release_gate(
                &GateRunRequestDto {
                    snapshot_id: target.clone(),
                    baseline_snapshot_id: None,
                },
                "op-gate",
            )
            .expect("the run completes");
        let build = Database::open(db.path())
            .expect("reopen")
            .build_id_for_snapshot(&target)
            .expect("build lookup")
            .expect("stored");
        let release = record_release(&db, &build, &run.run_id, "three");
        (run.run_id, release, target)
    };

    // A new session over the same file: the point of History is that the reader did not create the
    // rows they are looking at.
    let session = session(&db);
    let runs = session
        .list_history_gate_runs(&request(None, None, None), "op-runs")
        .expect("history survived the close");
    assert_eq!(runs.rows[0].run_id, run_id);

    let releases = session
        .list_history_releases(&request(None, None, None), "op-releases")
        .expect("the release record survived too");
    assert_eq!(releases.rows[0].release_id, release_id);

    let builds = session
        .list_history_builds(&request(None, None, None), "op-builds")
        .expect("and the build");
    assert_eq!(builds.rows[0].snapshot_id, snapshot);
}

#[test]
fn the_history_scope_is_the_shells_own_project_and_cannot_be_asked_for() {
    let db = TempDb::new("scope");
    let session = session(&db);
    store_side(&session, "target");

    // A page reads one project: the one the shell files user artifacts under. The request shape has
    // no project field at all, which is the boundary, so this asserts the wire type and the read
    // agree.
    let mut fields: Vec<String> = serde_json::to_value(request(None, None, None))
        .expect("a request serializes")
        .as_object()
        .expect("an object")
        .keys()
        .cloned()
        .collect();
    fields.sort();
    let expected = vec!["filter".to_owned(), "limit".to_owned(), "offset".to_owned()];
    assert_eq!(
        fields, expected,
        "a History request carries a page and a filter, and no scope to widen"
    );

    let page = session
        .list_history_builds(&request(None, None, None), "op-builds")
        .expect("the local project's page");
    assert_eq!(page.total, 1);

    // The P0 demo history this application also knows how to write is not in it.
    let db_path = db.path().to_path_buf();
    let stored_project: String = Database::open(&db_path)
        .expect("reopen")
        .connection()
        .query_row(
            "SELECT project_id FROM builds ORDER BY id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("one stored build");
    assert_eq!(stored_project, LOCAL_PROJECT_ID);
}

#[test]
fn the_window_title_comes_from_a_closed_page_set_with_nothing_to_inject() {
    let pages = [
        MainWindowPage::Analyze,
        MainWindowPage::Compare,
        MainWindowPage::Release,
        MainWindowPage::History,
        MainWindowPage::Help,
    ];
    let titles: Vec<&str> = pages.iter().map(|page| page.window_title()).collect();
    assert_eq!(
        titles,
        vec![
            "FirmwareSight - Analyze",
            "FirmwareSight - Compare",
            "FirmwareSight - Release",
            "FirmwareSight - History",
            "FirmwareSight - Help",
        ],
        "the five pages the shell can be told about"
    );
    for title in titles {
        assert!(
            !title.contains('/') && !title.contains('\\') && !title.contains(':'),
            "a title carried a path separator: {title}"
        );
        assert!(
            title.starts_with("FirmwareSight - "),
            "brand first: {title}"
        );
    }

    // The wire form is the closed union, not a free string: five variants and no payload.
    let json: Vec<Value> = pages
        .iter()
        .map(|page| serde_json::to_value(page).expect("a page serializes"))
        .collect();
    assert_eq!(json[0], Value::String("Analyze".to_owned()));
    assert_eq!(json[4], Value::String("Help".to_owned()));
}

#[test]
fn the_title_fix_took_no_new_capability() {
    // `AGENTS.md` 9 makes a capability change a human decision, so the title is set by a Rust command
    // instead. This is the check that the claim is still true on this tree: the WebView holds exactly
    // `core:default`, which includes `core:window:allow-title` but not `allow-set-title`.
    let capability = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities/main.json"),
    )
    .expect("the capability file is part of the shell");
    let parsed: Value = serde_json::from_str(&capability).expect("the capability is JSON");
    let permissions = parsed
        .get("permissions")
        .and_then(Value::as_array)
        .expect("a permission list");
    let granted: Vec<&str> = permissions
        .iter()
        .map(|entry| entry.as_str().expect("a permission is a string"))
        .collect();
    assert_eq!(
        granted,
        vec!["core:default"],
        "capability unchanged: {granted:?}"
    );
    assert!(
        !capability.contains("allow-set-title"),
        "the window title would need a new permission"
    );
    assert!(
        !capability.contains("shell") && !capability.contains("fs"),
        "a filesystem or shell capability appeared: {capability}"
    );
}
