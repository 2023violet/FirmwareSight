//! Golden comparison for the CLI projection.
//!
//! Goldens are regenerated only by an explicit, reviewed step. A failing comparison never
//! rewrites the file, because "test failed, update golden, green" would defeat the purpose of
//! having a golden (`05_ENGINEERING/02_TEST_STRATEGY.md`).

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_report::{AnalyzeResultDto, render};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("apps/cli lives at <root>/apps/cli")
        .to_path_buf()
}

fn analyze(elf: &str, map: Option<&str>) -> String {
    let root = repo_root();
    let mut request = AnalysisRequest::new(root.join(elf));
    if let Some(map) = map {
        request = request.with_map(root.join(map));
    }
    let analysis = pipeline::analyze(&request).expect("fixture must analyze");
    let name = Path::new(elf)
        .file_name()
        .expect("fixture path has a file name")
        .to_string_lossy();
    let dto = AnalyzeResultDto::from_snapshot(&analysis.snapshot, &name);
    render::to_json_pretty(&dto)
}

fn check(golden_rel: &str, actual: &str) {
    let path = repo_root().join(golden_rel);
    assert!(
        path.exists(),
        "{golden_rel} is missing. Regenerate goldens only via scripts/update_goldens.py \
         --confirm and review the semantic diff."
    );
    let expected = std::fs::read_to_string(&path).expect("golden is readable");
    assert!(
        render::semantically_equal(&expected, actual),
        "{golden_rel} differs from the current output.\n\
         --- golden ---\n{}\n--- actual ---\n{}",
        render::canonical_form(&expected).unwrap_or_default(),
        render::canonical_form(actual).unwrap_or_default()
    );
}

#[test]
fn cli_analyze_json_matches_the_committed_golden() {
    check(
        "golden/cli/p0-dual-region-analyze.json",
        &analyze(
            "fixtures/elf/p0-dual-region/firmware.elf",
            Some("fixtures/elf/p0-dual-region/firmware.map"),
        ),
    );
}

#[test]
fn cli_analyze_without_map_matches_the_committed_golden() {
    check(
        "golden/cli/p0-basic-analyze.json",
        &analyze("fixtures/elf/p0-basic/firmware.elf", None),
    );
}

#[test]
fn memory_golden_records_evidence_and_not_only_numbers() {
    // A golden whose arithmetic is right but whose provenance is wrong is still a failure,
    // which is why locator, classification and rule are part of the asserted content.
    let actual = analyze(
        "fixtures/elf/p0-dual-region/firmware.elf",
        Some("fixtures/elf/p0-dual-region/firmware.map"),
    );
    let document: serde_json::Value = serde_json::from_str(&actual).expect("output is JSON");
    let memory = &document["memory"];

    assert_eq!(memory["accountingRule"], "adr-0021-dual-budget");
    assert_eq!(memory["layoutSource"], "map");
    assert_eq!(
        memory["weakestEvidenceBasis"], "map-memory-configuration+elf-load",
        "the dual-region fixture must be attributed at ladder level 2"
    );
    assert_eq!(memory["admissibleForHardBlock"], true);
    assert_eq!(memory["nonvolatileImageFootprint"]["state"], "exact");
    assert_eq!(memory["runtimeRamFootprint"]["state"], "exact");

    for contribution in memory["contributions"].as_array().expect("contributions") {
        assert!(
            contribution.get("sectionLocator").is_some(),
            "every contribution must be traceable to a locator"
        );
        assert!(
            contribution.get("classification").is_some(),
            "every contribution must declare its evidence class"
        );
        assert!(
            contribution.get("accountingRule").is_some(),
            "every contribution must name the rule that produced it"
        );
    }

    check("golden/cli/p0-dual-region-analyze.json", &actual);
}

#[test]
fn without_region_evidence_the_same_source_is_reported_as_weaker_evidence() {
    // The two fixtures compile the same source. What differs is the available load evidence,
    // and the model must report that difference rather than producing identical claims.
    let with_map = serde_json::from_str::<serde_json::Value>(&analyze(
        "fixtures/elf/p0-dual-region/firmware.elf",
        Some("fixtures/elf/p0-dual-region/firmware.map"),
    ))
    .expect("json");
    let without_map = serde_json::from_str::<serde_json::Value>(&analyze(
        "fixtures/elf/p0-basic/firmware.elf",
        None,
    ))
    .expect("json");

    assert_eq!(with_map["memory"]["admissibleForHardBlock"], true);
    assert_eq!(
        without_map["memory"]["admissibleForHardBlock"], false,
        "no region table must never support a hard verdict"
    );
    assert_eq!(with_map["memory"]["layoutSource"], "map");
    assert_eq!(without_map["memory"]["layoutSource"], "none");
}
