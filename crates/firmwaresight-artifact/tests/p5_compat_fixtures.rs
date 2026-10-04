//! P5 Commit E compatibility acceptance tests.
//!
//! Every expectation here was read off the fixture bytes with `readelf` and off the product's own
//! answer, then written down as a number. That order matters: a fixture whose expected values were
//! copied from the analyzer would pass whatever the analyzer did, which is the opposite of the
//! evidence these files exist to provide. `P5_VALIDATION/P5_COMPATIBILITY_FIXTURE_REPORT.md` carries
//! the hand derivation beside each figure.
//!
//! The fixtures are committed binaries, so nothing in this file needs a compiler, a linker or
//! `readelf` at run time. Only regenerating them does (`scripts/gen_p5_compat_fixtures.py`).

use std::path::{Path, PathBuf};

use firmwaresight_artifact::intake::{DetectedFormat, GuardConfig, GuardedInput};
use firmwaresight_artifact::map;
use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::identity::{Architecture, ArtifactKind, Bitness, Endianness};
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

fn compat(set: &str, file: &str) -> PathBuf {
    fixture(&format!("fixtures/elf/p5-compat/{set}/{file}"))
}

fn analyze(set: &str, elf: &str) -> pipeline::Analysis {
    pipeline::analyze(&AnalysisRequest::new(compat(set, elf)).with_map(compat(set, "firmware.map")))
        .unwrap_or_else(|err| panic!("ELF + GNU ld MAP for {set} must analyze together: {err}"))
}

fn charge(total: &ByteTotal) -> u64 {
    match total {
        ByteTotal::Exact { bytes } => *bytes,
        ByteTotal::Partial { bytes, .. } => *bytes,
        ByteTotal::Unknown { reason } => panic!("the total was left Unknown: {reason}"),
    }
}

fn contribution<'a>(
    a: &'a pipeline::Analysis,
    name: &str,
) -> &'a firmwaresight_core::domain::memory::MemoryContribution {
    a.memory
        .contributions
        .iter()
        .find(|c| c.section_name.value().map(String::as_str) == Some(name))
        .unwrap_or_else(|| panic!("no memory contribution recorded for {name}"))
}

/// The program-header count, read straight from the ELF header with no toolchain and no parser
/// dependency. This exists because the claim under test is how many load segments a real linker
/// emitted, and `firmwaresight-core` deliberately does not surface a segment list to tests.
fn load_segments(path: &Path) -> usize {
    let bytes = std::fs::read(path).expect("fixture is readable");
    assert_eq!(
        &bytes[0..4],
        b"\x7fELF",
        "{} is not an ELF file",
        path.display()
    );
    assert_eq!(bytes[4], 1, "this helper reads ELFCLASS32 only");
    let phoff = u32::from_le_bytes(bytes[0x1C..0x20].try_into().unwrap()) as usize;
    let phentsize = u16::from_le_bytes(bytes[0x2A..0x2C].try_into().unwrap()) as usize;
    let phnum = u16::from_le_bytes(bytes[0x2C..0x2E].try_into().unwrap()) as usize;
    (0..phnum)
        .filter(|i| {
            let at = phoff + i * phentsize;
            u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) == 1 // PT_LOAD
        })
        .count()
}

/// `(e_entry, EI_CLASS, EI_DATA, e_machine)` from the file's own header, for the assertions about what
/// the artifact *is*. Deriving those from the model would make them unfalsifiable.
fn elf32_identity(path: &Path) -> (u64, u8, u8, u16) {
    let bytes = std::fs::read(path).expect("fixture is readable");
    assert_eq!(
        &bytes[0..4],
        b"\x7fELF",
        "{} is not an ELF file",
        path.display()
    );
    let entry = u32::from_le_bytes(bytes[0x18..0x1C].try_into().unwrap()) as u64;
    let machine = u16::from_le_bytes(bytes[0x12..0x14].try_into().unwrap());
    (entry, bytes[4], bytes[5], machine)
}

/// Every committed ELF the manifest records under `fixtures/elf/`. Cohort-wide invariants iterate this
/// instead of a hand-picked list, so a fixture added without its own named case is still checked.
/// The `fixtures/generated/` workloads are deliberately not in the manifest and are not analyzed here:
/// they exist to exercise the size guard, not the parser.
fn committed_elfs() -> Vec<String> {
    let raw =
        std::fs::read_to_string(fixture("fixtures/manifest.json")).expect("manifest is UTF-8");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("manifest is JSON");
    value["files"]
        .as_array()
        .expect("files is an array")
        .iter()
        .filter_map(|e| e["path"].as_str().map(str::to_owned))
        .filter(|path| path.starts_with("fixtures/elf/") && path.ends_with(".elf"))
        .collect()
}

fn analyze_relative(relative: &str) -> pipeline::Analysis {
    let path = repo_root().join(relative);
    let map = path
        .parent()
        .expect("a fixture file has a directory")
        .join("firmware.map");
    let request = if map.exists() {
        AnalysisRequest::new(path).with_map(map)
    } else {
        AnalysisRequest::new(path)
    };
    pipeline::analyze(&request).unwrap_or_else(|err| panic!("{relative}: {err}"))
}

/// Bytes the file itself says are loaded: `SHF_ALLOC` set and the section not `SHT_NOBITS`, read
/// from the section headers with no dependency on how the model classified them. This independence
/// is the point — an assertion built from the model's own `flags.alloc` stays green when the bug is
/// in how that flag is derived, which is exactly what the first version of the test below did.
fn raw_allocated_payload(path: &Path) -> u64 {
    let bytes = std::fs::read(path).expect("fixture is readable");
    assert_eq!(
        &bytes[0..4],
        b"\x7fELF",
        "{} is not an ELF file",
        path.display()
    );
    assert_eq!(bytes[4], 1, "this helper reads ELFCLASS32 only");
    // ELF32 header: e_shoff at 0x20, e_shentsize at 0x2E, e_shnum at 0x30.
    let shoff = u32::from_le_bytes(bytes[0x20..0x24].try_into().unwrap()) as usize;
    let shentsize = u16::from_le_bytes(bytes[0x2E..0x30].try_into().unwrap()) as usize;
    let shnum = u16::from_le_bytes(bytes[0x30..0x32].try_into().unwrap()) as usize;
    (0..shnum)
        .map(|i| {
            let at = shoff + i * shentsize;
            let sh_type = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
            let sh_flags = u32::from_le_bytes(bytes[at + 8..at + 12].try_into().unwrap());
            let sh_size = u32::from_le_bytes(bytes[at + 20..at + 24].try_into().unwrap()) as u64;
            // SHF_ALLOC = 0x2, SHT_NOBITS = 8.
            if sh_flags & 0x2 != 0 && sh_type != 8 {
                sh_size
            } else {
                0
            }
        })
        .sum()
}

fn text_of(path: &Path) -> String {
    String::from_utf8_lossy(&std::fs::read(path).expect("fixture is readable")).into_owned()
}

// ---- evidence integrity ------------------------------------------------------------------------

#[test]
fn every_compatibility_fixture_matches_the_hash_recorded_in_the_manifest() {
    let raw =
        std::fs::read_to_string(fixture("fixtures/manifest.json")).expect("manifest is UTF-8");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("manifest is JSON");
    let entries = value["files"].as_array().expect("files is an array");
    let mine: Vec<_> = entries
        .iter()
        .filter(|e| {
            e["path"]
                .as_str()
                .unwrap_or_default()
                .starts_with("fixtures/elf/p5-compat/")
        })
        .collect();

    assert_eq!(
        mine.len(),
        31,
        "the compatibility set is 31 recorded files across six sets; a different count means a \
         fixture or its record went missing, not that this assertion is stale"
    );
    for entry in mine {
        let relative = entry["path"].as_str().expect("path is a string");
        let expected = entry["sha256"].as_str().expect("sha256 is a string");
        let input = GuardedInput::load(&repo_root().join(relative), GuardConfig::default())
            .unwrap_or_else(|err| panic!("cannot load {relative}: {err}"));
        assert_eq!(
            input.sha256.hex(),
            expected,
            "fixture {relative} differs from the hash recorded in manifest.json"
        );
    }
}

#[test]
fn the_clang_fixture_names_clang_in_its_own_bytes_and_the_gcc_fixtures_name_gcc() {
    // Compiler identity is only evidence if the artifact carries it. A GCC ELF relabelled as Clang
    // — the specific shortcut the round's provenance rule forbids — fails here on its own bytes.
    // The claim is cohort-wide over every ELF the manifest records, so the pre-existing fixtures are
    // held to the same rule as the new ones rather than being assumed honest.
    let mut clang_built = Vec::new();
    let mut gcc_built = Vec::new();
    for relative in committed_elfs() {
        let text = text_of(&repo_root().join(&relative));
        let names_clang = text.contains("clang version");
        let names_gcc = text.contains("GCC: (Arm GNU Toolchain");
        assert!(
            names_clang != names_gcc,
            "{relative} must name exactly one compiler in its own bytes: clang={names_clang}              GCC={names_gcc}"
        );
        if names_clang {
            clang_built.push(relative);
        } else {
            gcc_built.push(relative);
        }
    }

    assert_eq!(
        clang_built,
        vec!["fixtures/elf/p5-compat/clang-arm/firmware.elf".to_owned()],
        "the cohort carries exactly one Clang-produced ELF, and it is the one built for that purpose"
    );
    assert_eq!(
        gcc_built.len(),
        10,
        "ten ELF files must carry the Arm GCC banner, got {}",
        gcc_built.len()
    );
}

// ---- case A: the classic FLASH + RAM layout, already proven --------------------------------------

#[test]
fn the_classic_flash_and_ram_case_needs_no_new_fixture() {
    // §14's six bullets, asserted in one place against the fixture that already satisfies them.
    // The naming limit is part of the claim: this script calls the read-only region `ROM`, not
    // `FLASH`, and the product follows addresses and attributes rather than either word.
    let elf = fixture("fixtures/elf/p0-dual-region/firmware.elf");
    let map_path = fixture("fixtures/elf/p0-dual-region/firmware.map");
    let analysis =
        pipeline::analyze(&AnalysisRequest::new(elf).with_map(map_path)).expect("analyzes");

    let rom = analysis
        .map_evidence
        .as_ref()
        .and_then(|m| m.regions.iter().find(|r| r.name == "ROM"))
        .expect("the fixture's read-only region is declared and named ROM, not FLASH");
    assert_eq!(rom.origin, 0x0800_0000);

    let data = contribution(&analysis, ".data");
    assert_eq!(
        data.nonvolatile_bytes.value(),
        Some(&4),
        ".data costs image bytes"
    );
    assert_eq!(
        data.runtime_ram_bytes.value(),
        Some(&4),
        ".data costs live RAM"
    );
    assert!(data.is_dual_accounted());

    let bss = contribution(&analysis, ".bss");
    assert_eq!(bss.runtime_ram_bytes.value(), Some(&4), ".bss is live RAM");
    assert_eq!(
        bss.nonvolatile_bytes.value(),
        Some(&0),
        ".bss stores no image bytes"
    );

    assert_eq!(charge(&analysis.memory.nonvolatile), 160);
    assert_eq!(charge(&analysis.memory.runtime_ram), 72);
    assert_eq!(
        analysis.memory.contributions.len(),
        5,
        "ROM, RAM, .data, .ota and .bss are charged"
    );
}

// ---- case B: executable code resident in RAM ---------------------------------------------------

#[test]
fn code_resident_in_ram_is_charged_to_ram_by_load_evidence_not_by_its_name() {
    let analysis = analyze("ram-exec", "firmware.elf");
    let fast = contribution(&analysis, ".fast_text");

    assert_eq!(
        fast.nonvolatile_bytes.value(),
        Some(&12),
        "the code is in the image"
    );
    assert_eq!(
        fast.runtime_ram_bytes.value(),
        Some(&12),
        "and it lives in RAM at run time"
    );
    assert!(fast.is_dual_accounted(), "one section, both budgets");
    assert_eq!(fast.basis, MemoryEvidenceBasis::MapRegionAndElfLoad);

    let section = analysis
        .snapshot
        .sections()
        .iter()
        .find(|s| s.name.value().map(String::as_str) == Some(".fast_text"))
        .expect("the RAM-resident code section is recorded");
    assert_eq!(
        section.virtual_address.value(),
        Some(&0x2000_0004),
        "runs from RAM"
    );
    assert_eq!(
        section.load_address.value(),
        Some(&0x0800_003c),
        "loaded from FLASH"
    );
    assert!(section.flags.execute, "it is executable, not data");

    assert_eq!(
        charge(&analysis.memory.nonvolatile),
        72,
        "24 text + 32 rodata + 4 data + 12 fast_text"
    );
    assert_eq!(
        charge(&analysis.memory.runtime_ram),
        20,
        "4 data + 12 fast_text + 4 bss"
    );
    assert!(matches!(
        analysis.memory.nonvolatile,
        ByteTotal::Exact { .. }
    ));
}

// ---- case C: external SRAM ---------------------------------------------------------------------

#[test]
fn an_external_sram_region_is_observed_from_the_map_and_charged_to_runtime_ram() {
    let analysis = analyze("extsram", "firmware.elf");

    let extsram = analysis
        .map_evidence
        .as_ref()
        .and_then(|m| m.regions.iter().find(|r| r.name == "EXTSRAM"))
        .expect("the external region is declared by the linker's own memory map");
    assert_eq!(extsram.origin, 0x6000_0000);
    assert_eq!(extsram.length, 0x0008_0000);

    let pool = contribution(&analysis, ".extsram_pool");
    assert_eq!(
        pool.nonvolatile_bytes.value(),
        Some(&128),
        "the pool ships in the image"
    );
    assert_eq!(
        pool.runtime_ram_bytes.value(),
        Some(&128),
        "and it is live memory out there"
    );
    assert_eq!(
        pool.basis,
        MemoryEvidenceBasis::MapRegionAndElfLoad,
        "from region evidence, \
        because no internal-RAM assumption reaches 0x6000_0000"
    );

    assert_eq!(
        charge(&analysis.memory.nonvolatile),
        184,
        "20 text + 32 rodata + 4 data + 128 pool"
    );
    assert_eq!(
        charge(&analysis.memory.runtime_ram),
        136,
        "4 data + 128 pool + 4 bss"
    );
}

// ---- case D: a custom-named writable region ----------------------------------------------------

#[test]
fn a_custom_named_writable_region_is_not_lost_and_no_cacheability_is_claimed() {
    let analysis = analyze("dma-region", "firmware.elf");

    let dmaram = analysis
        .map_evidence
        .as_ref()
        .and_then(|m| m.regions.iter().find(|r| r.name == "DMARAM"))
        .expect("a region named for its use, not for any name the product knows");
    assert_eq!(dmaram.origin, 0x3000_0000);
    assert_eq!(dmaram.length, 0x0000_4000);

    let buffer = contribution(&analysis, ".dma_buffer");
    assert_eq!(buffer.nonvolatile_bytes.value(), Some(&64));
    assert_eq!(
        buffer.runtime_ram_bytes.value(),
        Some(&64),
        "an unusual name must not drop the memory"
    );

    assert_eq!(
        charge(&analysis.memory.nonvolatile),
        120,
        "20 text + 32 rodata + 4 data + 64 buffer"
    );
    assert_eq!(
        charge(&analysis.memory.runtime_ram),
        72,
        "4 data + 64 buffer + 4 bss"
    );

    // Commit E adds no cacheability dimension (§17). The section name says DMA; the evidence says
    // writable RAM. Nothing the product reports may upgrade one into the other.
    let haystack = format!("{:?}", analysis.memory);
    let lowered = haystack.to_lowercase();
    for word in ["cache", "coherent", "non-cacheable"] {
        assert!(
            !lowered.contains(word),
            "the model must not claim {word} semantics it cannot derive"
        );
    }
}

// ---- case E: a MAP whose recognizable content starts late --------------------------------------

#[test]
fn a_map_whose_banner_sits_past_every_fixed_window_still_parses_and_analyzes() {
    // 220 unreferenced functions, discarded by `--gc-sections`, push the table this far in. The
    // bound is the claim; the measured value at generation was 16,571 bytes.
    let map_path = compat("long-preamble", "firmware.map");
    let text = std::fs::read_to_string(&map_path).expect("MAP is UTF-8");
    let offset = text
        .find("Memory Configuration")
        .expect("a GNU ld MAP declares its regions");
    assert!(
        offset > 4096,
        "this fixture only proves late-banner recognition while its banner is past the old window; \
         it is at byte {offset}"
    );

    assert!(map::detect(&text).expect("detection must not error on GNU ld output"));
    let evidence = map::parse(&text).expect("the long-preamble MAP must parse");
    assert_eq!(evidence.adapter_id, "gnu_ld");
    for name in ["FLASH", "RAM"] {
        assert!(
            evidence.regions.iter().any(|r| r.name == name),
            "the region table past the preamble must still parse: {name} is missing"
        );
    }

    let analysis = analyze("long-preamble", "firmware.elf");
    assert_eq!(
        charge(&analysis.memory.nonvolatile),
        4,
        "only main survives --gc-sections"
    );
    assert_eq!(charge(&analysis.memory.runtime_ram), 0);
    assert!(
        !evidence.object_contributions.is_empty(),
        "the discarded preamble still names the object each input section came from"
    );
}

// ---- case F: debug info present, excluded, and not a DWARF claim --------------------------------

#[test]
fn debug_sections_cost_real_bytes_and_enter_neither_budget() {
    let analysis = analyze("clang-arm", "firmware.elf");

    let debug: Vec<_> = analysis
        .snapshot
        .sections()
        .iter()
        .filter(|s| s.role == SectionRole::Debug)
        .collect();
    assert!(
        debug.len() >= 4,
        "clang was asked to emit debug info; found {}",
        debug.len()
    );
    assert!(
        debug.iter().any(|s| s.file_size > 0),
        "the exclusion must be of real bytes, not of empty sections"
    );
    for section in &debug {
        assert!(
            !analysis.memory.contributions.iter().any(|c| c
                .section_name
                .value()
                .map(String::as_str)
                == section.name.value().map(String::as_str)),
            "a debug section must not be charged to a device budget: {:?}",
            section.name.value()
        );
    }
    assert!(
        analysis.memory.excluded_metadata_bytes > 1000,
        "debug bytes are accounted as metadata"
    );

    // DWARF is recognized, never consumed. There is no semantic DWARF feature to claim.
    let lowered = format!("{:?}", analysis.capabilities).to_lowercase();
    assert!(
        !lowered.contains("dwarf"),
        "no DWARF analysis may be claimed by a capability"
    );
}

// ---- case G: built without debug, and stripped after link ---------------------------------------

#[test]
fn a_build_without_debug_and_a_post_link_strip_are_two_different_facts() {
    let nodebug = pipeline::analyze(
        &AnalysisRequest::new(compat("no-debug", "firmware-nodebug.elf"))
            .with_map(compat("no-debug", "firmware.map")),
    )
    .expect("the no-debug ELF analyzes");
    let stripped = pipeline::analyze(
        &AnalysisRequest::new(compat("no-debug", "firmware-stripped.elf"))
            .with_map(compat("no-debug", "firmware.map")),
    )
    .expect("the stripped ELF analyzes");

    // Measured on this binutils: -g built with no debug keeps its symbol table; objcopy -S took
    // .symtab and .strtab with the debug sections. So the pair differs in two ways and says so.
    assert_eq!(
        nodebug.snapshot.symbols().len(),
        17,
        "built without -g, symbols still present"
    );
    assert_eq!(
        stripped.snapshot.symbols().len(),
        0,
        "post-link strip removed the symbol table"
    );
    assert!(
        nodebug
            .snapshot
            .sections()
            .iter()
            .all(|s| s.role != SectionRole::Debug),
        "neither file carries debug info"
    );

    // Absence is not corruption: the memory model answers with the same exact numbers for both.
    for (name, analysis) in [("nodebug", &nodebug), ("stripped", &stripped)] {
        assert_eq!(analysis.input.format, DetectedFormat::Elf32Le, "{name}");
        assert_eq!(
            charge(&analysis.memory.nonvolatile),
            48,
            "{name}: 12 text + 32 rodata + 4 data"
        );
        assert_eq!(
            charge(&analysis.memory.runtime_ram),
            8,
            "{name}: 4 data + 4 bss"
        );
        assert!(
            matches!(analysis.memory.nonvolatile, ByteTotal::Exact { .. }),
            "{name}"
        );
        assert!(
            matches!(analysis.memory.runtime_ram, ByteTotal::Exact { .. }),
            "{name}"
        );
        assert_eq!(
            analysis
                .snapshot
                .sections()
                .iter()
                .find(|s| s.name.value().map(String::as_str) == Some(".text"))
                .and_then(|s| s.virtual_address.value()),
            Some(&0x0800_0000),
            "{name}: the entry region is unaffected by what was removed"
        );
    }
    assert_eq!(
        nodebug.snapshot.symbols().len().min(1),
        1,
        "no fake symbols for the stripped file"
    );
}

// ---- case H: multiple loadable segments ---------------------------------------------------------

#[test]
fn real_linkers_emit_several_load_segments_and_the_fixtures_say_how_many() {
    // Counted with readelf at generation, then asserted from the header here. The claim Commit E
    // needs from case H is that the model reads a multi-segment image, so the segment counts are
    // facts on the record, and `p0-dual-region` already carried three without any test naming it.
    let cases = [
        ("fixtures/elf/p0-basic/firmware.elf", 2),
        ("fixtures/elf/p0-dual-region/firmware.elf", 3),
        ("fixtures/elf/p5-compat/ram-exec/firmware.elf", 2),
        ("fixtures/elf/p5-compat/extsram/firmware.elf", 4),
        ("fixtures/elf/p5-compat/dma-region/firmware.elf", 4),
        ("fixtures/elf/p5-compat/clang-arm/firmware.elf", 2),
    ];
    for (relative, expected) in cases {
        let path = fixture(relative);
        let count = load_segments(&path);
        assert_eq!(count, expected, "{relative} load segments");
        assert!(
            count >= 2,
            "{relative} must be a multi-segment image to serve case H"
        );
    }

    // And the accounting follows the segments rather than a `.text == FLASH` shortcut: the RAM
    // image with three load-bearing regions still totals exactly.
    let analysis = analyze("extsram", "firmware.elf");
    assert_eq!(load_segments(&compat("extsram", "firmware.elf")), 4);
    assert_eq!(charge(&analysis.memory.nonvolatile), 184);
}

// ---- the defect this round's evidence found, and fixed -----------------------------------------

#[test]
fn an_allocated_section_the_parser_does_not_recognise_is_still_charged() {
    // What the Clang fixture was for. clang emits `.ARM.exidx.text.main` for `main`, GNU ld keeps
    // it inside the declared FLASH region with SHF_ALLOC set, and the model used to infer
    // "not allocated" from "unrecognised section kind" — so eight real image bytes entered neither
    // budget while the total still reported `Exact` with nothing in `unattributed`.
    // `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §3 settles a custom section by segment and region
    // evidence, never by its name, and that is what the numbers below show.
    let analysis = analyze("clang-arm", "firmware.elf");

    let exidx = analysis
        .snapshot
        .sections()
        .iter()
        .find(|s| s.name.value().map(String::as_str) == Some(".ARM.exidx.text.main"))
        .expect("clang's unwind index section is in the file the linker wrote");
    assert!(
        exidx.flags.alloc,
        "the header sets SHF_ALLOC, so it is allocated: {exidx:?}"
    );
    assert_eq!(exidx.file_size, 8, "and it carries eight bytes of payload");
    assert_eq!(
        exidx.role,
        SectionRole::Unknown,
        "the parser still does not recognise what it is"
    );
    assert!(exidx.has_file_payload());

    let indexed = contribution(&analysis, ".ARM.exidx.text.main");
    assert_eq!(
        indexed.nonvolatile_bytes.value(),
        Some(&8),
        "it occupies the load image"
    );
    assert_eq!(
        indexed.runtime_ram_bytes.value(),
        Some(&0),
        "and no RAM, because FLASH is not writable"
    );
    assert_eq!(
        indexed.basis,
        MemoryEvidenceBasis::MapRegionAndElfLoad,
        "it is charged from region evidence, not from a name the model happens to know"
    );

    assert_eq!(
        charge(&analysis.memory.nonvolatile),
        58,
        "14 text + 32 rodata + 8 exidx + 4 data"
    );
    assert_eq!(charge(&analysis.memory.runtime_ram), 8);
}

#[test]
fn the_reported_image_footprint_equals_the_bytes_the_file_claims_are_loaded() {
    // The invariant that was broken, written once for every committed ELF instead of for the one
    // fixture that caught it, and read against the section headers rather than against the model's own
    // flag derivation. Before the fix the clang fixture reported 50 against 58 allocated bytes, and
    // every other file already agreed — which is why nothing in the cohort could see the gap.
    let elfs = committed_elfs();
    assert_eq!(
        elfs.len(),
        11,
        "the manifest records eleven ELF fixtures; a different count means a fixture went missing,          not that this assertion is stale"
    );

    for relative in &elfs {
        let analysis = analyze_relative(relative);
        assert_eq!(
            charge(&analysis.memory.nonvolatile),
            raw_allocated_payload(&repo_root().join(relative)),
            "{relative}: the image footprint must equal the allocated payload the file stores"
        );
    }
}

#[test]
fn the_identity_the_product_reports_is_the_identity_the_file_declares() {
    // §25's identity fields, asserted against each file's own header for the whole cohort. `e_machine`
    // is why the answer is `Arm` and not `Thumb`: the machine comes from the header, and thumb is an
    // instruction-set state this field does not carry.
    for relative in committed_elfs() {
        let analysis = analyze_relative(&relative);
        let artifact = analysis
            .snapshot
            .artifacts()
            .first()
            .unwrap_or_else(|| panic!("{relative}: expected one artifact"));
        let (entry, class, data, machine) = elf32_identity(&repo_root().join(&relative));

        assert_eq!(data, 1, "{relative} declares little-endian");
        assert_eq!(artifact.endianness, Endianness::Little, "{relative}");
        assert_eq!(artifact.kind, ArtifactKind::Elf, "{relative}");
        assert_eq!(analysis.input.format, DetectedFormat::Elf32Le, "{relative}");
        assert_eq!(class, 1, "{relative} declares ELFCLASS32");
        assert_eq!(artifact.bitness, Bitness::Bits32, "{relative}");
        assert_eq!(machine, 40, "{relative} declares EM_ARM");
        assert_eq!(artifact.architecture, Architecture::Arm, "{relative}");
        assert_eq!(
            artifact.entry_point.value(),
            Some(&entry),
            "{relative}: the entry the product reports is the entry the file declares"
        );
    }
}

#[test]
fn the_provenance_record_and_the_product_answer_the_same_numbers() {
    // Each `fixture.toml` carries two budget figures derived from readelf and the MAP's region table,
    // with no help from the analyzer. This test holds the product to them, which is what keeps the
    // record honest if the model changes and the record honest if the record is edited by hand.
    for set in [
        "ram-exec",
        "extsram",
        "dma-region",
        "long-preamble",
        "clang-arm",
        "no-debug",
    ] {
        let record = text_of(&compat(set, "fixture.toml"));
        let elf = if set == "no-debug" {
            "firmware-nodebug.elf"
        } else {
            "firmware.elf"
        };
        let analysis = analyze(set, elf);
        let expected = |key: &str| -> u64 {
            record
                .lines()
                .find_map(|line| line.strip_prefix(&format!("{key} = \"\"\"")))
                .unwrap_or_else(|| panic!("{set}: {key} is not recorded in fixture.toml"))
                .split(" = ")
                .next()
                .and_then(|head| head.parse::<u64>().ok())
                .unwrap_or_else(|| panic!("{set}: {key} does not lead with a number"))
        };

        assert_eq!(
            charge(&analysis.memory.nonvolatile),
            expected("expected_image_bytes"),
            "{set}: the record and the product disagree about the image"
        );
        assert_eq!(
            charge(&analysis.memory.runtime_ram),
            expected("expected_live_ram_bytes"),
            "{set}: the record and the product disagree about live RAM"
        );

        let entry = record
            .lines()
            .find_map(|line| line.strip_prefix("entry_point = \"\"\""))
            .and_then(|rest| rest.split('"').next())
            .expect("the record names the entry point");
        assert_eq!(
            analysis
                .snapshot
                .artifacts()
                .first()
                .and_then(|a| a.entry_point.value())
                .map(|value| format!("0x{value:08x}"))
                .as_deref(),
            Some(entry),
            "{set}: the entry the file was linked with is the entry the record and product both give"
        );
    }
}
