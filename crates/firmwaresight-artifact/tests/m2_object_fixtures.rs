//! ADR-0033 against real GNU ld output: the committed `fixtures/elf/m2-objects` MAPs, built by
//! arm-none-eabi-gcc 14.3.1 with -ffunction-sections and --gc-sections from two objects and a library.

use std::path::PathBuf;

use firmwaresight_artifact::map::{self, LinkerBytesKind, MapEvidence};

fn evidence(side: &str) -> MapEvidence {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/elf/m2-objects")
        .join(side)
        .join("firmware.map");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    map::parse(&text).expect("the fixture MAP parses")
}

fn bytes_of(evidence: &MapEvidence, key: &str, output: &str) -> u64 {
    evidence
        .object_contributions
        .iter()
        .filter(|c| c.object == key && c.output_section.as_deref() == Some(output))
        .map(|c| c.size)
        .sum()
}

#[test]
fn every_object_and_library_member_is_found_under_a_project_relative_key() {
    let target = evidence("target");
    let mut keys: Vec<&str> = target
        .object_contributions
        .iter()
        .map(|c| c.object.as_str())
        .collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(
        keys,
        [
            "libdrv.a(spi.o)",
            "libdrv.a(uart.o)",
            "main.o",
            "sensors/imu.o"
        ]
    );
    for c in &target.object_contributions {
        assert!(
            !c.object.contains("build/"),
            "the shared build directory is stripped: {}",
            c.object
        );
    }
}

#[test]
fn objects_plus_linker_bytes_account_for_every_byte_of_each_output_section() {
    for side in ["base", "target"] {
        let evidence = evidence(side);
        for placement in evidence.placements.iter().filter(|p| {
            p.size > 0
                && !p.section.starts_with(".debug")
                && p.section != ".comment"
                && p.section != ".ARM.attributes"
        }) {
            let objects: u64 = evidence
                .object_contributions
                .iter()
                .filter(|c| c.output_section.as_deref() == Some(placement.section.as_str()))
                .map(|c| c.size)
                .sum();
            let linker: u64 = evidence
                .linker_bytes
                .iter()
                .filter(|b| b.output_section.as_deref() == Some(placement.section.as_str()))
                .map(|b| b.size)
                .sum();
            assert_eq!(
                objects + linker,
                placement.size,
                "{side} {}",
                placement.section
            );
        }
    }
}

#[test]
fn the_discarded_helper_is_not_attributed_and_padding_is_named() {
    let base = evidence("base");
    assert!(
        base.object_contributions
            .iter()
            .all(|c| !c.input_section.contains("unused_helper"))
    );
    assert!(
        base.linker_bytes
            .iter()
            .any(|b| b.kind == LinkerBytesKind::Fill)
    );
}

#[test]
fn the_target_grew_where_the_source_did() {
    let (base, target) = (evidence("base"), evidence("target"));
    assert!(
        bytes_of(&target, "sensors/imu.o", ".text") > bytes_of(&base, "sensors/imu.o", ".text")
    );
    assert_eq!(
        bytes_of(&base, "libdrv.a(spi.o)", ".text"),
        0,
        "spi is not linked in the base"
    );
    assert!(bytes_of(&target, "libdrv.a(spi.o)", ".text") > 0);
    assert_eq!(
        bytes_of(&base, "libdrv.a(uart.o)", ".text"),
        bytes_of(&target, "libdrv.a(uart.o)", ".text"),
        "uart did not change"
    );
    assert!(
        bytes_of(&target, "main.o", ".bss") > 0,
        "COMMON g_samples is attributed to main.o"
    );
}
