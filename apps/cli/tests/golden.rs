//! Golden comparison for the CLI projection.
//!
//! Goldens are regenerated only by an explicit, reviewed step. A failing comparison never
//! rewrites the file, because "test failed, update golden, green" would defeat the purpose of
//! having a golden (`05_ENGINEERING/02_TEST_STRATEGY.md`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

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
fn human_memory_report_explains_each_section_against_both_budgets() {
    let root = repo_root();
    let request = AnalysisRequest::new(root.join("fixtures/elf/p0-dual-region/firmware.elf"))
        .with_map(root.join("fixtures/elf/p0-dual-region/firmware.map"));
    let analysis = pipeline::analyze(&request).expect("real linked fixture must analyze");
    let dto = AnalyzeResultDto::from_snapshot(&analysis.snapshot, "firmware.elf");
    let report = render::render_human(&dto);

    assert!(
        report.contains("Section contributions (nonvolatile / runtime RAM bytes; evidence basis)")
    );
    assert!(report.contains(
        ".data (elf.section_header[3]): 4 / 4 bytes [observed; map-memory-configuration+elf-load]"
    ));
    assert!(report.contains(
        ".bss (elf.section_header[4]): 0 / 4 bytes [observed; map-memory-configuration+elf-load]"
    ));
    assert!(report.contains(
        ".ota (elf.section_header[5]): 64 / 64 bytes [observed; map-memory-configuration+elf-load]"
    ));
}

#[test]
fn human_memory_report_keeps_unattributed_ram_unknown_without_a_map() {
    let root = repo_root();
    let request = AnalysisRequest::new(root.join("fixtures/elf/p0-basic/firmware.elf"));
    let analysis = pipeline::analyze(&request).expect("real ELF fixture must analyze");
    let dto = AnalyzeResultDto::from_snapshot(&analysis.snapshot, "firmware.elf");
    let report = render::render_human(&dto);

    assert!(report.contains(
        "Runtime RAM:              72 bytes or more, 2 section(s) unattributed [derived]"
    ));
    assert!(report.contains(
        ".persistent (elf.section_header[5]): 0 / unknown bytes [observed; elf-address-and-flags]"
    ));
    assert!(report.contains(
        ".noinit (elf.section_header[7]): 0 / unknown bytes [observed; elf-address-and-flags]"
    ));
}

#[test]
fn human_memory_report_escapes_untrusted_text_without_changing_json_facts() {
    let root = repo_root();
    let request = AnalysisRequest::new(root.join("fixtures/elf/p0-dual-region/firmware.elf"))
        .with_map(root.join("fixtures/elf/p0-dual-region/firmware.map"));
    let analysis = pipeline::analyze(&request).expect("real linked fixture must analyze");
    let mut dto = AnalyzeResultDto::from_snapshot(&analysis.snapshot, "firmware.elf");
    let hostile = "模块\u{1b}[2J\u{1b}[31m\r\n\t\u{7f}\u{85}\u{202e}";
    dto.artifact.file_name = hostile.to_owned();
    dto.memory.contributions[2].section_name = Some(hostile.to_owned());
    dto.memory.contributions[2].section_locator = hostile.to_owned();
    dto.memory.dual_accounted_sections[0] = hostile.to_owned();

    let report = render::render_human(&dto);
    assert!(!report.chars().any(|ch| ch.is_control() && ch != '\n'));
    assert!(report.contains("模块"));
    assert!(report.contains("\\u{1b}[2J\\u{1b}[31m\\r\\n\\t\\u{7f}\\u{85}\\u{202e}"));

    let json: serde_json::Value =
        serde_json::from_str(&render::to_json_pretty(&dto)).expect("DTO JSON remains valid");
    assert_eq!(json["artifact"]["fileName"], hostile);
    assert_eq!(json["memory"]["contributions"][2]["sectionName"], hostile);
    assert_eq!(
        json["memory"]["contributions"][2]["sectionLocator"],
        hostile
    );
    assert_eq!(json["memory"]["dualAccountedSections"][0], hostile);
}

#[test]
fn mutated_elf_section_control_is_escaped_on_cli_and_preserved_in_json() {
    let root = repo_root();
    let mutant = mutated_ota_elf(&root);
    let elf = mutant.0.to_string_lossy().into_owned();
    let map = root.join("fixtures/elf/p0-dual-region/firmware.map");
    let run = |json: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fwsight"));
        command.arg("analyze").arg(&elf).arg("--map").arg(&map);
        if json {
            command.arg("--json");
        }
        command.output().expect("isolated CLI invocation succeeds")
    };

    let human = run(false);
    assert!(human.status.success());
    assert!(!human.stdout.contains(&0x1b));
    assert!(String::from_utf8_lossy(&human.stdout).contains(r"\u{1b}[2J"));

    let machine = run(true);
    assert!(machine.status.success());
    let json: serde_json::Value = serde_json::from_slice(&machine.stdout).expect("valid JSON");
    let contribution = json["memory"]["contributions"]
        .as_array()
        .expect("contributions")
        .iter()
        .find(|row| row["sectionLocator"] == "elf.section_header[5]")
        .expect("mutated OTA contribution");
    assert_eq!(contribution["sectionName"], "\u{1b}[2J");
    assert_eq!(contribution["nonvolatileBytes"], serde_json::Value::Null);
    assert_eq!(contribution["runtimeRamBytes"], 64);
    assert_eq!(
        json["memory"]["nonvolatileImageFootprint"]["state"],
        "partial"
    );
}

struct TemporaryElf(PathBuf);

impl Drop for TemporaryElf {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn mutated_ota_elf(root: &Path) -> TemporaryElf {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
    let source = std::fs::read(root.join("fixtures/elf/p0-dual-region/firmware.elf"))
        .expect("fixture ELF is readable");
    assert_eq!(&source[..4], b"\x7fELF");
    assert_eq!(source[4], 1, "fixture is ELF32");
    assert_eq!(source[5], 1, "fixture is little-endian");

    let section_table = u32::from_le_bytes(source[0x20..0x24].try_into().unwrap()) as usize;
    let section_entry_size = u16::from_le_bytes(source[0x2e..0x30].try_into().unwrap()) as usize;
    let section_count = u16::from_le_bytes(source[0x30..0x32].try_into().unwrap()) as usize;
    let names_index = u16::from_le_bytes(source[0x32..0x34].try_into().unwrap()) as usize;
    assert!(section_entry_size >= 40 && names_index < section_count);
    let names_header = section_table + names_index * section_entry_size;
    let names_offset = u32::from_le_bytes(
        source[names_header + 16..names_header + 20]
            .try_into()
            .unwrap(),
    ) as usize;
    let names_size = u32::from_le_bytes(
        source[names_header + 20..names_header + 24]
            .try_into()
            .unwrap(),
    ) as usize;

    let mut bytes = source;
    let names_end = names_offset + names_size;
    let name_at = bytes[names_offset..names_end]
        .windows(5)
        .position(|window| window == b".ota\0")
        .expect("fixture section-name table contains .ota")
        + names_offset;
    bytes[name_at..name_at + 5].copy_from_slice(b"\x1b[2J\0");

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "firmwaresight-next17-{}-{id}.elf",
        std::process::id()
    ));
    std::fs::write(&path, bytes).expect("write isolated mutated ELF");
    TemporaryElf(path)
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

#[test]
fn memory_accounting_golden_passes_on_both_fixtures() {
    // The focused memory golden is the gate item "memory accounting golden passes". It is a
    // projection of the same payload the CLI golden asserts, kept separate so a reviewer can see
    // the two budgets and their evidence basis without reading a whole artifact dump.
    for (fixture, elf, map) in [
        ("p0-basic", "fixtures/elf/p0-basic/firmware.elf", None),
        (
            "p0-dual-region",
            "fixtures/elf/p0-dual-region/firmware.elf",
            Some("fixtures/elf/p0-dual-region/firmware.map"),
        ),
    ] {
        let actual = analyze(elf, map);
        let document: serde_json::Value = serde_json::from_str(&actual).expect("output is JSON");
        let memory = serde_json::to_string_pretty(&document["memory"]).expect("memory serializes");
        check(&format!("golden/core/{fixture}-memory.json"), &memory);
    }
}
