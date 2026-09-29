//! The MAP must be sealed as a companion artifact, and evidence must name the source it came from.
//!
//! Two defects this file pins down, both found by the P1-A0 real desktop smoke and both closed by
//! the correctness-closure round:
//!
//! 1. `pipeline::analyze` loaded a supplied MAP into the memory model but sealed only the ELF into
//!    the snapshot, so `SnapshotId::compose` received `map_sha256 = None`. ELF-only and ELF+MAP
//!    produced the same id, and storage dedupe then refused the stronger build.
//! 2. The dual-accounting evidence records were labelled `SourceType::MapFile` with a
//!    `map:load-address` locator even for a run that was given no MAP at all.
//!
//! Both are provenance claims, so every assertion here reads what the snapshot actually says rather
//! than what the caller asked for.

use std::path::{Path, PathBuf};

use firmwaresight_artifact::pipeline::{self, AnalysisRequest};
use firmwaresight_core::domain::evidence::{EvidenceClass, SourceType};
use firmwaresight_core::domain::identity::{Architecture, ArtifactKind, Bitness, Endianness};

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

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let unique = format!(
            "fwsight-map-companion-{label}-{}-{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let path = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory creates");
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn elf_only() -> firmwaresight_artifact::pipeline::Analysis {
    pipeline::analyze(&AnalysisRequest::new(dual_region_elf())).expect("the committed ELF analyzes")
}

fn elf_with_map() -> firmwaresight_artifact::pipeline::Analysis {
    pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(dual_region_map()))
        .expect("the committed ELF + MAP pair analyzes")
}

/// A second valid GNU ld MAP over the same ELF: same banner, same placements, a wider RAM region.
/// Only the region length changes, so it is a different piece of evidence about the same build
/// rather than a malformed file.
fn wider_ram_map(dir: &TempDir) -> PathBuf {
    let text = std::fs::read_to_string(dual_region_map()).expect("the fixture MAP is readable");
    let variant = text.replace(
        "RAM              0x20000000         0x00040000         xrw",
        "RAM              0x20000000         0x00080000         xrw",
    );
    assert!(
        variant != text,
        "the fixture MAP carries the RAM region row"
    );
    let path = dir.join("wider-ram.map");
    std::fs::write(&path, variant).expect("the variant writes");
    path
}

#[test]
fn an_elf_only_snapshot_seals_one_artifact() {
    assert_eq!(elf_only().snapshot.artifacts().len(), 1);
}

#[test]
fn a_map_makes_the_snapshot_seal_a_companion_artifact() {
    let analysis = elf_with_map();
    let artifacts = analysis.snapshot.artifacts();

    assert_eq!(
        artifacts.len(),
        2,
        "the MAP is an input, so it is an artifact"
    );
    assert_eq!(artifacts[0].kind, ArtifactKind::Elf);
    assert_eq!(artifacts[1].kind, ArtifactKind::Map);
    assert_eq!(
        analysis.snapshot.primary_artifact().map(|a| a.kind),
        Some(ArtifactKind::Elf),
        "the snapshot is still addressed by the ELF"
    );
}

#[test]
fn the_companion_artifact_reports_the_map_bytes_and_the_gnu_ld_adapter() {
    let analysis = elf_with_map();
    let map = &analysis.snapshot.artifacts()[1];

    assert_eq!(
        map.sha256,
        analysis.map_input.as_ref().expect("MAP loaded").sha256,
        "the companion artifact hashes the MAP bytes, not the ELF bytes"
    );
    assert_eq!(
        map.byte_size,
        analysis.map_input.as_ref().expect("MAP loaded").byte_size
    );
    assert_eq!(
        map.parser_id,
        firmwaresight_core::domain::artifact::ParserId::gnu_ld_map()
    );
    assert_ne!(map.sha256, analysis.snapshot.artifacts()[0].sha256);
}

#[test]
fn the_companion_artifact_claims_no_elf_facts_a_map_cannot_have() {
    let analysis = elf_with_map();
    let map = &analysis.snapshot.artifacts()[1];

    assert_eq!(map.architecture, Architecture::Unknown);
    assert_eq!(map.bitness, Bitness::Unknown);
    assert_eq!(map.endianness, Endianness::Unknown);
    assert!(
        map.entry_point.value().is_none() && map.entry_point.reason_if_unknown().is_some(),
        "a linker MAP has no entry point, so claiming one would be invented"
    );
    assert!(
        map.build_id.value().is_none() && map.build_id.reason_if_unknown().is_some(),
        "a linker MAP carries no build-id"
    );
}

#[test]
fn a_map_changes_the_snapshot_id_through_the_existing_optional_map_component() {
    let without = elf_only();
    let with = elf_with_map();

    assert_eq!(
        without.snapshot.artifacts()[0].sha256,
        with.snapshot.artifacts()[0].sha256,
        "the same ELF bytes are being described in both runs"
    );
    assert_ne!(
        without.snapshot.id(),
        with.snapshot.id(),
        "different evidence inputs must not collide onto one immutable snapshot"
    );
    let map_sha = with.snapshot.artifacts()[1].sha256.hex();
    assert!(
        with.snapshot.id().as_str().contains(map_sha),
        "the id must carry the MAP hash the identity rule already defines: {}",
        with.snapshot.id().as_str()
    );
    assert!(
        !without.snapshot.id().as_str().contains(map_sha),
        "an ELF-only id must not claim a MAP it never saw"
    );
}

#[test]
fn the_same_pair_produces_the_same_id_and_a_different_map_produces_another() {
    let dir = TempDir::new("ids");
    let first = elf_with_map();
    let second = elf_with_map();

    assert_eq!(
        first.snapshot.id(),
        second.snapshot.id(),
        "stable inputs, stable id"
    );

    let wider =
        pipeline::analyze(&AnalysisRequest::new(dual_region_elf()).with_map(wider_ram_map(&dir)))
            .expect("the wider-RAM MAP is a valid GNU ld MAP");

    assert_ne!(
        first.snapshot.id(),
        wider.snapshot.id(),
        "different MAP bytes are different evidence"
    );
}

#[test]
fn closing_the_identity_gap_needs_no_normalization_version_bump() {
    // The identity function already had a slot for the MAP hash; supplying that input is a bug
    // fix, not a new normalization rule, so old ELF-only ids must keep their meaning.
    let analysis = elf_with_map();
    let id = analysis.snapshot.id().as_str();

    assert!(
        id.contains(firmwaresight_core::domain::build_snapshot::NORMALIZATION_VERSION),
        "every id still carries `p0-normalize-1`: {id}"
    );
    assert_eq!(
        firmwaresight_core::domain::build_snapshot::NORMALIZATION_VERSION,
        "p0-normalize-1"
    );
    assert_eq!(
        elf_only().snapshot.id().as_str(),
        format!(
            "snap-{}-p0-normalize-1",
            analysis.snapshot.artifacts()[0].sha256.hex()
        ),
        "the ELF-only id formula is unchanged"
    );
}

#[test]
fn an_elf_only_run_names_no_map_in_its_evidence() {
    // Both committed fixtures dual-account, and both were analyzed without a MAP when the P0
    // goldens were cut - which is why both goldens carried the false `map` label.
    for elf in [dual_region_elf(), basic_elf()] {
        let analysis = pipeline::analyze(&AnalysisRequest::new(elf)).expect("ELF alone analyzes");
        let dual: Vec<_> = analysis
            .snapshot
            .evidence()
            .iter()
            .filter(|e| e.field == "dual_accounted")
            .collect();

        assert!(
            !dual.is_empty(),
            "this fixture really does dual-account without a MAP, so this test would pass \
             vacuously if it stopped being true"
        );
        for item in &dual {
            assert_ne!(
                item.source_type,
                SourceType::MapFile,
                "no MAP was supplied, so `{}` may not claim map provenance",
                item.id
            );
            assert!(
                !item.source_locator.contains("map:"),
                "locator `{}` names a MAP record that does not exist in this run",
                item.source_locator
            );
        }
        assert!(
            !analysis
                .snapshot
                .evidence()
                .iter()
                .any(|e| e.source_type == SourceType::MapFile),
            "an ELF-only run has no map-sourced evidence item at all"
        );
    }
}

#[test]
fn a_map_backed_run_uses_the_map_source_and_keeps_observed_items_observed() {
    let analysis = elf_with_map();
    let dual: Vec<_> = analysis
        .snapshot
        .evidence()
        .iter()
        .filter(|e| e.field == "dual_accounted")
        .collect();

    assert!(!dual.is_empty());
    for item in &dual {
        assert_eq!(
            item.source_type,
            SourceType::MapFile,
            "this charge really did come from the MAP region table"
        );
        assert!(
            item.source_locator.contains("elf.section_header")
                && item.source_locator.contains("map:"),
            "the locator must name both inputs it rests on: {}",
            item.source_locator
        );
        assert_eq!(item.classification, EvidenceClass::Observed);
    }

    assert!(
        analysis
            .snapshot
            .evidence()
            .iter()
            .any(|e| e.field == "memory_regions" && e.source_type == SourceType::MapFile),
        "the MAP's own region facts stay recorded"
    );
}

#[test]
fn each_charge_records_the_source_and_class_its_own_rule_actually_supports() {
    // The invariant behind the fix: the accounting rule names the evidence a number rests on, so
    // the source type, the locator and the evidence class must all agree with that rule or the
    // record claims a provenance the run never had.
    fn expected_for(rule: &str) -> Option<(SourceType, EvidenceClass, bool)> {
        match rule {
            "map-memory-configuration+elf-load" => {
                Some((SourceType::MapFile, EvidenceClass::Observed, true))
            }
            "region-config+elf-load" => Some((
                SourceType::MemoryRegionConfig,
                EvidenceClass::Observed,
                false,
            )),
            "elf-address-and-flags" => {
                Some((SourceType::ElfProgramHeader, EvidenceClass::Observed, false))
            }
            "section-name-heuristic" => {
                Some((SourceType::ElfSectionHeader, EvidenceClass::Derived, false))
            }
            "unattributed" => Some((SourceType::RuleEngine, EvidenceClass::Unknown, false)),
            _ => None,
        }
    }

    for analysis in [elf_only(), elf_with_map()] {
        let mut seen = 0_usize;
        for item in analysis
            .snapshot
            .evidence()
            .iter()
            .filter(|e| e.field == "dual_accounted")
        {
            let (source, class, names_map) = expected_for(&item.rule)
                .unwrap_or_else(|| panic!("{} claims an unknown rule: {}", item.id, item.rule));
            seen += 1;

            assert_eq!(
                item.source_type, source,
                "{} charged by `{}`",
                item.id, item.rule
            );
            assert_eq!(
                item.classification, class,
                "{} charged by `{}`",
                item.id, item.rule
            );
            assert_eq!(
                item.source_locator.contains("map:"),
                names_map,
                "locator `{}` must name a MAP only for rule `{}`",
                item.source_locator,
                item.rule
            );
            if class != EvidenceClass::Observed {
                assert_eq!(
                    item.confidence,
                    Some(firmwaresight_core::domain::evidence::Confidence::Low),
                    "a non-observed charge must be marked low-confidence: {}",
                    item.id
                );
            }
        }
        assert!(seen > 0, "both fixtures dual-account at least one section");
    }
}
