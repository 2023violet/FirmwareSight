//! `fwsight analyze --record` (ADR-0042): an analysis stored in a local store, once, under the desktop's
//! project, without changing what stdout says.
//!
//! Every test names its own store with `--store`, so nothing here touches a real user's store.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use firmwaresight_core::domain::trend::TrendRegions;
use firmwaresight_storage::{Database, LOCAL_PROJECT_ID};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const ELF: &str = "fixtures/elf/p2-diff/target/firmware.elf";
const MAP: &str = "fixtures/elf/p2-diff/target/firmware.map";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("apps/cli lives at <root>/apps/cli")
        .to_path_buf()
}

fn run(args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_fwsight"))
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("the fwsight binary is built for its own tests");
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(!stderr.contains("panicked"), "the CLI panicked:\n{stderr}");
    output
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "fwsight-record-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }

    fn store(&self) -> String {
        self.0.join("store.sqlite").to_string_lossy().into_owned()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn a_recorded_analysis_is_stored_once_under_the_desktop_project() {
    let dir = TempDir::new();
    let store = dir.store();
    let first = run(&["analyze", ELF, "--map", MAP, "--record", "--store", &store]);
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));
    assert!(
        stderr(&first).contains("diagnostics: recorded build build-"),
        "{}",
        stderr(&first)
    );

    let again = run(&["analyze", ELF, "--map", MAP, "--record", "--store", &store]);
    assert_eq!(again.status.code(), Some(0));
    assert!(
        stderr(&again).contains("was already recorded"),
        "{}",
        stderr(&again)
    );

    let db = Database::open(Path::new(&store)).expect("opens");
    let trend = db
        .history_trend(&[LOCAL_PROJECT_ID.to_owned()], 100)
        .expect("reads");
    assert_eq!(trend.points.len(), 1, "one build, recorded once");
    assert!(matches!(trend.points[0].regions, TrendRegions::Recorded(_)));
    assert_eq!(trend.points[0].file_name, "firmware.elf");
}

#[test]
fn json_output_stays_one_document_when_recording() {
    let dir = TempDir::new();
    let store = dir.store();
    let plain = run(&["analyze", ELF, "--map", MAP, "--json"]);
    let recorded = run(&[
        "analyze", ELF, "--map", MAP, "--json", "--record", "--store", &store,
    ]);
    assert_eq!(recorded.status.code(), Some(0));
    assert_eq!(
        recorded.stdout, plain.stdout,
        "recording does not change stdout"
    );
    serde_json::from_slice::<serde_json::Value>(&recorded.stdout).expect("one JSON document");
}

#[test]
fn a_store_that_cannot_be_written_is_exit_6_with_one_error_document() {
    let dir = TempDir::new();
    // A directory where the store file should be.
    let blocked = dir.0.join("not-a-file");
    std::fs::create_dir_all(&blocked).expect("dir");
    let out = run(&[
        "analyze",
        ELF,
        "--json",
        "--record",
        "--store",
        &blocked.to_string_lossy(),
    ]);
    assert_eq!(out.status.code(), Some(6), "{}", stderr(&out));
    let doc: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout is the error document only");
    assert!(
        doc["code"]
            .as_str()
            .is_some_and(|code| code.starts_with("ERR-STORAGE-")),
        "{doc}"
    );
}

#[test]
fn store_needs_record_and_analysis_alone_writes_nothing() {
    let dir = TempDir::new();
    let store = dir.store();
    let out = run(&["analyze", ELF, "--store", &store]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "--store without --record is a usage error"
    );
    let out = run(&["analyze", ELF]);
    assert_eq!(out.status.code(), Some(0));
    assert!(!Path::new(&store).exists(), "no store was created");
    assert!(!stderr(&out).contains("recorded"));
}
