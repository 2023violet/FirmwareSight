//! ADR-0036 against real armlink output: the committed `fixtures/elf/d4-armlink` sets, built by Arm Compiler
//! for Embedded 6.24 (Keil MDK 5.43) from the same sources as `m2-objects`. Every figure below is checked
//! against armlink's own `--info=totals` and GNU readelf (`evidence/D4/CROSSCHECK.md`): base ROM 108 / RW 12,
//! target ROM 192 / RW 44.

use std::path::PathBuf;

use firmwaresight_artifact::map::{self, MapEvidence};
use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::attribution::summarize;
use firmwaresight_core::domain::memory::RegionKind;

fn fixture(side: &str, file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/elf/d4-armlink")
        .join(side)
        .join(file)
}

fn evidence(side: &str) -> MapEvidence {
    let text = std::fs::read_to_string(fixture(side, "firmware.map")).expect("fixture MAP");
    map::parse_any(&text).expect("the armlink MAP parses")
}

fn analysis(side: &str) -> pipeline::Analysis {
    let request =
        AnalysisRequest::new(fixture(side, "firmware.axf")).with_map(fixture(side, "firmware.map"));
    pipeline::analyze(&request).expect("the armlink build analyzes")
}

#[test]
fn the_armlink_adapter_reads_the_execution_regions_as_the_region_table() {
    let target = evidence("target");
    assert_eq!(target.adapter_id, "armlink");
    let regions: Vec<_> = target
        .regions
        .iter()
        .map(|r| (r.name.as_str(), r.origin, r.length, r.kind))
        .collect();
    assert_eq!(
        regions,
        vec![
            ("ER_IROM1", 0x0800_0000, 0x1_0000, RegionKind::Nonvolatile),
            ("RW_IRAM1", 0x2000_0000, 0x4000, RegionKind::VolatileRam),
        ]
    );
    let rw = target
        .placements
        .iter()
        .find(|p| p.section == "RW_IRAM1")
        .expect("RW placement");
    assert_eq!(
        rw.load_address,
        Some(0x0800_00bc),
        "--load_addr_map_info gives the Load base"
    );
}

#[test]
fn footprints_with_the_map_equal_armlinks_totals_and_are_map_backed() {
    for (side, rom, rw) in [("base", 108, 12), ("target", 192, 44)] {
        let analysis = analysis(side);
        assert!(analysis.memory.nonvolatile.is_exact(), "{side}");
        assert_eq!(
            analysis.memory.nonvolatile.bytes(),
            Some(rom),
            "{side}: armlink Total ROM"
        );
        assert_eq!(
            analysis.memory.runtime_ram.bytes(),
            Some(rw),
            "{side}: armlink Total RW"
        );
        assert_eq!(
            analysis.memory.layout_source,
            firmwaresight_core::domain::memory::LayoutSource::MapMemoryConfiguration,
            "{side}"
        );
        let usage: Vec<_> = analysis
            .memory
            .region_usage
            .iter()
            .map(|u| (u.name.as_str(), u.capacity_bytes, u.used_bytes))
            .collect();
        assert_eq!(
            usage,
            vec![
                ("ER_IROM1", Some(0x1_0000), Some(rom)),
                ("RW_IRAM1", Some(0x4000), Some(rw))
            ],
            "{side}"
        );
        assert_eq!(
            analysis
                .snapshot
                .artifacts()
                .iter()
                .find(|a| a.kind == firmwaresight_core::domain::identity::ArtifactKind::Map)
                .map(|a| a.parser_id.0.as_str()),
            Some("map-adapter/armlink")
        );
    }
}

#[test]
fn objects_and_library_members_are_charged_like_their_own_section_not_a_namesake() {
    // armlink gives the RW data and the ZI part of RW_IRAM1 one name; each row is charged by the section
    // that holds its address: ZI costs RAM only.
    let analysis = analysis("target");
    let summary = summarize(analysis.snapshot.objects(), &analysis.memory);
    let row = |key: &str| {
        summary
            .objects
            .iter()
            .find(|o| o.key == key)
            .map(|o| (o.nonvolatile, o.runtime_ram))
            .unwrap_or_else(|| panic!("{key} missing"))
    };
    assert_eq!(row("imu.o"), (Some(92), Some(0)));
    assert_eq!(row("main.o"), (Some(62), Some(4)));
    assert_eq!(row("libdrv.lib(spi.o)"), (Some(24), Some(32)));
    assert_eq!(row("libdrv.lib(uart.o)"), (Some(12), Some(4)));
    assert_eq!(
        row("anon$$obj.o"),
        (Some(0), Some(4)),
        "armlink's object for common symbols"
    );
    let library = summary
        .libraries
        .iter()
        .find(|l| l.key == "libdrv.lib")
        .expect("library");
    assert_eq!(
        (library.nonvolatile, library.runtime_ram),
        (Some(36), Some(36))
    );
    // The 2-byte PAD row is the linker's: it is the only Flash no object accounts for.
    assert_eq!(summary.unattributed_nonvolatile, Some(2));
    assert_eq!(summary.unattributed_runtime_ram, Some(0));
}

#[test]
fn an_iar_map_and_an_armlink_listing_without_a_memory_map_are_still_refused() {
    let iar = "** IAR Linker\nPLACEBLOCKS\n";
    assert!(map::parse_any(iar).is_err());
    let banner_only = "Component: Arm Compiler for Embedded 6.24 Tool: armlink [5f371500]\n";
    let err = map::parse_any(banner_only).expect_err("no image map");
    assert_eq!(err.stable_code(), "ERR-MAP-3001");
}
