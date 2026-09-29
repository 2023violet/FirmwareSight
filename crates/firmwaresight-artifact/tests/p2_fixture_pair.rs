//! The P2 Compare fixture pair, proven against what the linker actually produced.
//!
//! `01_PRODUCT/01_PRD_MVP.md:125` makes "Diff 结果可由测试 fixture 精确验证" a success metric, so
//! this file asserts named facts — which section moved, which symbol was added, which budget grew
//! and by how many bytes — rather than row counts. A test that only checked "three sections
//! changed" would still pass if the matcher paired the wrong rows.

use std::path::PathBuf;

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::capability::{Availability, FormatSupport, Provision};
use firmwaresight_core::domain::identity::Fact;
use firmwaresight_core::domain::memory::{ByteTotal, LayoutSource, MemoryEvidenceBasis};
use firmwaresight_core::domain::section::Section;
use firmwaresight_core::domain::symbol::{Symbol, SymbolSectionRef};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("crate sits two levels under the workspace root")
        .to_path_buf()
}

fn fixture(side: &str, name: &str) -> PathBuf {
    let path = repo_root().join(format!("fixtures/elf/p2-diff/{side}/{name}"));
    assert!(
        path.exists(),
        "missing committed P2 fixture: {}",
        path.display()
    );
    path
}

fn analyze(side: &str) -> pipeline::Analysis {
    let request =
        AnalysisRequest::new(fixture(side, "firmware.elf")).with_map(fixture(side, "firmware.map"));
    pipeline::analyze(&request)
        .unwrap_or_else(|err| panic!("the {side} fixture must analyze: {err}"))
}

fn section<'a>(analysis: &'a pipeline::Analysis, name: &str) -> Option<&'a Section> {
    analysis.snapshot.sections().iter().find(|section| {
        section
            .name
            .value()
            .is_some_and(|section_name| section_name == name)
    })
}

fn symbol<'a>(analysis: &'a pipeline::Analysis, name: &str) -> Option<&'a Symbol> {
    analysis
        .snapshot
        .symbols()
        .iter()
        .find(|symbol| symbol.name.value().is_some_and(|n| n == name))
}

fn bytes(total: &ByteTotal) -> Option<u64> {
    total.bytes()
}

fn address_of(section: &Section) -> Option<u64> {
    section.virtual_address.value().copied()
}

fn value_of(fact: &Fact<u64>) -> Option<u64> {
    fact.value().copied()
}

#[test]
fn the_pair_is_two_distinct_builds_not_the_same_snapshot_twice() {
    let base = analyze("base");
    let target = analyze("target");

    assert_ne!(
        base.snapshot.id().as_str(),
        target.snapshot.id().as_str(),
        "Compare needs two snapshots; identical ids would be refused by the diff itself"
    );
    assert_ne!(base.sha256().hex(), target.sha256().hex());
}

#[test]
fn both_sides_of_the_pair_are_recorded_in_the_fixture_manifest() {
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("fixtures/manifest.json")).expect("manifest"),
    )
    .expect("manifest is JSON");
    let entries = manifest["files"].as_array().expect("files is an array");

    for side in ["base", "target"] {
        for name in [
            "firmware.elf",
            "firmware.map",
            "firmware.ld",
            "fixture.toml",
            "source/common.c",
            "source/payload.c",
        ] {
            let relative = format!("fixtures/elf/p2-diff/{side}/{name}");
            assert!(
                entries.iter().any(|entry| {
                    entry["path"].as_str() == Some(&relative)
                        && entry["sha256"]
                            .as_str()
                            .is_some_and(|hash| hash.len() == 64)
                }),
                "{relative} is not recorded with a SHA-256 in fixtures/manifest.json"
            );
        }
    }
}

#[test]
fn the_control_source_file_is_byte_identical_on_both_sides() {
    // The unchanged facts in the pair only mean something if the shared half really is shared.
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("fixtures/manifest.json")).expect("manifest"),
    )
    .expect("manifest is JSON");
    let entries = manifest["files"].as_array().expect("files is an array");

    let recorded = |side: &str| -> String {
        entries
            .iter()
            .find(|entry| {
                entry["path"].as_str()
                    == Some(format!("fixtures/elf/p2-diff/{side}/source/common.c").as_str())
            })
            .and_then(|entry| entry["sha256"].as_str())
            .expect("common.c is recorded for both sides")
            .to_owned()
    };

    assert_eq!(
        recorded("base"),
        recorded("target"),
        "source/common.c must be the same file on both sides, or the unchanged rows prove nothing"
    );
}

#[test]
fn the_text_section_grew_and_the_rodata_section_only_moved() {
    let base = analyze("base");
    let target = analyze("target");

    let base_text = section(&base, ".text").expect("base has .text");
    let target_text = section(&target, ".text").expect("target has .text");
    assert_eq!(base_text.file_size, 124);
    assert_eq!(target_text.file_size, 212);
    assert_eq!(value_of(&base_text.memory_size), Some(124));
    assert_eq!(value_of(&target_text.memory_size), Some(212));
    assert_eq!(
        address_of(base_text),
        address_of(target_text),
        ".text stays at the ROM origin, so its size change is not an address artefact"
    );

    let base_ro = section(&base, ".rodata").expect("base has .rodata");
    let target_ro = section(&target, ".rodata").expect("target has .rodata");
    assert_eq!(base_ro.file_size, 32);
    assert_eq!(target_ro.file_size, 32);
    assert_eq!(address_of(base_ro), Some(0x0800_007c));
    assert_eq!(address_of(target_ro), Some(0x0800_00d4));
    assert_ne!(
        address_of(base_ro),
        address_of(target_ro),
        ".rodata is the address-only case: same size, different place"
    );
}

#[test]
fn one_section_is_unchanged_in_every_field_the_diff_compares() {
    let base = analyze("base");
    let target = analyze("target");

    let from = section(&base, ".settings").expect("base .settings");
    let to = section(&target, ".settings").expect("target .settings");

    assert_eq!(from.file_size, 64);
    assert_eq!(to.file_size, 64);
    assert_eq!(address_of(from), Some(0x0801_0000));
    assert_eq!(address_of(to), Some(0x0801_0000));
    assert_eq!(
        from.load_address.value(),
        to.load_address.value(),
        "the pinned settings page keeps its place in the image too"
    );
    assert_eq!(from.memory_size.value(), to.memory_size.value());
    assert_eq!(from.role, to.role);
    assert_eq!(from.region.value(), to.region.value());
    assert_eq!(
        (from.flags.alloc, from.flags.write, from.flags.execute),
        (to.flags.alloc, to.flags.write, to.flags.execute)
    );
}

#[test]
fn data_stays_identical_and_bss_grows_without_touching_the_image() {
    let base = analyze("base");
    let target = analyze("target");

    let base_data = section(&base, ".data").expect("base .data");
    let target_data = section(&target, ".data").expect("target .data");
    assert_eq!(base_data.file_size, 4);
    assert_eq!(target_data.file_size, 4);
    assert_eq!(address_of(base_data), address_of(target_data));
    assert_eq!(
        base_data.role, target_data.role,
        "the control section keeps its role on both sides"
    );
    assert_eq!(
        base_data.load_address.value(),
        target_data.load_address.value(),
        "the pinned .settings page absorbs the grown .text, so .data keeps both addresses and is \
         a second unchanged control"
    );

    let base_bss = section(&base, ".bss").expect("base .bss");
    let target_bss = section(&target, ".bss").expect("target .bss");
    assert_eq!(base_bss.file_size, 0, ".bss stores no image payload");
    assert_eq!(target_bss.file_size, 0);
    assert_eq!(value_of(&base_bss.memory_size), Some(4));
    assert_eq!(
        value_of(&target_bss.memory_size),
        Some(8),
        "g_frame_counter adds 4 bytes of RAM and no bytes of image"
    );
}

#[test]
fn one_section_exists_only_in_the_base_and_one_only_in_the_target() {
    let base = analyze("base");
    let target = analyze("target");

    let calib = section(&base, ".calib").expect("the base carries the calibration blob");
    assert_eq!(calib.file_size, 32);
    assert!(
        section(&target, ".calib").is_none(),
        "the target linker script declares no .calib rule, so the section is genuinely absent \
         rather than present at zero size"
    );

    let ota = section(&target, ".ota").expect("the target carries the staging area");
    assert_eq!(ota.file_size, 64);
    assert_eq!(value_of(&ota.memory_size), Some(64));
    assert_eq!(
        address_of(ota),
        Some(0x2000_000c),
        ".ota is resident in RAM, which is what makes it charge both budgets"
    );
    assert!(
        section(&base, ".ota").is_none(),
        "the base has no staging area at all"
    );
}

#[test]
fn the_shared_functions_keep_their_size_and_two_of_them_keep_their_address() {
    let base = analyze("base");
    let target = analyze("target");

    for name in ["boot_check", "main", "main_entry"] {
        let from = symbol(&base, name).unwrap_or_else(|| panic!("base must define {name}"));
        let to = symbol(&target, name).unwrap_or_else(|| panic!("target must define {name}"));
        assert_eq!(
            (value_of(&from.size), value_of(&from.address)),
            (value_of(&to.size), value_of(&to.address)),
            "{name} lives in the untouched common.o, so it is the unchanged-symbol control"
        );
    }

    let mqtt_base = symbol(&base, "mqtt_task").expect("base mqtt_task");
    let mqtt_target = symbol(&target, "mqtt_task").expect("target mqtt_task");
    assert_eq!(value_of(&mqtt_base.size), Some(4));
    assert_eq!(
        value_of(&mqtt_target.size),
        Some(46),
        "mqtt_task is the changed-size symbol"
    );

    let handshake_base = symbol(&base, "tls_handshake").expect("base tls_handshake");
    let handshake_target = symbol(&target, "tls_handshake").expect("target tls_handshake");
    assert_eq!(value_of(&handshake_base.size), Some(16));
    assert_eq!(value_of(&handshake_target.size), Some(16));
    assert_ne!(
        handshake_base.address.value(),
        handshake_target.address.value(),
        "its body did not change; only its address did, which is the movement Compare must show"
    );

    let apply_base = symbol(&base, "calib_apply").expect("base calib_apply");
    let apply_target = symbol(&target, "calib_apply").expect("target calib_apply");
    assert_eq!(value_of(&apply_base.size), Some(16));
    assert_eq!(
        value_of(&apply_target.size),
        Some(2),
        "the entry point stayed and its data went, so this is a shrink"
    );
}

#[test]
fn three_symbols_exist_only_in_the_target_and_one_only_in_the_base() {
    let base = analyze("base");
    let target = analyze("target");

    for name in ["packet_router", "staging_area", "g_frame_counter"] {
        assert!(
            symbol(&base, name).is_none(),
            "{name} must not exist in the base build"
        );
        let added = symbol(&target, name)
            .unwrap_or_else(|| panic!("{name} must exist in the target build"));
        assert!(
            added.size.is_known(),
            "{name} is expected to carry a real size, not an unknown one"
        );
    }

    let calib = symbol(&base, "calib").expect("base calib");
    assert_eq!(value_of(&calib.size), Some(32));
    assert!(
        symbol(&target, "calib").is_none(),
        "calib is the removed-symbol case"
    );
}

#[test]
fn both_budgets_move_by_a_checkable_amount_on_exact_map_backed_evidence() {
    let base = analyze("base");
    let target = analyze("target");

    let base_memory = base.snapshot.memory().expect("base footprint");
    let target_memory = target.snapshot.memory().expect("target footprint");

    assert_eq!(bytes(&base_memory.nonvolatile), Some(256));
    assert_eq!(bytes(&target_memory.nonvolatile), Some(376));
    assert_eq!(bytes(&base_memory.runtime_ram), Some(8));
    assert_eq!(bytes(&target_memory.runtime_ram), Some(76));

    // Both sides are charged from a real linker region table, so the diff earns an exact delta.
    for (side, memory) in [("base", base_memory), ("target", target_memory)] {
        assert!(
            memory.nonvolatile.is_exact() && memory.runtime_ram.is_exact(),
            "the {side} side is expected to be exactly accounted, because a MAP is stored"
        );
        assert_eq!(memory.layout_source, LayoutSource::MapMemoryConfiguration);
        assert_eq!(
            memory.weakest_basis,
            Some(MemoryEvidenceBasis::MapRegionAndElfLoad),
            "the weakest basis must stay at the MAP rung, not drop to a name heuristic"
        );
        assert!(
            memory.admissible_for_hard_block(),
            "the {side} footprint is strong enough for a hard verdict later"
        );
    }

    assert_eq!(
        base_memory.dual_accounted_sections().len(),
        1,
        ".data is dual-accounted in the base; .ota adds a second one in the target"
    );
    assert_eq!(target_memory.dual_accounted_sections().len(), 2);
}

#[test]
fn each_side_stores_both_artifacts_and_reports_the_map_as_provided() {
    let base = analyze("base");
    let target = analyze("target");

    for (side, analysis) in [("base", &base), ("target", &target)] {
        assert_eq!(
            analysis.snapshot.artifacts().len(),
            2,
            "the {side} build is ELF plus MAP, so Compare has a per-side evidence quality"
        );
        assert_eq!(analysis.capabilities.map, Provision::Provided);
        assert_eq!(analysis.capabilities.elf, FormatSupport::Supported);
        assert_eq!(analysis.capabilities.symbols, Availability::Available);
    }
}

#[test]
fn nothing_in_the_pair_claims_an_object_per_symbol_that_the_parser_did_not_read() {
    // The pair is deliberately built from two translation units so the claim can be checked rather
    // than assumed: attribution may be reported as a MAP capability, but no symbol row is required
    // to name the object file it came from, because storage has nowhere to keep it.
    let target = analyze("target");
    let packet_router = symbol(&target, "packet_router").expect("target packet_router");

    assert!(
        packet_router.name.is_known()
            && matches!(packet_router.section, SymbolSectionRef::Index(_)),
        "the added symbol is attributable to a section, which is the strongest persisted link there is"
    );
    assert!(
        !packet_router
            .name
            .value()
            .is_some_and(|name| name.contains("payload.o")),
        "a symbol name is never an object attribution"
    );
}
