//! P0 acceptance tests against real linker output.
//!
//! These are the tests the P0 prompt says memory accounting must not be declared proven
//! without: the fixture is a genuine ELF that GNU ld produced, and the dual-accounting claim is
//! read back out of explicit load evidence rather than inferred from a section name.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::ArtifactError;
use firmwaresight_artifact::intake::{DetectedFormat, GuardConfig, GuardedInput};
use firmwaresight_artifact::map;
use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::capability::{Availability, Provision};
use firmwaresight_core::domain::identity::Fact;
use firmwaresight_core::domain::memory::{ByteTotal, MemoryEvidenceBasis};
use firmwaresight_core::domain::section::SectionRole;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives at <root>/crates/<name>")
        .to_path_buf()
}

fn fixture(relative: &str) -> PathBuf {
    let path = repo_root().join(relative);
    assert!(
        path.exists(),
        "missing committed fixture: {}",
        path.display()
    );
    path
}

fn dual_region_elf() -> PathBuf {
    fixture("fixtures/elf/p0-dual-region/firmware.elf")
}

fn dual_region_map() -> PathBuf {
    fixture("fixtures/elf/p0-dual-region/firmware.map")
}

fn basic_elf() -> PathBuf {
    fixture("fixtures/elf/p0-basic/firmware.elf")
}

#[test]
fn committed_fixtures_match_their_recorded_hashes() {
    // A golden that passes against a silently-different binary proves nothing, so the manifest
    // is checked before any other expectation in this file relies on fixture content.
    let manifest_path = fixture("fixtures/manifest.json");
    let raw = std::fs::read_to_string(&manifest_path).expect("manifest is UTF-8");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("manifest is JSON");
    let entries = value["files"].as_array().expect("files is an array");
    assert!(
        !entries.is_empty(),
        "the fixture manifest recorded no files"
    );

    for entry in entries {
        let relative = entry["path"].as_str().expect("path is a string");
        let expected = entry["sha256"].as_str().expect("sha256 is a string");
        let path = repo_root().join(relative);
        let input = GuardedInput::load(&path, GuardConfig::default())
            .unwrap_or_else(|err| panic!("cannot load {relative}: {err}"));
        assert_eq!(
            input.sha256.hex(),
            expected,
            "fixture {relative} differs from the hash recorded in manifest.json"
        );
    }
}

#[test]
fn real_elf_fixture_is_detected_and_hashed() {
    let analysis = pipeline::analyze(&AnalysisRequest::new(dual_region_elf()))
        .expect("the committed ELF must analyze");

    assert_eq!(analysis.input.format, DetectedFormat::Elf32Le);
    assert_eq!(analysis.input.sha256.hex().len(), 64);
    assert!(
        analysis
            .input
            .sha256
            .hex()
            .chars()
            .all(|c| c.is_ascii_hexdigit()),
        "identity must be lowercase hex"
    );
    assert_eq!(analysis.snapshot.artifacts().len(), 1);
}

#[test]
fn map_adapter_produces_real_memory_region_facts() {
    // PRE-0.6: not just detection and a capability claim, but a verified fact out of a real MAP.
    let input = GuardedInput::load(&dual_region_map(), GuardConfig::default()).expect("MAP loads");
    let text = std::str::from_utf8(&input.bytes).expect("MAP is UTF-8");

    assert!(map::detect(text).expect("detection must not error on GNU ld output"));

    let evidence = map::parse(text).expect("the committed GNU ld MAP must parse");
    assert_eq!(evidence.adapter_id, "gnu_ld");

    let rom = evidence
        .regions
        .iter()
        .find(|r| r.name == "ROM")
        .expect("ROM region recorded by the linker");
    let ram = evidence
        .regions
        .iter()
        .find(|r| r.name == "RAM")
        .expect("RAM region recorded by the linker");

    assert_eq!(rom.origin, 0x0800_0000);
    assert_eq!(rom.length, 0x0010_0000);
    assert_eq!(ram.origin, 0x2000_0000);
    assert_eq!(ram.length, 0x0004_0000);

    // The emulation string is a second independent real fact from the same file.
    let emulation = evidence.emulation.as_deref().unwrap_or_default();
    assert!(
        emulation.contains("elf32") && emulation.contains("arm"),
        "expected an ARM ELF32 emulation record, got {emulation:?}"
    );

    // Symbol contributions are real too.
    for name in [
        "mqtt_task",
        "tls_handshake",
        "sensor_fifo",
        "g_threshold",
        "g_scratch",
    ] {
        assert!(
            evidence.symbols.iter().any(|s| s.name == name),
            "the MAP should record a contribution for {name}"
        );
    }
    assert!(
        evidence
            .object_contributions
            .iter()
            .any(|c| c.object.ends_with("main.o")),
        "the MAP should record per-object contributions"
    );
}

#[test]
fn data_is_dual_accounted_from_explicit_load_address_evidence() {
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(dual_region_map()))
            .expect("ELF + GNU ld MAP must analyze together");

    let data = analysis
        .memory
        .contributions
        .iter()
        .find(|c| c.section_name.value().map(String::as_str) == Some(".data"))
        .expect("the fixture declares initialized data");

    assert_eq!(data.nonvolatile_bytes.value(), Some(&4));
    assert_eq!(data.runtime_ram_bytes.value(), Some(&4));
    assert!(
        data.is_dual_accounted(),
        ".data must be charged to both budgets"
    );
    assert_eq!(
        data.basis,
        MemoryEvidenceBasis::MapRegionAndElfLoad,
        "the charge must rest on MAP region + ELF load evidence, not on the section name"
    );
    assert_eq!(data.rule, "map-memory-configuration+elf-load");
}

#[test]
fn custom_region_section_is_also_dual_accounted() {
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(dual_region_map()))
            .expect("analysis");

    let ota = analysis
        .memory
        .contributions
        .iter()
        .find(|c| c.section_name.value().map(String::as_str) == Some(".ota"))
        .expect("the fixture declares an .ota output section");

    assert_eq!(ota.nonvolatile_bytes.value(), Some(&64));
    assert_eq!(ota.runtime_ram_bytes.value(), Some(&64));
    assert!(ota.is_dual_accounted());
    // The role is not recognizable as a standard class, so only evidence could produce numbers.
    assert_eq!(ota.basis, MemoryEvidenceBasis::MapRegionAndElfLoad);
}

#[test]
fn bss_costs_ram_and_nothing_in_the_image() {
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(dual_region_map()))
            .expect("analysis");

    let bss = analysis
        .memory
        .contributions
        .iter()
        .find(|c| c.section_name.value().map(String::as_str) == Some(".bss"))
        .expect("the fixture declares zero-initialized data");

    assert_eq!(bss.nonvolatile_bytes.value(), Some(&0));
    assert_eq!(bss.runtime_ram_bytes.value(), Some(&4));
    assert!(!bss.is_dual_accounted());
}

#[test]
fn debug_sections_are_excluded_from_both_budgets() {
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(dual_region_map()))
            .expect("analysis");

    assert!(
        analysis
            .snapshot
            .sections()
            .iter()
            .any(|s| s.role == SectionRole::Debug),
        "the fixture is built with -g, so debug sections must be present to exclude"
    );
    assert!(
        analysis
            .memory
            .contributions
            .iter()
            .all(|c| c.section_name.value().map(String::as_str) != Some(".debug_info")),
        "debug sections must never be charged to a device budget"
    );
    assert!(analysis.memory.excluded_metadata_bytes > 0);
}

#[test]
fn totals_are_exact_and_admissible_for_a_hard_verdict() {
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(dual_region_map()))
            .expect("analysis");

    assert!(
        matches!(analysis.memory.nonvolatile, ByteTotal::Exact { .. }),
        "every allocatable section should be attributable: {:?}",
        analysis.memory.nonvolatile
    );
    assert!(
        matches!(analysis.memory.runtime_ram, ByteTotal::Exact { .. }),
        "every allocatable section should be attributable: {:?}",
        analysis.memory.runtime_ram
    );
    assert!(analysis.memory.admissible_for_hard_block());
    assert_eq!(analysis.memory.dual_accounted_sections().len(), 2);
}

#[test]
fn the_two_budgets_are_different_numbers() {
    // The whole point of ADR-0021: if the model collapsed the budgets these would be equal.
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(dual_region_map()))
            .expect("analysis");

    let nonvolatile = analysis.memory.nonvolatile.bytes().expect("exact");
    let runtime_ram = analysis.memory.runtime_ram.bytes().expect("exact");
    assert_ne!(
        nonvolatile, runtime_ram,
        "load-image footprint and runtime RAM footprint must not collapse into one number"
    );
    assert!(nonvolatile > runtime_ram);
}

#[test]
fn without_a_map_the_model_reports_weaker_evidence_not_false_confidence() {
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf())).expect("ELF alone analyzes");

    assert_eq!(analysis.capabilities.map, Provision::NotProvided);
    assert_eq!(
        analysis.capabilities.object_attribution,
        Availability::Unavailable
    );
    // No region table means the strongest available basis is below the hard-verdict threshold.
    assert!(!analysis.memory.admissible_for_hard_block());
    assert!(
        !matches!(
            analysis.memory.weakest_basis,
            Some(MemoryEvidenceBasis::MapRegionAndElfLoad)
        ),
        "a MAP-less analysis must not claim MAP region evidence"
    );
}

#[test]
fn evidence_records_provenance_and_not_only_numbers() {
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(dual_region_map()))
            .expect("analysis");

    let dual = analysis
        .snapshot
        .evidence()
        .iter()
        .find(|e| e.field == "dual_accounted")
        .expect("dual accounting must carry its own evidence record");

    assert_eq!(
        dual.classification,
        firmwaresight_core::domain::evidence::EvidenceClass::Observed
    );
    assert!(
        dual.source_locator.contains("elf.section_header") && dual.source_locator.contains("map:"),
        "locator must name both the ELF section and the MAP record: {}",
        dual.source_locator
    );

    let regions = analysis
        .snapshot
        .evidence()
        .iter()
        .find(|e| e.field == "memory_regions")
        .expect("region evidence must be recorded");
    assert!(regions.raw_value.contains("ROM@0x8000000"));
    assert!(regions.raw_value.contains("RAM@0x20000000"));
}

#[test]
fn malformed_inputs_return_typed_errors_and_never_panic() {
    let root = repo_root();
    let cases = [
        "fixtures/malformed/empty.bin",
        "fixtures/malformed/wrong-magic.bin",
        "fixtures/malformed/truncated-elf.bin",
        "fixtures/malformed/sparse-elf-header.bin",
    ];

    for case in cases {
        let path = root.join(case);
        let result =
            std::panic::catch_unwind(|| pipeline::analyze(&AnalysisRequest::new(path.clone())));

        let outcome = result.unwrap_or_else(|_| panic!("{case} must not panic"));
        assert!(
            outcome.is_err(),
            "{case} was accepted, which means the guard or the parser is not rejecting it"
        );
        let err = outcome.unwrap_err();
        assert!(
            err.is_parse_or_import(),
            "{case} produced a non-input error: {err:?}"
        );
    }
}

#[test]
fn truncated_elf_is_reported_as_invalid_container_not_as_a_missing_file() {
    let result = pipeline::analyze(&AnalysisRequest::new(fixture(
        "fixtures/malformed/truncated-elf.bin",
    )));
    let err = result.expect_err("a 64-byte truncated ELF must not parse");
    assert!(
        matches!(err, ArtifactError::InvalidElf { .. }),
        "expected a typed container error, got {err:?}"
    );
}

#[test]
fn missing_file_is_input_not_found_and_carries_a_stable_code() {
    let result = pipeline::analyze(&AnalysisRequest::new(repo_root().join("fixtures/nope.elf")));
    let err = result.expect_err("a nonexistent path must not analyze");
    assert!(matches!(err, ArtifactError::InputNotFound { .. }));
    assert_eq!(err.stable_code(), "ERR-INPUT-0001");
    assert!(!err.remediation().is_empty());
}

#[test]
fn oversized_artifact_is_rejected_before_allocation() {
    // A 1 MiB limit against the real fixture proves the guard path without generating a
    // 512 MiB file. The full-ceiling behaviour is covered separately by the benchmark script.
    let path = dual_region_elf();
    let size = std::fs::metadata(&path).expect("fixture exists").len();

    let result = pipeline::analyze(&AnalysisRequest::new(path).with_guard(GuardConfig {
        max_full_buffer_bytes: size.saturating_sub(1),
    }));

    match result {
        Err(ArtifactError::ArtifactTooLarge {
            byte_size,
            limit_bytes,
            ..
        }) => {
            assert_eq!(byte_size, size);
            assert_eq!(limit_bytes, size - 1);
        }
        other => panic!("expected a size-guard rejection, got {other:?}"),
    }
}

#[test]
fn foreign_toolchain_maps_are_rejected_without_falling_back() {
    // ArmClang and IAR must never be handed to the GNU parser.
    let armclang = "ARM Linker map information\nImage Header\n  Load Region LR_1\n".to_owned();
    let err = map::detect(&armclang).expect_err("ArmClang output must be refused");
    assert!(matches!(err, ArtifactError::MapUnsupported { .. }));
    assert_eq!(err.stable_code(), "ERR-MAP-3001");

    let iar = "** IAR Linker\nPLACEBLOCKS\n".to_owned();
    assert!(map::detect(&iar).is_err(), "IAR output must be refused");

    // Unrecognizable text is a clean negative, not a partial success.
    let unknown = "just some notes about a build\n".to_owned();
    assert!(
        matches!(map::detect(&unknown), Ok(false)),
        "unrelated text must be a clean negative"
    );
    assert!(matches!(
        map::parse(&unknown),
        Err(ArtifactError::MapUnsupported { .. })
    ));
}

#[test]
fn gnu_ld_map_is_detected_when_its_banner_sits_behind_a_long_preamble() {
    // E2E-F003. GNU ld prints an "Archive member included to satisfy reference by file (symbol)"
    // block, and a "Discarded input sections" block whenever --gc-sections is active, long before
    // it reaches "Memory Configuration". Both grow with the number of objects and libraries, so an
    // ordinary vendor-HAL build puts the banner far past any fixed head window: the owner's own
    // project put it at character 1,162,310 and a 51 MB synthetic at 491,879, while the same linker
    // with the banner at 745 was accepted.
    let mut preamble =
        String::from("Archive member included to satisfy reference by file (symbol)\n\n");
    while preamble.len() < 500_000 {
        preamble.push_str("libat32f4.a(gpio.c.o)           0x08000100        gpio_init\n");
    }
    assert!(
        preamble.len() > 4096,
        "the fixture has to cross the window this defect is about, got {} bytes",
        preamble.len()
    );
    let map = format!(
        "{preamble}Memory Configuration\n\nName             Origin             Length             Attributes\n\
         FLASH            0x08000000         0x00020000         xr\n\
         RAM              0x20000000         0x00008000         xrw\n\
         *default*\n\nLinker script and memory map\n"
    );

    assert!(
        map::detect(&map).expect("a GNU ld map must not be refused"),
        "a GNU ld banner behind the preamble must still identify the file as GNU ld"
    );

    // The refusal this test replaces stated a cause that was false, so pin the shape of the
    // negative too: a file with no banner at all is still a clean negative, never a guess.
    assert!(
        matches!(map::detect("notes about a build\n"), Ok(false)),
        "unrelated text must stay a clean negative"
    );
}

#[test]
fn basic_fixture_reports_symbols_available_and_map_not_provided() {
    let analysis = pipeline::analyze(&AnalysisRequest::new(basic_elf())).expect("analyzes");

    assert_eq!(analysis.capabilities.symbols, Availability::Available);
    assert_eq!(analysis.capabilities.map, Provision::NotProvided);
    assert_eq!(analysis.capabilities.debug_info, Availability::Available);
    assert!(
        analysis
            .snapshot
            .symbols()
            .iter()
            .any(|s| s.name.value().map(String::as_str) == Some("mqtt_task")),
        "the parser must surface real symbol names from the fixture"
    );
}

#[test]
fn the_same_bytes_produce_the_same_snapshot_identically_twice() {
    let first = pipeline::analyze(&AnalysisRequest::new(dual_region_elf())).expect("first");
    let second = pipeline::analyze(&AnalysisRequest::new(dual_region_elf())).expect("second");

    assert_eq!(first.input.sha256, second.input.sha256);
    assert_eq!(first.snapshot.id(), second.snapshot.id());
    assert_eq!(first.memory, second.memory);
}

#[test]
fn unknown_load_address_stays_unknown_rather_than_defaulting_to_zero() {
    let input = GuardedInput::load(&basic_elf(), GuardConfig::default()).expect("loads");
    let facts = firmwaresight_artifact::elf::parse(&input.bytes, &input.sha256).expect("parses");

    assert!(
        facts
            .sections
            .iter()
            .all(|s| matches!(s.load_address, Fact::Unknown { .. })),
        "without a MAP there is no load address evidence, so none may be claimed"
    );
}
