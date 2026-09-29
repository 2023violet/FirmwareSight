//! The P2 goldens: the portable diff document and the HTML it renders into.
//!
//! Same rule as the P0 goldens (`apps/cli/tests/golden.rs`): a failing comparison never rewrites
//! its own expectation. Regeneration is a separate, reviewed step
//! (`python scripts/update_goldens.py --confirm`), so this file can only ever report a change.
//!
//! The two documents are the whole output, not an excerpt. Trimming either one would hide a change
//! in the part that was dropped, and the part that was dropped is the ambiguous mapping-symbol rows
//! a reader most needs to see.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::diff::{DiffSnapshotInput, compare};
use firmwaresight_report::diff::DiffResultDto;
use firmwaresight_report::{diff_render, render};

const BASE_ELF: &str = "fixtures/elf/p2-diff/base/firmware.elf";
const BASE_MAP: &str = "fixtures/elf/p2-diff/base/firmware.map";
const TARGET_ELF: &str = "fixtures/elf/p2-diff/target/firmware.elf";
const TARGET_MAP: &str = "fixtures/elf/p2-diff/target/firmware.map";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|parent| parent.parent())
        .expect("apps/cli lives at <root>/apps/cli")
        .to_path_buf()
}

/// The comparison the goldens were cut from: the dedicated P2 pair, both sides MAP-backed.
fn comparison() -> firmwaresight_core::domain::diff::DiffResult {
    let root = repo_root();
    let base =
        pipeline::analyze(&AnalysisRequest::new(root.join(BASE_ELF)).with_map(root.join(BASE_MAP)))
            .expect("the base fixture must analyze");
    let target = pipeline::analyze(
        &AnalysisRequest::new(root.join(TARGET_ELF)).with_map(root.join(TARGET_MAP)),
    )
    .expect("the target fixture must analyze");

    compare(
        &DiffSnapshotInput::from_snapshot(&base.snapshot),
        &DiffSnapshotInput::from_snapshot(&target.snapshot),
    )
    .expect("the fixture pair is comparable")
}

fn read(golden_rel: &str) -> String {
    let path = repo_root().join(golden_rel);
    assert!(
        path.exists(),
        "{golden_rel} is missing. Regenerate goldens only via scripts/update_goldens.py \
         --confirm and review the semantic diff."
    );
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{golden_rel} must read: {error}"))
}

#[test]
fn the_portable_diff_document_matches_the_committed_golden() {
    let dto = DiffResultDto::from_diff(&comparison());
    let actual = render::to_json_pretty(&dto);
    let expected = read("golden/core/p2-diff.json");
    assert!(
        render::semantically_equal(&expected, &actual),
        "golden/core/p2-diff.json differs from the current output.\n--- golden ---\n{}\n--- actual ---\n{}",
        render::canonical_form(&expected).unwrap_or_default(),
        render::canonical_form(&actual).unwrap_or_default(),
    );
}

#[test]
fn the_html_export_matches_the_committed_golden_byte_for_byte() {
    let dto = DiffResultDto::from_diff(&comparison());
    let actual = diff_render::render_html(&dto);
    let expected = read("golden/reports/p2-diff.html");
    assert_eq!(
        expected, actual,
        "golden/reports/p2-diff.html differs from the rendered HTML"
    );
}

#[test]
fn the_golden_carries_signed_totals_for_both_budgets() {
    let document: serde_json::Value =
        serde_json::from_str(&read("golden/core/p2-diff.json")).expect("the golden is JSON");
    let memory = &document["memory"];

    assert_eq!(memory["nonvolatile"]["base"], 256);
    assert_eq!(memory["nonvolatile"]["target"], 376);
    assert_eq!(memory["nonvolatile"]["delta"], 120);
    assert_eq!(memory["runtimeRam"]["base"], 8);
    assert_eq!(memory["runtimeRam"]["target"], 76);
    assert_eq!(memory["runtimeRam"]["delta"], 68);
    assert_eq!(memory["comparability"], "exact");
    assert_eq!(memory["nonvolatile"]["delta"].as_i64().unwrap(), 120);
    assert!(
        memory["nonvolatile"]["delta"].as_i64().unwrap() > 0,
        "a growth delta is signed, not an absolute value"
    );
}

#[test]
fn the_golden_shows_added_removed_and_changed_rows_for_sections_and_symbols() {
    let document: serde_json::Value =
        serde_json::from_str(&read("golden/core/p2-diff.json")).expect("the golden is JSON");

    for kind in ["sectionChanges", "symbolChanges"] {
        let rows = document[kind].as_array().expect("rows");
        let kinds: Vec<&str> = rows
            .iter()
            .filter_map(|row| row["change"].as_str())
            .collect();
        for expected in ["Added", "Removed", "Changed"] {
            assert!(
                kinds.contains(&expected),
                "{kind} must demonstrate {expected}, found {kinds:?}"
            );
        }
    }

    let sections = document["sectionChanges"].as_array().expect("sections");
    let added = sections
        .iter()
        .find(|row| row["key"] == ".ota")
        .expect("the target-only section");
    assert_eq!(added["change"], "Added");
    let removed = sections
        .iter()
        .find(|row| row["key"] == ".calib")
        .expect("the base-only section");
    assert_eq!(removed["change"], "Removed");
    let changed = sections
        .iter()
        .find(|row| row["key"] == ".text")
        .expect("the grown code section");
    assert_eq!(changed["change"], "Changed");
    assert_eq!(changed["fileSize"]["delta"], 88);
}

#[test]
fn absence_is_never_recorded_as_zero_in_the_golden() {
    let document: serde_json::Value =
        serde_json::from_str(&read("golden/core/p2-diff.json")).expect("the golden is JSON");
    let sections = document["sectionChanges"].as_array().expect("sections");

    let added = sections
        .iter()
        .find(|row| row["key"] == ".ota")
        .expect("the added section");
    assert_eq!(added["base"], serde_json::Value::Null, "no base row exists");
    assert_eq!(added["fileSize"]["base"], serde_json::Value::Null);
    assert_eq!(added["fileSize"]["delta"], serde_json::Value::Null);

    let removed = sections
        .iter()
        .find(|row| row["key"] == ".calib")
        .expect("the removed section");
    assert_eq!(removed["target"], serde_json::Value::Null);
    assert_eq!(removed["fileSize"]["target"], serde_json::Value::Null);
    assert_eq!(removed["fileSize"]["delta"], serde_json::Value::Null);

    // The two are counted separately from Changed rows, so a reader cannot add them up wrong.
    let counts = &document["counts"];
    assert_eq!(counts["sectionsAdded"], 1);
    assert_eq!(counts["sectionsRemoved"], 1);
    assert_eq!(counts["sectionsAmbiguous"], 0);
}

#[test]
fn object_attribution_is_unavailable_and_says_why() {
    let document: serde_json::Value =
        serde_json::from_str(&read("golden/core/p2-diff.json")).expect("the golden is JSON");
    assert_eq!(document["objectChanges"]["available"], false);
    assert!(
        document["objectChanges"]["reason"]
            .as_str()
            .expect("reason")
            .contains("object-file")
    );
}

#[test]
fn both_sides_are_named_with_their_own_identity() {
    let document: serde_json::Value =
        serde_json::from_str(&read("golden/core/p2-diff.json")).expect("the golden is JSON");
    assert_eq!(
        document["base"]["artifact"]["sha256"],
        "3f615b6247179d930f59b76dcdbaea1a5eca8c835ab6d410feb20a062f33f21f"
    );
    assert_eq!(
        document["target"]["artifact"]["sha256"],
        "4b4087e1407ceddcb1e1cfcd978eedbc8906cf88083ba3b4672c7c225ddd8374"
    );
    assert_ne!(
        document["base"]["snapshotId"],
        document["target"]["snapshotId"]
    );
    assert_eq!(document["base"]["memory"]["evidence"]["mapBacked"], true);
    assert_eq!(
        document["base"]["memory"]["evidence"]["weakestEvidenceBasis"],
        "MapRegionAndElfLoad"
    );
}

#[test]
fn neither_golden_carries_a_wall_clock_value_or_a_host_path() {
    // Two renders an instant apart producing the same bytes is the only practical test for "no
    // timestamp": a single document cannot prove the absence of a clock on its own.
    let first = render::to_json_pretty(&DiffResultDto::from_diff(&comparison()));
    let second = render::to_json_pretty(&DiffResultDto::from_diff(&comparison()));
    assert_eq!(first, second);
    let html_a = diff_render::render_html(&DiffResultDto::from_diff(&comparison()));
    let html_b = diff_render::render_html(&DiffResultDto::from_diff(&comparison()));
    assert_eq!(html_a, html_b);

    // The footer says "no timestamp" in prose, so searching for that word would report its own
    // denial. What has to be absent is a value: a key that could hold a clock, or a date-shaped
    // string anywhere in either document.
    for needle in ["generatedAt", "generated_at", "Date(", "toISOString"] {
        assert!(!first.contains(needle), "the JSON mentions {needle}");
        assert!(!html_a.contains(needle), "the HTML mentions {needle}");
    }
    assert!(
        !carries_a_date_value(&first),
        "the JSON carries a date value"
    );
    assert!(
        !carries_a_date_value(&html_a),
        "the HTML carries a date value"
    );
    for marker in [
        BASE_ELF, BASE_MAP, "C:\\", "D:\\", "/home/", "/Users/", "/tmp/",
    ] {
        assert!(!first.contains(marker), "the JSON carries {marker}");
        assert!(!html_a.contains(marker), "the HTML carries {marker}");
    }
}

/// `YYYY-MM-DD`, optionally with a time after it, anywhere in the text. Enough to catch a generated
/// timestamp and cheap enough to keep in a test.
fn carries_a_date_value(text: &str) -> bool {
    text.as_bytes().windows(10).any(|window| {
        window[0] == b'2'
            && window[1] == b'0'
            && window[2].is_ascii_digit()
            && window[3].is_ascii_digit()
            && window[4] == b'-'
            && window[5].is_ascii_digit()
            && window[6].is_ascii_digit()
            && window[7] == b'-'
            && window[8].is_ascii_digit()
            && window[9].is_ascii_digit()
    })
}

#[test]
fn the_html_golden_is_one_self_contained_file() {
    let html = read("golden/reports/p2-diff.html");
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<style>"), "the CSS is embedded");
    for forbidden in ["<script", "http://", "https://", "<link", "@import", "cdn"] {
        assert!(
            !html.to_ascii_lowercase().contains(forbidden),
            "the golden HTML references {forbidden}"
        );
    }
    assert!(
        html.contains("FirmwareSight 0.1.0"),
        "the version is visible"
    );
    assert!(html.contains("urn:firmwaresight:schema:diff:1"));
    assert!(html.contains("Delta = target"));
    assert!(
        html.contains("left unpaired because the name repeats"),
        "ambiguity stays in the open"
    );
}

#[test]
fn the_golden_pair_agrees_with_what_the_cli_writes() {
    // The goldens were cut by scripts/update_goldens.py from the binary. Reaching the same bytes
    // through the library here proves the two paths are one projection, not two implementations.
    let dto = DiffResultDto::from_diff(&comparison());
    assert!(render::semantically_equal(
        &read("golden/core/p2-diff.json"),
        &diff_render::render_json(&dto)
    ));
    assert_eq!(
        read("golden/reports/p2-diff.html"),
        diff_render::render_html(&dto)
    );
}
