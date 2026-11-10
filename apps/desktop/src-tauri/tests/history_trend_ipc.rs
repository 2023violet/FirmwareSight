//! D8 History trend over the shell boundary (ADR-0041): the trend is read from stored builds through the
//! product path, oldest first, with Core's deltas and headroom, no host path in the JSON, and a store
//! reopened in a later run reading the same trend.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::ipc::HistoryTrendRequestDto;
use firmwaresight_desktop::{Session, service::FixtureCatalog};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("the desktop shell lives at <root>/apps/desktop/src-tauri")
        .to_path_buf()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "fwsight-d8-trend-{label}-{}-{:x}.sqlite",
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

fn session(db: &TempDb) -> Session {
    Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens")
}

fn store(session: &Session, elf: &str, map: Option<&str>) {
    let root = repo_root().join("fixtures/elf");
    let selection = session
        .stage_artifact(root.join(elf), "op-stage")
        .expect("stages");
    if let Some(map) = map {
        session
            .stage_map(&selection.selection_id, root.join(map), "op-map")
            .expect("MAP stages");
    }
    session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("analyzes and stores");
}

#[test]
fn the_trend_reads_stored_builds_oldest_first_with_core_deltas() {
    let db = TempDb::new("order");
    {
        let session = session(&db);
        store(
            &session,
            "p2-diff/base/firmware.elf",
            Some("p2-diff/base/firmware.map"),
        );
        store(
            &session,
            "p2-diff/target/firmware.elf",
            Some("p2-diff/target/firmware.map"),
        );
        // The same build again is the same stored build, not a third point.
        store(
            &session,
            "p2-diff/target/firmware.elf",
            Some("p2-diff/target/firmware.map"),
        );
    }

    // A later run of the application reads the same history.
    let session = session(&db);
    let trend = session
        .history_trend(&HistoryTrendRequestDto::default(), "op-trend")
        .expect("reads");
    assert_eq!(trend.points.len(), 2);
    assert_eq!(trend.total_builds, 2);
    let (base, target) = (&trend.points[0], &trend.points[1]);
    assert_eq!(
        base.flash.delta, None,
        "the first build has nothing to differ from"
    );
    assert_eq!(
        target.flash.delta,
        Some(120),
        "p2-diff grows Flash by 120 bytes"
    );
    assert_eq!(base.regions, "recorded");
    assert!(!trend.regions.is_empty(), "the MAP's regions become series");
    let latest = trend.regions[0].latest.as_ref().expect("headroom");
    assert_eq!(
        latest.point_index, 1,
        "headroom is read from the newest build"
    );

    let json = serde_json::to_string(&trend).expect("serializes");
    assert!(!json.contains("fixtures"), "no host path crosses: {json}");
    assert!(!json.contains('\\'), "no Windows path crosses");
}

#[test]
fn a_build_without_a_map_reads_no_region_table_and_the_limit_is_clamped() {
    let db = TempDb::new("no-map");
    let session = session(&db);
    store(&session, "p2-diff/base/firmware.elf", None);
    let trend = session
        .history_trend(
            &HistoryTrendRequestDto {
                limit: Some(usize::MAX),
            },
            "op-trend",
        )
        .expect("reads");
    assert_eq!(trend.points[0].regions, "noRegionTable");
    assert!(trend.regions.is_empty());
    assert!(trend.movers_flash.is_empty(), "one build moves nothing");
    assert_eq!(trend.limit, 500, "the ceiling, not the request");
}

#[test]
fn an_empty_store_has_an_empty_trend() {
    let db = TempDb::new("empty");
    let trend = session(&db)
        .history_trend(&HistoryTrendRequestDto::default(), "op-trend")
        .expect("reads");
    assert!(trend.points.is_empty() && trend.regions.is_empty());
}

#[test]
fn the_cli_resolves_the_store_this_application_opens() {
    // ADR-0042 D2: `fwsight analyze --record` writes `<data dir>/<identifier>/<file>`. The identifier is
    // this application's, so a renamed identifier would silently split the two histories.
    let conf: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("apps/desktop/src-tauri/tauri.conf.json"))
            .expect("tauri.conf.json"),
    )
    .expect("valid JSON");
    assert_eq!(
        conf["identifier"].as_str(),
        Some(firmwaresight_storage::DESKTOP_APP_IDENTIFIER)
    );
    assert_eq!(
        firmwaresight_desktop::LOCAL_PROJECT_ID,
        firmwaresight_storage::LOCAL_PROJECT_ID
    );
}
