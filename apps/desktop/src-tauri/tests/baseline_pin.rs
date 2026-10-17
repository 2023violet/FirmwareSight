//! The pinned baseline across the IPC boundary (R5 / D3).
//!
//! The commands take a snapshot id the screen already holds and nothing else: no project, no path. These
//! tests hold the boundary's half of the contract — the scope is Rust's, an unknown snapshot is a typed
//! error rather than a silent pin, an unpin with nothing pinned changes nothing, and no directory crosses
//! in the DTO.

use std::path::{Path, PathBuf};

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
        let unique = format!(
            "fwsight-r5-pin-{label}-{}-{:x}.sqlite",
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

fn store_side(session: &Session, side: &str) -> String {
    let root = repo_root().join(format!("fixtures/elf/p2-diff/{side}"));
    let selection = session
        .stage_artifact(root.join("firmware.elf"), "op-stage")
        .expect("the committed fixture stages");
    session
        .stage_map(&selection.selection_id, root.join("firmware.map"), "op-map")
        .expect("the fixture MAP stages");
    session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("the fixture analyzes and stores")
        .identity
        .snapshot_id
}

#[test]
fn a_pin_round_trips_by_snapshot_id_and_carries_no_directory() {
    let db = TempDb::new("round-trip");
    let session =
        Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
    let base = store_side(&session, "base");
    let target = store_side(&session, "target");

    assert_eq!(session.baseline_pin("op-read").expect("reads"), None);

    let pin = session
        .pin_baseline(&base, "op-pin")
        .expect("pins the base");
    assert_eq!(pin.snapshot_id, base);
    assert_eq!(pin.file_name, "firmware.elf");
    let wire = serde_json::to_string(&pin).expect("serializes");
    assert!(
        !wire.contains(":\\\\") && !wire.contains(":/") && !wire.contains("fixtures"),
        "no path: {wire}"
    );

    let moved = session
        .pin_baseline(&target, "op-pin-2")
        .expect("pins the target");
    assert_eq!(
        session.baseline_pin("op-read-2").expect("reads"),
        Some(moved)
    );

    assert!(session.unpin_baseline("op-unpin").expect("unpins"));
    assert_eq!(session.baseline_pin("op-read-3").expect("reads"), None);
    assert!(
        !session
            .unpin_baseline("op-unpin-2")
            .expect("nothing to unpin")
    );
}

#[test]
fn an_unknown_snapshot_is_a_typed_refusal_not_a_pin() {
    let db = TempDb::new("unknown");
    let session =
        Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
    store_side(&session, "base");
    let refused = session
        .pin_baseline("snap-that-was-never-stored", "op-pin")
        .expect_err("an unknown snapshot is refused");
    assert_eq!(refused.code, "ERR-STORAGE-4005");
    assert_eq!(session.baseline_pin("op-read").expect("reads"), None);
}
