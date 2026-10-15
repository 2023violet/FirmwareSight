//! ADR-0033 D5: object rows round-trip through the store without a host path, are immutable, and a
//! build without them reads as "none recorded" rather than as zero.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::build_snapshot::BuildSnapshot;
use firmwaresight_storage::Database;

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

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Self(std::env::temp_dir().join(format!(
            "fwsight-objects-{label}-{}-{nanos:x}.sqlite",
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

#[test]
fn object_rows_round_trip_in_map_order_with_no_host_path() {
    let file = TempDb::new("round-trip");
    let mut db = Database::open(&file.0).expect("opens");
    let snapshot = snapshot(
        "m2-objects/target/firmware.elf",
        Some("m2-objects/target/firmware.map"),
    );
    assert!(!snapshot.objects().is_empty());
    let build = db.import_snapshot("p", "P", &snapshot).expect("imports");

    let read = db.object_contributions(&build).expect("reads");
    assert_eq!(
        read,
        snapshot.objects(),
        "every field and the MAP order come back unchanged"
    );

    let paths: i64 = db
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM object_contributions
              WHERE object_key LIKE '%:%' OR object_key LIKE '/%' OR object_key LIKE '%' || char(92) || '%'
                 OR member LIKE '%/%' OR archive LIKE '%/%'",
            [],
            |row| row.get(0),
        )
        .expect("counts");
    assert_eq!(paths, 0, "no stored name is a host path");
}

#[test]
fn stored_rows_are_immutable() {
    let file = TempDb::new("immutable");
    let mut db = Database::open(&file.0).expect("opens");
    let build = db
        .import_snapshot(
            "p",
            "P",
            &snapshot(
                "m2-objects/base/firmware.elf",
                Some("m2-objects/base/firmware.map"),
            ),
        )
        .expect("imports");
    let refused = db.connection().execute(
        "UPDATE object_contributions SET size = size + 1 WHERE build_id = ?1",
        [&build],
    );
    assert!(
        refused.is_err(),
        "an UPDATE of a stored object row is refused"
    );
}

#[test]
fn a_build_without_a_map_has_no_rows_rather_than_zero_rows_of_something() {
    let file = TempDb::new("no-map");
    let mut db = Database::open(&file.0).expect("opens");
    let snapshot = snapshot("m2-objects/target/firmware.elf", None);
    assert!(snapshot.objects().is_empty());
    let build = db.import_snapshot("p", "P", &snapshot).expect("imports");
    assert!(db.object_contributions(&build).expect("reads").is_empty());
}
