//! M2 object and library attribution over IPC (ADR-0033): what the boundary may say about which
//! object or library member a build's bytes came from, and how that changed.
//!
//! - an ELF-only build is "not available" with a reason, never a table of zeros;
//! - a MAP-backed build pages its objects and libraries, ranked by Core's charge, and its rows plus
//!   the stated remainder add up to the exact budget;
//! - Compare offers the delta only when both builds carry rows, and an object present on one side is
//!   Added or Removed with the other side `null`, never `0`;
//! - no key crossing the boundary is a host path, and a page never exceeds the bound.
//!
//! These call the same `Session` methods the commands do, with no window and no runtime.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::ipc::{
    ChangeKindFilterDto, CompareRequestDto, ObjectChangeQueryDto, ObjectChangeSortDto,
    ObjectRequestDto, ObjectSortDto, SortDirDto,
};
use firmwaresight_desktop::{Session, service::FixtureCatalog};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the desktop shell lives at <root>/apps/desktop/src-tauri")
        .to_path_buf()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "fwsight-m2-objects-{label}-{}-{:x}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_file(&path);
        Self(path)
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

/// Stage, attach the MAP when there is one, analyze and store, as the product path does.
fn store(session: &Session, side: &str, with_map: bool) -> String {
    let dir = repo_root().join(format!("fixtures/elf/m2-objects/{side}"));
    let selection = session
        .stage_artifact(dir.join("firmware.elf"), "op-stage")
        .expect("the committed fixture stages");
    if with_map {
        session
            .stage_map(&selection.selection_id, dir.join("firmware.map"), "op-map")
            .expect("the fixture MAP stages");
    }
    session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("the fixture analyzes and stores")
        .identity
        .snapshot_id
}

fn session(label: &str) -> (Session, TempDb) {
    let db = TempDb::new(label);
    let session =
        Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
    (session, db)
}

fn objects(snapshot_id: &str, libraries: bool) -> ObjectRequestDto {
    ObjectRequestDto {
        snapshot_id: snapshot_id.to_owned(),
        libraries,
        filter: None,
        sort: ObjectSortDto::Flash,
        direction: SortDirDto::Desc,
        offset: 0,
        limit: None,
    }
}

fn is_host_path(key: &str) -> bool {
    key.contains(':') || key.starts_with('/') || key.contains('\\')
}

#[test]
fn a_build_without_a_map_is_not_available_rather_than_all_zero() {
    let (session, _db) = session("elf-only");
    let snapshot = store(&session, "base", false);

    let page = session
        .query_objects(&objects(&snapshot, false), "op-objects")
        .expect("the query answers");
    assert!(!page.available);
    assert!(!page.map_present, "D6: no MAP is its own reason");
    assert!(
        page.reason
            .as_deref()
            .is_some_and(|reason| reason.contains("GNU ld MAP")),
        "{:?}",
        page.reason
    );
    assert!(page.rows.is_empty(), "no row is invented");
    assert_eq!(page.nonvolatile_total, None);
}

#[test]
fn a_map_backed_build_pages_ranked_objects_that_add_up_to_its_budget() {
    let (session, _db) = session("objects");
    let snapshot = store(&session, "target", true);

    let page = session
        .query_objects(&objects(&snapshot, false), "op-objects")
        .expect("the query answers");
    assert!(page.available, "{:?}", page.reason);
    assert!(page.map_present);
    assert!(page.rows.len() >= 3, "two app objects and library members");
    assert!(page.rows.iter().all(|row| !is_host_path(&row.key)));
    assert!(
        page.rows
            .windows(2)
            .all(|pair| pair[0].nonvolatile_bytes >= pair[1].nonvolatile_bytes),
        "Flash descending"
    );

    let flash: u64 = page
        .rows
        .iter()
        .filter_map(|row| row.nonvolatile_bytes)
        .sum();
    let total = page
        .nonvolatile_total
        .expect("a MAP-backed fixture is exact");
    let remainder = page
        .unattributed_nonvolatile
        .expect("an exact remainder is stated");
    assert_eq!(flash + remainder, total, "rows and remainder reconcile");

    let libraries = session
        .query_objects(&objects(&snapshot, true), "op-libraries")
        .expect("the query answers");
    assert_eq!(
        libraries
            .rows
            .iter()
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        ["libdrv.a"],
        "application objects are not grouped under a made-up library"
    );
    assert_eq!(libraries.rows[0].archive, None);
    let members: u64 = page
        .rows
        .iter()
        .filter(|row| row.archive.as_deref() == Some("libdrv.a"))
        .filter_map(|row| row.nonvolatile_bytes)
        .sum();
    assert_eq!(libraries.rows[0].nonvolatile_bytes, Some(members));
}

#[test]
fn an_object_page_filters_and_clamps_without_hiding_a_row() {
    let (session, _db) = session("paging");
    let snapshot = store(&session, "target", true);

    let mut request = objects(&snapshot, false);
    request.filter = Some("LIBDRV".to_owned());
    let filtered = session
        .query_objects(&request, "op-filter")
        .expect("the query answers");
    assert!(filtered.total >= 2);
    assert!(
        filtered
            .rows
            .iter()
            .all(|row| row.key.starts_with("libdrv.a("))
    );

    request.filter = None;
    request.limit = Some(1);
    let first = session
        .query_objects(&request, "op-page")
        .expect("the query answers");
    assert_eq!(first.rows.len(), 1);
    assert_eq!(first.next_offset, Some(1));

    request.limit = Some(100_000);
    let clamped = session
        .query_objects(&request, "op-clamp")
        .expect("the query answers");
    assert!(
        clamped.limit <= 500,
        "the bound is Rust's, not the caller's"
    );
}

#[test]
fn compare_lists_object_and_library_changes_with_absence_never_zero() {
    let (session, _db) = session("compare");
    let base = store(&session, "base", true);
    let target = store(&session, "target", true);

    let summary = session
        .compare_snapshots(
            &CompareRequestDto {
                base_snapshot_id: base,
                target_snapshot_id: target,
            },
            "op-compare",
        )
        .expect("the pair compares");
    let attribution = &summary.object_changes;
    assert!(attribution.available, "{}", attribution.reason);
    assert!(attribution.objects.changed >= 1, "the IMU object grew");
    assert!(attribution.objects.added >= 1, "spi.o was linked in");
    assert!(attribution.top_objects.len() <= 5);
    assert!(
        attribution
            .top_objects
            .iter()
            .any(|row| row.key.contains("imu.o")),
        "{:?}",
        attribution.top_objects
    );
    assert!(
        attribution
            .added_objects
            .iter()
            .any(|row| row.key == "libdrv.a(spi.o)")
    );

    let mut request = ObjectChangeQueryDto {
        diff_id: summary.diff_id.clone(),
        libraries: false,
        filter: None,
        change_kind: Some(ChangeKindFilterDto::Added),
        sort: ObjectChangeSortDto::FlashDelta,
        direction: SortDirDto::Desc,
        offset: 0,
        limit: None,
    };
    let added = session
        .query_object_changes(&request, "op-added")
        .expect("the page answers");
    let spi = added
        .rows
        .iter()
        .find(|row| row.key == "libdrv.a(spi.o)")
        .expect("the added member is listed");
    assert_eq!(spi.change_kind, "Added");
    assert_eq!(spi.nonvolatile.base, None, "absent in base, not zero");
    assert_eq!(spi.nonvolatile.delta, None);
    assert!(spi.nonvolatile.target.is_some_and(|bytes| bytes > 0));

    request.change_kind = None;
    let all = session
        .query_object_changes(&request, "op-all")
        .expect("the page answers");
    let measured = all
        .rows
        .iter()
        .take_while(|row| row.nonvolatile.delta.is_some())
        .count();
    assert!(
        all.rows[measured..]
            .iter()
            .all(|row| row.nonvolatile.delta.is_none()),
        "rows without a delta sort last, never as small numbers"
    );
    assert!(all.rows.iter().all(|row| !is_host_path(&row.key)));

    request.libraries = true;
    let libraries = session
        .query_object_changes(&request, "op-libraries")
        .expect("the page answers");
    assert_eq!(libraries.rows.len(), 1);
    assert_eq!(libraries.rows[0].key, "libdrv.a");
    assert!(
        libraries.rows[0]
            .nonvolatile
            .delta
            .is_some_and(|delta| delta > 0)
    );
}

#[test]
fn compare_with_one_side_unattributed_says_which_side() {
    let (session, _db) = session("one-sided");
    let base = store(&session, "base", false);
    let target = store(&session, "target", true);

    let summary = session
        .compare_snapshots(
            &CompareRequestDto {
                base_snapshot_id: base,
                target_snapshot_id: target,
            },
            "op-compare",
        )
        .expect("the pair compares");
    assert!(!summary.object_changes.available);
    assert!(
        summary.object_changes.reason.starts_with("the base build"),
        "{}",
        summary.object_changes.reason
    );
    assert!(summary.object_changes.top_objects.is_empty());
}
