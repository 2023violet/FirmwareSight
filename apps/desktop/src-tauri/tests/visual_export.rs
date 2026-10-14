//! What the desktop may export as a visual report, and what it must refuse.
//!
//! Three promises live here, and none of them belongs to the report crate:
//!
//! - a rendered visual is a real document of its claimed format — the PNG carries its dimensions in
//!   its own header bytes, which is read here without asking the encoder that produced them;
//! - an export is bound to the result the shell actually displays. Staging another artifact or
//!   attaching a MAP spends the previous selection's right to export, because the bytes a person
//!   saved must be explainable by the screen they were looking at;
//! - the Gate surface names the remedy of its own surface, so a refused export says to run the Gate
//!   again and not to re-analyze a build.
//!
//! These tests call the same `Session` methods the commands do, with no window and no runtime, the
//! way the parity file does. The native dialog itself is not reachable without one; `compare.rs`
//! owns the cancel / keep-existing / changed-target outcomes around it.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::ipc::{GateRunRequestDto, VisualFormatDto};
use firmwaresight_desktop::{FixtureKey, Session, service::FixtureCatalog};

const STALE_CODE: &str = "ERR-EXPORT-6002";

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "fwsight-visual-export-{label}-{:x}-{:x}.sqlite",
            std::process::id(),
            nanos
        ));
        let _ = std::fs::remove_file(&path);
        Self(path)
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "fwsight-visual-export-{label}-{:x}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory creates");
        Self(path)
    }

    fn copy(&self, name: &str, source: &Path) -> PathBuf {
        let target = self.0.join(name);
        std::fs::copy(source, &target).unwrap_or_else(|err| {
            panic!("copy {} to {}: {err}", source.display(), target.display())
        });
        target
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn open_session(db: &TempDb) -> Session {
    Session::open(FixtureCatalog::from_environment(), db.0.clone())
        .unwrap_or_else(|err| panic!("desktop session opens: {err}"))
}

/// Stage one committed fixture, analyze it, and return the two ids the export commands are given.
fn analyze(session: &Session, scratch: &Scratch, label: &str, key: FixtureKey) -> (String, String) {
    let source = FixtureCatalog::from_environment().artifact_path(key);
    let artifact = scratch.copy(&format!("{label}-firmware.elf"), &source);
    let selection = session
        .stage_artifact(&artifact, &format!("op-stage-{label}"))
        .unwrap_or_else(|err| panic!("{label} stages: {}", err.code));
    let summary = session
        .analyze_selection(&selection.selection_id, &format!("op-analyze-{label}"))
        .unwrap_or_else(|err| panic!("{label} analyzes: {}", err.code));
    (selection.selection_id, summary.identity.snapshot_id)
}

#[test]
fn every_visual_format_the_desktop_exports_is_a_document_of_that_format() {
    let db = TempDb::new("formats");
    let scratch = Scratch::new("formats");
    let session = open_session(&db);
    let (selection_id, snapshot_id) = analyze(&session, &scratch, "fmt", FixtureKey::P0DualRegion);

    let svg = session
        .render_analysis_visual(&selection_id, &snapshot_id, VisualFormatDto::Svg, "op-svg")
        .expect("the displayed analysis exports SVG");
    let svg = std::str::from_utf8(&svg).expect("SVG is UTF-8");
    assert!(svg.starts_with("<svg "), "SVG must be a real SVG document");
    assert!(svg.contains("viewBox=\"0 0 1600 900\""));
    assert!(svg.ends_with("</svg>"));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(
        !svg.contains("<script"),
        "no executable markup crosses a file boundary"
    );

    let html = session
        .render_analysis_visual(
            &selection_id,
            &snapshot_id,
            VisualFormatDto::Html,
            "op-html",
        )
        .expect("the displayed analysis exports HTML");
    let html = std::str::from_utf8(&html).expect("HTML is UTF-8");
    assert!(html.starts_with("<!doctype html>"));
    assert!(
        html.contains("@page { size: A4;"),
        "the print sheet is part of the document"
    );
    assert!(html.contains("native PDF export is unavailable"));
    assert!(
        !html.contains("http://") && !html.contains("https://"),
        "no remote reference"
    );

    let png = session
        .render_analysis_visual(&selection_id, &snapshot_id, VisualFormatDto::Png, "op-png")
        .expect("the displayed analysis exports PNG");
    assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n", "PNG signature");
    assert_eq!(&png[12..16], b"IHDR", "IHDR is the first chunk");
    // The header is read as bytes, not through the encoder that wrote them: width and height are
    // big-endian at fixed offsets, and 8:truecolour+alpha is what a share card must be.
    let width = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
    let height = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
    assert_eq!((width, height), (1600, 900));
    assert_eq!(png[24], 8, "bit depth");
    assert_eq!(png[25], 6, "color type truecolour with alpha");
    assert_eq!(
        &png[png.len() - 8..png.len() - 4],
        b"IEND",
        "the file ends its chunk stream"
    );
    assert!(
        png.len() > 20_000,
        "a share card is bytes of pixels, not a stub"
    );
}

#[test]
fn attaching_a_map_spends_the_export_the_previous_analysis_held() {
    let db = TempDb::new("map-invalidates");
    let scratch = Scratch::new("map-invalidates");
    let session = open_session(&db);
    let (selection_id, snapshot_id) = analyze(&session, &scratch, "inv", FixtureKey::P0DualRegion);
    session
        .render_analysis_visual(
            &selection_id,
            &snapshot_id,
            VisualFormatDto::Svg,
            "op-before",
        )
        .expect("the analysis just displayed exports");

    let map_source = FixtureCatalog::from_environment()
        .artifact_path(FixtureKey::P0DualRegion)
        .parent()
        .expect("the fixture lives in a directory")
        .join("firmware.map");
    let map = scratch.copy("attached.map", &map_source);
    session
        .stage_map(&selection_id, &map, "op-map")
        .expect("the MAP stages");

    let refused = session
        .render_analysis_visual(
            &selection_id,
            &snapshot_id,
            VisualFormatDto::Svg,
            "op-after",
        )
        .expect_err("the bytes behind that snapshot are no longer what the screen holds");
    assert_eq!(refused.code, STALE_CODE);
}

#[test]
fn a_second_selection_makes_the_first_one_stale_for_export() {
    let db = TempDb::new("superseded");
    let scratch = Scratch::new("superseded");
    let session = open_session(&db);
    let (first_selection, first_snapshot) =
        analyze(&session, &scratch, "one", FixtureKey::P0DualRegion);
    session
        .render_analysis_visual(
            &first_selection,
            &first_snapshot,
            VisualFormatDto::Svg,
            "op-one",
        )
        .expect("the current selection exports");

    let (second_selection, second_snapshot) =
        analyze(&session, &scratch, "two", FixtureKey::P0Basic);
    assert_ne!(
        first_snapshot, second_snapshot,
        "the two fixtures differ by construction"
    );
    let refused = session
        .render_analysis_visual(
            &first_selection,
            &first_snapshot,
            VisualFormatDto::Svg,
            "op-two",
        )
        .expect_err("only the newest selection may export");
    assert_eq!(refused.code, STALE_CODE);
    session
        .render_analysis_visual(
            &second_selection,
            &second_snapshot,
            VisualFormatDto::Svg,
            "op-three",
        )
        .expect("the newest selection exports");
}

#[test]
fn the_gate_export_is_bound_to_the_run_the_shell_last_displayed() {
    let db = TempDb::new("gate-current");
    let scratch = Scratch::new("gate-current");
    let session = open_session(&db);
    let (_, target) = analyze(&session, &scratch, "gate-target", FixtureKey::P0DualRegion);
    let (_, baseline) = analyze(&session, &scratch, "gate-baseline", FixtureKey::P0Basic);

    let first = session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: target.clone(),
                baseline_snapshot_id: Some(baseline.clone()),
            },
            "op-gate-one",
        )
        .expect("the Gate run completes");
    assert!(
        session
            .gate_run_is_current(&first.run_id, &first.snapshot_id, "op-check-one")
            .expect("the check itself does not fail"),
        "a run the shell just produced is the run it displays"
    );
    for format in [
        VisualFormatDto::Svg,
        VisualFormatDto::Png,
        VisualFormatDto::Html,
    ] {
        let bytes = session
            .render_gate_visual(&first.run_id, &first.snapshot_id, format, "op-render")
            .expect("the displayed Gate run renders in every supported format");
        assert!(!bytes.is_empty());
    }

    let second = session
        .run_release_gate(
            &GateRunRequestDto {
                snapshot_id: baseline.clone(),
                baseline_snapshot_id: Some(target.clone()),
            },
            "op-gate-two",
        )
        .expect("the reversed pair is a different run");
    assert_ne!(first.run_id, second.run_id);
    assert!(
        !session
            .gate_run_is_current(&first.run_id, &first.snapshot_id, "op-check-two")
            .expect("the check itself does not fail"),
        "the superseded run must not export as if it were still on screen"
    );
    assert!(
        session
            .gate_run_is_current(&second.run_id, &second.snapshot_id, "op-check-three")
            .expect("the check itself does not fail")
    );

    let refused = session
        .gate_run_is_current(&first.run_id, "snap-never-displayed", "op-check-four")
        .expect("a mismatched snapshot is an answer, not a failure");
    assert!(!refused, "the pair, not the run id alone, decides");
}

#[test]
fn an_unknown_gate_run_or_snapshot_is_refused_before_any_file_is_chosen() {
    let db = TempDb::new("gate-unknown");
    let scratch = Scratch::new("gate-unknown");
    let session = open_session(&db);
    let (_, target) = analyze(&session, &scratch, "known", FixtureKey::P0DualRegion);

    let refused = session
        .render_gate_visual(
            "gate-never-run",
            &target,
            VisualFormatDto::Svg,
            "op-unknown-run",
        )
        .expect_err("a run id this session never produced is not an empty report");
    assert_eq!(refused.code, STALE_CODE);
    assert_eq!(refused.details, None, "a refusal names no host path");

    let refused = session
        .render_gate_visual(
            "gate-never-run",
            "snap-never-stored",
            VisualFormatDto::Png,
            "op-unknown-both",
        )
        .expect_err("both halves unknown is still a refusal");
    assert_eq!(refused.code, STALE_CODE);
    assert!(
        refused
            .remediation
            .as_deref()
            .is_some_and(|text| text.contains("Gate")),
        "the Gate surface names the remedy of the Gate surface"
    );
    assert!(
        !target.is_empty(),
        "the analyzed build stays stored for the run the next assertion takes"
    );
}
