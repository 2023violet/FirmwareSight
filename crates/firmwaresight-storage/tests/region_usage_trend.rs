//! ADR-0041 / ADR-0042: per-region usage is stored with its build, the trend read returns stored builds
//! oldest first with the three region answers kept apart, and `record_snapshot` stores a build once even
//! when two writers race on the same file.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_core::domain::trend::{RegionCell, Trend, TrendRegions};
use firmwaresight_storage::{Database, RecordOutcome};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/elf")
        .join(name)
}

fn snapshot(elf: &str, map: Option<&str>) -> BuildSnapshot {
    let mut request = AnalysisRequest::new(fixture(elf));
    if let Some(map) = map {
        request = request.with_map(fixture(map));
    }
    pipeline::analyze(&request)
        .expect("the fixture analyzes")
        .snapshot
        .clone()
}

fn p2(side: &str) -> BuildSnapshot {
    snapshot(
        &format!("p2-diff/{side}/firmware.elf"),
        Some(&format!("p2-diff/{side}/firmware.map")),
    )
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Self(std::env::temp_dir().join(format!(
            "fwsight-trend-{label}-{}-{nanos:x}.sqlite",
            std::process::id()
        )))
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.0.display()));
        }
    }
}

const PROJECT: &str = "p";

fn scope() -> Vec<String> {
    vec![PROJECT.to_owned()]
}

#[test]
fn region_usage_round_trips_with_its_build() {
    let file = TempDb::new("round-trip");
    let mut db = Database::open(&file.0).expect("opens");
    let snap = p2("target");
    let expected = &snap.memory().expect("memory").region_usage;
    assert!(!expected.is_empty(), "a MAP-backed fixture has regions");
    db.import_snapshot(PROJECT, "P", &snap).expect("imports");

    let trend = db.history_trend(&scope(), 100).expect("reads");
    let TrendRegions::Recorded(stored) = &trend.points[0].regions else {
        panic!("regions recorded: {:?}", trend.points[0].regions);
    };
    assert_eq!(stored.len(), expected.len());
    for (row, region) in stored.iter().zip(expected) {
        assert_eq!(row.name, region.name);
        assert_eq!(row.kind, region.kind);
        assert_eq!(row.capacity, region.capacity_bytes);
        assert_eq!(row.used, region.used_bytes);
    }
}

#[test]
fn a_build_without_a_region_table_is_not_a_build_stored_before_0009() {
    let file = TempDb::new("three-answers");
    let mut db = Database::open(&file.0).expect("opens");
    db.import_snapshot(PROJECT, "P", &snapshot("p2-diff/base/firmware.elf", None))
        .expect("imports without a MAP");
    let older = db
        .import_snapshot(PROJECT, "P", &p2("base"))
        .expect("imports");
    db.import_snapshot(PROJECT, "P", &p2("target"))
        .expect("imports");
    // A build stored before migration 0009 has no set row. Removing one is how that store looks.
    db.connection()
        .execute(
            "DELETE FROM region_usage_sets WHERE build_id = ?1",
            [&older],
        )
        .expect("simulate an older build");

    let trend = db.history_trend(&scope(), 100).expect("reads");
    let kinds: Vec<&TrendRegions> = trend.points.iter().map(|p| &p.regions).collect();
    assert_eq!(kinds[0], &TrendRegions::NoRegionTable);
    assert_eq!(kinds[1], &TrendRegions::NotRecorded);
    assert!(matches!(kinds[2], TrendRegions::Recorded(_)));

    let builds: Vec<_> = trend.points.iter().map(|p| p.trend_build()).collect();
    let computed = Trend::compute(&builds, 5);
    let first = &computed.regions[0];
    assert_eq!(first.cells[0], RegionCell::NoRegionTable);
    assert_eq!(first.cells[1], RegionCell::NotRecorded);
    assert!(
        first.latest.is_some(),
        "headroom from the newest recorded build"
    );
}

#[test]
fn the_trend_reads_the_newest_builds_oldest_first_with_their_marks() {
    let file = TempDb::new("order");
    let mut db = Database::open(&file.0).expect("opens");
    let base = db
        .import_snapshot(PROJECT, "P", &p2("base"))
        .expect("imports");
    let target = db
        .import_snapshot(PROJECT, "P", &p2("target"))
        .expect("imports");
    let objects = db
        .import_snapshot(
            PROJECT,
            "P",
            &snapshot(
                "m2-objects/target/firmware.elf",
                Some("m2-objects/target/firmware.map"),
            ),
        )
        .expect("imports");
    db.import_snapshot("other", "O", &p2("base").clone())
        .expect_err("one snapshot is one build, whatever the project");
    let pin_snapshot = db
        .connection()
        .query_row(
            "SELECT snapshot_id FROM builds WHERE id = ?1",
            [&base],
            |r| r.get::<_, String>(0),
        )
        .expect("snapshot id");
    db.pin_baseline(PROJECT, &pin_snapshot).expect("pins");

    let all = db.history_trend(&scope(), 100).expect("reads");
    let order: Vec<&str> = all.points.iter().map(|p| p.build_id.as_str()).collect();
    assert_eq!(
        order,
        [base.as_str(), target.as_str(), objects.as_str()],
        "import order, oldest first"
    );
    assert_eq!(all.total_builds, 3);
    assert!(all.points[0].pinned && !all.points[1].pinned);
    assert!(
        !all.points[0].file_name.contains(['/', '\\']),
        "a file name, never a path"
    );
    assert!(
        all.points[2]
            .objects
            .as_ref()
            .is_some_and(|o| !o.is_empty())
    );

    let newest_two = db.history_trend(&scope(), 2).expect("reads");
    let order: Vec<&str> = newest_two
        .points
        .iter()
        .map(|p| p.build_id.as_str())
        .collect();
    assert_eq!(
        order,
        [target.as_str(), objects.as_str()],
        "the limit keeps the newest"
    );
    assert_eq!((newest_two.total_builds, newest_two.limit), (3, 2));

    assert!(
        db.history_trend(&[], 100).expect("reads").points.is_empty(),
        "no scope, no rows"
    );
    assert!(
        db.history_trend(&["other".to_owned()], 100)
            .expect("reads")
            .points
            .is_empty()
    );
}

#[test]
fn stored_region_usage_cannot_be_edited() {
    let file = TempDb::new("immutable");
    let mut db = Database::open(&file.0).expect("opens");
    db.import_snapshot(PROJECT, "P", &p2("target"))
        .expect("imports");
    for sql in [
        "UPDATE region_usage SET used_bytes = 0",
        "UPDATE region_usage_sets SET region_count = 0",
    ] {
        let err = db.connection().execute(sql, []).expect_err("refused");
        assert!(err.to_string().contains("immutable"), "{err}");
    }
}

#[test]
fn record_snapshot_stores_a_build_once() {
    let file = TempDb::new("record-once");
    let mut db = Database::open(&file.0).expect("opens");
    let snap = p2("target");
    let first = db.record_snapshot(PROJECT, "P", &snap).expect("records");
    assert!(matches!(first, RecordOutcome::Stored(_)));
    let again = db.record_snapshot(PROJECT, "P", &snap).expect("records");
    assert_eq!(
        again,
        RecordOutcome::AlreadyStored(first.build_id().to_owned())
    );
}

#[test]
fn two_writers_on_one_store_record_the_same_build_once() {
    let file = TempDb::new("race");
    // Create and migrate the store first, as the desktop does on start.
    drop(Database::open(&file.0).expect("opens"));
    let snap = p2("target");
    let other = p2("base");
    let path = file.0.clone();
    let writers: Vec<_> = (0..4)
        .map(|i| {
            let path = path.clone();
            let snap = if i % 2 == 0 {
                snap.clone()
            } else {
                other.clone()
            };
            std::thread::spawn(move || {
                let mut db = Database::open(&path).expect("opens");
                db.record_snapshot(PROJECT, "P", &snap)
                    .expect("records despite the other writer")
            })
        })
        .collect();
    let outcomes: Vec<RecordOutcome> = writers
        .into_iter()
        .map(|w| w.join().expect("no panic"))
        .collect();
    let stored = outcomes
        .iter()
        .filter(|o| matches!(o, RecordOutcome::Stored(_)))
        .count();
    assert_eq!(stored, 2, "each snapshot stored exactly once: {outcomes:?}");
    let db = Database::open(&file.0).expect("opens");
    assert_eq!(
        db.history_trend(&scope(), 100).expect("reads").points.len(),
        2
    );
}
