//! The desktop's Markdown exports (R6), checked without a window: the renderers behind the three Save
//! dialogs produce the report grammar, carry no path, and refuse a subject that is no longer on screen.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::compare::ExportFormat;
use firmwaresight_desktop::ipc::CompareRequestDto;
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
            "fwsight-r6-markdown-{label}-{}-{:x}.sqlite",
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

/// Stage and analyze one fixture side; returns (selection id, snapshot id).
fn analyze(session: &Session, side: &str) -> (String, String) {
    let root = repo_root().join(format!("fixtures/elf/m2-objects/{side}"));
    let selection = session
        .stage_artifact(root.join("firmware.elf"), "op-stage")
        .expect("the committed fixture stages");
    session
        .stage_map(&selection.selection_id, root.join("firmware.map"), "op-map")
        .expect("the fixture MAP stages");
    let snapshot = session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("the fixture analyzes")
        .identity
        .snapshot_id;
    (selection.selection_id, snapshot)
}

#[test]
fn the_analysis_markdown_names_the_build_and_refuses_a_subject_no_longer_on_screen() {
    let db = TempDb::new("analysis");
    let session =
        Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
    let (selection, snapshot) = analyze(&session, "target");
    let doc = session
        .render_analysis_markdown(&selection, &snapshot, "op-md")
        .expect("renders");
    assert!(doc.starts_with("# FirmwareSight · Analysis report\n"));
    assert!(doc.contains("**Build:** firmware.elf"));
    assert!(
        !doc.contains("fixtures") && !doc.contains(":\\"),
        "no path:\n{doc}"
    );

    let stale = session
        .render_analysis_markdown(&selection, "snap-not-this-one", "op-md-2")
        .expect_err("a different snapshot is refused");
    assert_eq!(stale.code, "ERR-EXPORT-6002");
}

#[test]
fn the_comparison_markdown_comes_from_the_same_diff_the_screen_shows() {
    let db = TempDb::new("compare");
    let session =
        Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
    let (_, base) = analyze(&session, "base");
    let (_, target) = analyze(&session, "target");
    let summary = session
        .compare_snapshots(
            &CompareRequestDto {
                base_snapshot_id: base,
                target_snapshot_id: target,
            },
            "op-compare",
        )
        .expect("compares");
    let (doc, suggested) = session
        .render_export(&summary.diff_id, ExportFormat::Markdown, "op-md")
        .expect("renders");
    assert!(doc.starts_with("# FirmwareSight · Comparison report\n"));
    assert!(doc.contains("▲64 bytes"));
    assert!(suggested.ends_with(".md"), "{suggested}");
    assert!(!doc.contains("</"), "no HTML:\n{doc}");
}
