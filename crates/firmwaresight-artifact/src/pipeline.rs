//! End-to-end analysis for the P0 slice: intake, parse, normalize, classify, account.
//!
//! Stage order is fixed by `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md`. Everything returned to
//! callers is core domain types plus this crate's own evidence, never `object` or MAP parser
//! types.

use std::path::{Path, PathBuf};

use crate::elf;
use crate::error::ArtifactError;
use crate::intake::{DetectedFormat, GuardConfig, GuardedInput};
use crate::map::{self, MapEvidence};
use firmwaresight_core::domain::artifact::{Artifact, ParserId};
use firmwaresight_core::domain::capability::{Availability, Capabilities, Provision};
use firmwaresight_core::domain::evidence::{EvidenceClass, EvidenceItem, SourceType};
use firmwaresight_core::domain::identity::{
    Architecture, ArtifactKind, ArtifactTimes, Bitness, BuildIdentity, Endianness, Fact, Sha256,
};
use firmwaresight_core::domain::memory::{MemoryEvidenceBasis, MemoryFootprint};
use firmwaresight_core::domain::section::Section;
use firmwaresight_core::domain::symbol::Symbol;

pub const FWSIGHT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct AnalysisRequest {
    pub artifact: PathBuf,
    pub map_path: Option<PathBuf>,
    pub guard: GuardConfig,
}

impl AnalysisRequest {
    #[must_use]
    pub fn new(artifact: impl Into<PathBuf>) -> Self {
        Self {
            artifact: artifact.into(),
            map_path: None,
            guard: GuardConfig::default(),
        }
    }

    #[must_use]
    pub fn with_map(mut self, map_path: impl Into<PathBuf>) -> Self {
        self.map_path = Some(map_path.into());
        self
    }

    #[must_use]
    pub fn with_guard(mut self, guard: GuardConfig) -> Self {
        self.guard = guard;
        self
    }
}

/// Everything the CLI and the desktop shell render. Both derive their views from this one value.
#[derive(Debug, Clone)]
pub struct Analysis {
    pub snapshot: firmwaresight_core::domain::build_snapshot::BuildSnapshot,
    pub input: GuardedInput,
    pub map_input: Option<GuardedInput>,
    pub map_evidence: Option<MapEvidence>,
    pub capabilities: Capabilities,
    pub memory: MemoryFootprint,
    /// Wall-clock stage timings for `P0_PERFORMANCE_REPORT.md`.
    ///
    /// They live here and never in the DTO: a benchmark number must not leak into the
    /// deterministic payload, whose whole value is being reproducible.
    pub parse_ms: u128,
    pub normalize_ms: u128,
}

impl Analysis {
    #[must_use]
    pub fn sha256(&self) -> &Sha256 {
        &self.input.sha256
    }
}

/// Run the pipeline. A missing MAP or a missing symbol table degrades capabilities; neither
/// fails the analysis.
pub fn analyze(request: &AnalysisRequest) -> Result<Analysis, ArtifactError> {
    let input = GuardedInput::load(&request.artifact, request.guard)?;

    if !matches!(
        input.format,
        DetectedFormat::Elf32Le
            | DetectedFormat::Elf32Be
            | DetectedFormat::Elf64Le
            | DetectedFormat::Elf64Be
    ) {
        return Err(ArtifactError::UnsupportedFormat {
            detected: format!(
                "{:?} at {}",
                input.format,
                crate::error::display_path(&request.artifact)
            ),
        });
    }

    let parse_start = std::time::Instant::now();
    let mut facts = elf::parse(&input.bytes, &input.sha256)?;

    let mut map_input = None;
    let mut map_evidence = None;

    if let Some(map_path) = &request.map_path {
        let candidate = GuardedInput::load(map_path, request.guard)?;
        let text =
            std::str::from_utf8(&candidate.bytes).map_err(|_| ArtifactError::MapUnsupported {
                detected: "the MAP file is not UTF-8 text".to_owned(),
            })?;
        let evidence = map::parse_any(text)?;
        apply_map_evidence(&mut facts.sections, &evidence);
        map_input = Some(candidate);
        map_evidence = Some(evidence);
    } else {
        mark_compressed_initialization_data(&mut facts.sections, &facts.symbols);
    }
    let parse_ms = parse_start.elapsed().as_millis();

    let normalize_start = std::time::Instant::now();
    let layout = map_evidence.as_ref().map_or_else(
        firmwaresight_core::domain::memory::MemoryLayout::empty,
        MapEvidence::layout,
    );
    let memory = MemoryFootprint::compute(&facts.sections, &layout);

    let capabilities = build_capabilities(&facts, map_evidence.as_ref());
    let evidence = build_evidence(&input, &facts, &memory, map_evidence.as_ref());

    let artifact = Artifact {
        id: format!("art-{}", input.sha256.hex()),
        path: crate::error::display_path(&input.path),
        kind: ArtifactKind::Elf,
        sha256: input.sha256.clone(),
        byte_size: input.byte_size,
        parser_id: ParserId::elf_object("0.40"),
        architecture: facts.architecture.clone(),
        bitness: facts.bitness,
        endianness: facts.endianness,
        entry_point: facts.entry_point.clone(),
        build_id: facts.build_id.clone(),
        times: ArtifactTimes::default(),
        identity: BuildIdentity::default(),
        evidence: evidence.clone(),
    };

    let mut builder =
        firmwaresight_core::domain::build_snapshot::SnapshotBuilder::new(FWSIGHT_VERSION)
            .project_id("p0-technical-slice")
            .artifact(artifact);

    // The MAP is an input the user supplied, so it is sealed as the build's second artifact.
    // `SnapshotId::compose` already takes an optional MAP hash and `seal()` already looks for an
    // artifact of kind `Map`; omitting this row was what left `map_sha256` = None and let an
    // ELF+MAP build collide onto the ELF-only id, which in turn made storage dedupe skip the
    // stronger import. Index 1 keeps the primary artifact at index 0.
    if let Some(map) = &map_input {
        builder = builder.artifact(Artifact {
            id: format!("art-{}", map.sha256.hex()),
            path: crate::error::display_path(&map.path),
            kind: ArtifactKind::Map,
            sha256: map.sha256.clone(),
            byte_size: map.byte_size,
            parser_id: map_evidence
                .as_ref()
                .map_or_else(ParserId::gnu_ld_map, |evidence| {
                    ParserId::map_adapter(evidence.adapter_id)
                }),
            // A linker MAP names no machine, no entry point and no build-id. Claiming the ELF's
            // values here would attribute facts to a file that cannot carry them.
            architecture: Architecture::Unknown,
            bitness: Bitness::Unknown,
            endianness: Endianness::Unknown,
            entry_point: Fact::unknown("a linker MAP records addresses, not an entry point"),
            build_id: Fact::unknown("a linker MAP carries no build-id"),
            times: ArtifactTimes::default(),
            identity: BuildIdentity::default(),
            // The MAP's own observed facts are the `ev-map-*` items on the snapshot; repeating them
            // per artifact would count one claim twice.
            evidence: Vec::new(),
        });
    }

    let snapshot = builder
        .memory(memory.clone())
        .sections(facts.sections.clone())
        .symbols(facts.symbols.clone())
        .evidence(evidence)
        .capabilities(capabilities.clone())
        .objects(object_rows(map_evidence.as_ref()))
        .seal()
        .map_err(|err| ArtifactError::InternalBug {
            detail: format!("{err:?}"),
        })?;

    Ok(Analysis {
        snapshot,
        input,
        map_input,
        map_evidence,
        capabilities,
        memory,
        parse_ms,
        normalize_ms: normalize_start.elapsed().as_millis(),
    })
}

/// armlink's decompressor in the image means some RW data is stored compressed: its ELF section holds the
/// compressed bytes, so the section size is not its runtime size (ADR-0036 A1). Without the MAP nothing
/// says which section or how large it runs, so every initialized writable section's runtime size is Unknown
/// rather than reported as the stored size.
fn mark_compressed_initialization_data(sections: &mut [Section], symbols: &[Symbol]) {
    let decompressor = symbols.iter().any(|symbol| {
        symbol
            .name
            .value()
            .is_some_and(|name| name.starts_with("__decompress"))
    });
    if !decompressor {
        return;
    }
    for section in sections
        .iter_mut()
        .filter(|s| s.flags.alloc && s.flags.write && s.has_file_payload())
    {
        section.memory_size = Fact::unknown(
            "the image contains armlink's decompressor, so this RW data may be stored compressed and its runtime size is not in the ELF; analyze with the linker MAP",
        );
    }
}

/// Attach MAP-derived region and load-address evidence to the parsed sections.
fn apply_map_evidence(sections: &mut [Section], evidence: &MapEvidence) {
    for section in sections.iter_mut() {
        let Some(name) = section.name.value() else {
            continue;
        };
        let address = section.virtual_address.value().copied();
        let placement = match address {
            Some(address) => evidence.placement_containing(name, address),
            None => evidence.placement_for(name),
        };
        let Some(placement) = placement else {
            continue;
        };

        // GNU ld prints `load address` only when it differs from the runtime address, so an absent clause
        // legitimately means the two are equal. An armlink region that cannot say is unknown (ADR-0036).
        // A section that starts inside its placement keeps its offset on the load side too.
        if let Some(why) = placement.load_unknown {
            section.load_address = Fact::unknown(why);
        } else {
            // The offset applies only to a section the placement actually holds (armlink's ZI part of a
            // region); otherwise the placement's own load address stands, as it always did for GNU ld.
            let offset = address
                .filter(|a| *a >= placement.address && *a - placement.address < placement.size)
                .map_or(0, |a| a - placement.address);
            section.load_address = match placement
                .load_address
                .unwrap_or(placement.address)
                .checked_add(offset)
            {
                Some(load_address) => Fact::known(load_address),
                None => Fact::unknown(
                    "the MAP's load address plus the section's offset does not fit in 64 bits",
                ),
            };
        }

        // `region` means where the section lives at runtime; the load side is carried by
        // `load_address` so the accounting rule can charge both.
        if let Some(region) = evidence
            .regions
            .iter()
            .find(|r| r.contains(placement.address))
        {
            section.region = Fact::known(region.name.clone());
        }
    }

    // ADR-0036 A1: an armlink region with compressed RW data keeps the compressed bytes in its ELF data
    // section, so that section's size is the stored size, not the runtime size. The MAP's region Size is the
    // runtime size; the data section is that Size less the region's other (ZI) sections.
    for placement in evidence
        .placements
        .iter()
        .filter(|p| p.runtime_size.is_some())
    {
        let Some(runtime) = placement.runtime_size else {
            continue;
        };
        let inside = |section: &Section| {
            section
                .name
                .value()
                .is_some_and(|n| n == &placement.section)
                && section.virtual_address.value().is_some_and(|a| {
                    *a >= placement.address && *a - placement.address < runtime.max(1)
                })
        };
        // Sizes come from user input: an overflowing sum is a malformed image, not a number.
        let zero_init: Option<u64> = sections
            .iter()
            .filter(|s| inside(s) && !s.has_file_payload())
            .filter_map(|s| s.memory_size.value().copied())
            .try_fold(0_u64, u64::checked_add);
        let data: Vec<usize> = (0..sections.len())
            .filter(|&i| inside(&sections[i]) && sections[i].has_file_payload())
            .collect();
        if let [only] = data.as_slice() {
            sections[*only].memory_size = match zero_init.map(|zi| runtime.checked_sub(zi)) {
                Some(Some(bytes)) => Fact::known(bytes),
                Some(None) => Fact::unknown(
                    "the MAP's region size is smaller than the region's zero-initialized sections",
                ),
                None => Fact::unknown(
                    "the region's zero-initialized section sizes overflow when added; the image is malformed",
                ),
            };
        } else {
            // Several data sections share one compressed region: the MAP gives only their total, so no
            // section's runtime size is known, and the stored size must not stand in for it.
            for &index in &data {
                sections[index].memory_size = Fact::unknown(
                    "armlink compressed this region and it holds several data sections; the MAP gives only the region's total runtime size",
                );
            }
        }
    }
}

/// The MAP's object rows as Core records them (ADR-0033). Keys are already free of host paths.
fn object_rows(
    map_evidence: Option<&MapEvidence>,
) -> Vec<firmwaresight_core::domain::attribution::ObjectContribution> {
    map_evidence
        .map(|evidence| {
            evidence
                .object_contributions
                .iter()
                .map(
                    |row| firmwaresight_core::domain::attribution::ObjectContribution {
                        object: row.object.clone(),
                        archive: row.archive.clone(),
                        member: row.member.clone(),
                        input_section: row.input_section.clone(),
                        output_section: row.output_section.clone(),
                        address: row.address,
                        size: row.size,
                        map_line: row.line as u64,
                    },
                )
                .collect()
        })
        .unwrap_or_default()
}

fn build_capabilities(facts: &elf::ElfFacts, map_evidence: Option<&MapEvidence>) -> Capabilities {
    Capabilities::elf_only()
        .with_symbols(if facts.symbol_table_present {
            Availability::Available
        } else {
            Availability::Unavailable
        })
        .with_debug_info(facts.debug_support)
        .with_map(
            if map_evidence.is_some() {
                Provision::Provided
            } else {
                Provision::NotProvided
            },
            // ADR-0033 D6: attribution exists exactly when the MAP named at least one object row, and
            // those rows travel in the snapshot (`object_rows`). A MAP with no rows, or no MAP, is
            // Unavailable rather than an empty success.
            if map_evidence.is_some_and(|evidence| !evidence.object_contributions.is_empty()) {
                Availability::Available
            } else {
                Availability::Unavailable
            },
        )
}

/// A locator fragment that survives being run from a different directory or machine.
///
/// Deterministic output must not embed an absolute host path; the full path still reaches the
/// human renderer and the local database, where it is genuinely useful.
fn stable_path_token(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed-artifact".to_owned())
}

/// The rule, source, locator and evidence class an accounting basis genuinely supports.
///
/// One function returns all four because they describe the same thing: a record that named the
/// right rule but the wrong source was exactly how the second P1-A0 defect happened. The
/// dual-accounting rows used to print `SourceType::MapFile` and `+ map:load-address` whatever the
/// basis turned out to be, so an ELF analyzed with no MAP at all claimed MAP provenance - both
/// committed CLI goldens carried that label.
fn basis_provenance(
    basis: MemoryEvidenceBasis,
    section_locator: &str,
) -> (&'static str, SourceType, String, EvidenceClass) {
    match basis {
        MemoryEvidenceBasis::MapRegionAndElfLoad => (
            "map-memory-configuration+elf-load",
            SourceType::MapFile,
            format!("{section_locator} + map:load-address"),
            EvidenceClass::Observed,
        ),
        MemoryEvidenceBasis::RegionConfigAndElfLoad => (
            "region-config+elf-load",
            SourceType::MemoryRegionConfig,
            format!("{section_locator} + region-config:load-address"),
            EvidenceClass::Observed,
        ),
        MemoryEvidenceBasis::ElfAddressAndFlags => (
            "elf-address-and-flags",
            SourceType::ElfProgramHeader,
            format!("{section_locator} + elf:sh_flags"),
            EvidenceClass::Observed,
        ),
        MemoryEvidenceBasis::SectionNameHeuristic => (
            "section-name-heuristic",
            SourceType::ElfSectionHeader,
            format!("{section_locator} + name-heuristic"),
            EvidenceClass::Derived,
        ),
        // Nothing supported this charge, so no source is claimed for it beyond the rule itself.
        MemoryEvidenceBasis::Insufficient => (
            "unattributed",
            SourceType::RuleEngine,
            section_locator.to_owned(),
            EvidenceClass::Unknown,
        ),
    }
}

/// Record how each headline fact became known, so a golden can be checked for provenance and
/// not only for arithmetic.
fn build_evidence(
    input: &GuardedInput,
    facts: &elf::ElfFacts,
    memory: &MemoryFootprint,
    map_evidence: Option<&MapEvidence>,
) -> Vec<EvidenceItem> {
    let mut items = Vec::new();

    items.push(EvidenceItem::new(
        "ev-sha256",
        EvidenceClass::Observed,
        SourceType::FileSystem,
        format!("file:{}", stable_path_token(&input.path)),
        "sha256",
        input.sha256.hex().to_owned(),
        "streaming-sha256/64KiB",
    ));

    items.push(EvidenceItem::new(
        "ev-byte-size",
        EvidenceClass::Observed,
        SourceType::FileSystem,
        format!("stat:{}", stable_path_token(&input.path)),
        "byte_size",
        input.byte_size.to_string(),
        "fs::metadata.len",
    ));

    items.push(EvidenceItem::new(
        "ev-architecture",
        EvidenceClass::Observed,
        SourceType::ElfFileHeader,
        "elf.e_machine",
        "architecture",
        format!("{:?}", facts.architecture),
        elf::PARSER_ID,
    ));

    items.push(EvidenceItem::new(
        "ev-bitness",
        EvidenceClass::Observed,
        SourceType::ElfFileHeader,
        "elf.e_ident[EI_CLASS]",
        "bitness",
        format!("{:?}", facts.bitness),
        elf::PARSER_ID,
    ));

    items.push(EvidenceItem::new(
        "ev-endianness",
        EvidenceClass::Observed,
        SourceType::ElfFileHeader,
        "elf.e_ident[EI_DATA]",
        "endianness",
        format!("{:?}", facts.endianness),
        elf::PARSER_ID,
    ));

    if let Some(value) = facts.entry_point.value() {
        items.push(EvidenceItem::new(
            "ev-entry",
            EvidenceClass::Observed,
            SourceType::ElfFileHeader,
            "elf.e_entry",
            "entry_point",
            format!("{value:#010x}"),
            elf::PARSER_ID,
        ));
    }

    for budget in [
        ("nonvolatile_image_footprint_bytes", &memory.nonvolatile),
        ("runtime_ram_footprint_bytes", &memory.runtime_ram),
    ] {
        let (field, total) = budget;
        let value = total
            .bytes()
            .map_or_else(|| "unknown".to_owned(), |bytes| bytes.to_string());
        // The totals inherit the weakest basis of the charges they sum, so their source is named
        // from that same rung. `memory-accounting/totals` stays the locator: it names the aggregate,
        // not a file, so it cannot claim a record this run did not read.
        let (rule, source, _, _) = basis_provenance(
            memory
                .weakest_basis
                .unwrap_or(MemoryEvidenceBasis::Insufficient),
            "memory-accounting",
        );
        items.push(EvidenceItem::new(
            format!("ev-memory-{field}"),
            total.classification(),
            source,
            "memory-accounting/totals",
            field,
            value,
            rule,
        ));
    }

    // The dual-accounted sections are the interesting provenance: name the exact lines.
    for contribution in memory.dual_accounted_sections() {
        let (_, source, locator, class) =
            basis_provenance(contribution.basis, &contribution.section_locator);
        let mut item = EvidenceItem::new(
            format!("ev-dual-{}", contribution.section_index),
            class,
            source,
            locator,
            "dual_accounted",
            format!(
                "nonvolatile={} runtime={}",
                contribution
                    .nonvolatile_bytes
                    .value()
                    .map_or_else(|| "?".to_owned(), |v| v.to_string()),
                contribution
                    .runtime_ram_bytes
                    .value()
                    .map_or_else(|| "?".to_owned(), |v| v.to_string()),
            ),
            contribution.rule,
        );
        if class != EvidenceClass::Observed {
            item = item.with_confidence(firmwaresight_core::domain::evidence::Confidence::Low);
        }
        items.push(item);
    }

    if let Some(evidence) = map_evidence {
        items.push(EvidenceItem::new(
            "ev-map-regions",
            EvidenceClass::Observed,
            SourceType::MapFile,
            if evidence.adapter_id == crate::map_armlink::ADAPTER_ID {
                "map:Execution Regions"
            } else {
                "map:Memory Configuration"
            },
            "memory_regions",
            evidence
                .regions
                .iter()
                .map(|r| format!("{}@{:#x}+{:#x}", r.name, r.origin, r.length))
                .collect::<Vec<_>>()
                .join(","),
            evidence.adapter_id,
        ));
    }

    items
}

#[cfg(test)]
mod map_application_tests {
    use super::*;
    use crate::map::MapPlacement;
    use firmwaresight_core::domain::section::{SectionFlags, SectionRole};

    fn section(
        index: usize,
        name: &str,
        address: u64,
        file_size: u64,
        memory_size: u64,
    ) -> Section {
        Section {
            index,
            name: Fact::known(name.to_owned()),
            role: if file_size > 0 {
                SectionRole::InitializedData
            } else {
                SectionRole::UninitializedData
            },
            flags: SectionFlags {
                alloc: true,
                write: true,
                ..SectionFlags::default()
            },
            virtual_address: Fact::known(address),
            load_address: Fact::unknown("not in the section header table"),
            file_offset: Fact::unknown("test"),
            file_size,
            memory_size: Fact::known(memory_size),
            region: Fact::unknown("test"),
        }
    }

    #[test]
    fn a_compressed_armlink_region_is_loaded_from_its_load_base_and_sized_by_the_map() {
        // The CMSIS-DSP FIR example built by Keil MDK: RW_RAM holds 428 compressed bytes in its ELF data
        // section and 1620 bytes of ZI; the MAP says the region is 0x1060 (4192) bytes at runtime.
        let mut sections = vec![
            section(2, "RW_RAM", 0x2000_0000, 0x1ac, 0x1ac),
            section(3, "RW_RAM", 0x2000_0a0c, 0, 0x654),
        ];
        let evidence = MapEvidence {
            adapter_id: crate::map_armlink::ADAPTER_ID,
            regions: Vec::new(),
            placements: vec![MapPlacement {
                section: "RW_RAM".to_owned(),
                address: 0x2000_0000,
                size: 0x1060,
                load_address: Some(0x194c),
                load_unknown: None,
                runtime_size: Some(0x1060),
                line: 1,
            }],
            symbols: Vec::new(),
            object_contributions: Vec::new(),
            linker_bytes: Vec::new(),
            emulation: None,
        };
        apply_map_evidence(&mut sections, &evidence);
        assert_eq!(
            sections[0].load_address,
            Fact::known(0x194c),
            "stored at its Load base"
        );
        assert_eq!(
            sections[0].file_size, 0x1ac,
            "Flash keeps the compressed size"
        );
        assert_eq!(
            sections[0].memory_size,
            Fact::known(0xa0c),
            "runtime is the region less its ZI: 2572"
        );
        assert_eq!(
            sections[1].memory_size,
            Fact::known(0x654),
            "ZI is untouched"
        );
    }

    fn compressed_region(size: u64) -> MapEvidence {
        MapEvidence {
            adapter_id: crate::map_armlink::ADAPTER_ID,
            regions: Vec::new(),
            placements: vec![MapPlacement {
                section: "RW_RAM".to_owned(),
                address: 0x2000_0000,
                size,
                load_address: Some(0x194c),
                load_unknown: None,
                runtime_size: Some(size),
                line: 1,
            }],
            symbols: Vec::new(),
            object_contributions: Vec::new(),
            linker_bytes: Vec::new(),
            emulation: None,
        }
    }

    #[test]
    fn a_compressed_region_never_reports_a_stored_size_as_a_runtime_size() {
        // Two data sections in one compressed region: only their total is known.
        let mut sections = vec![
            section(2, "RW_RAM", 0x2000_0000, 0x10, 0x10),
            section(3, "RW_RAM", 0x2000_0100, 0x20, 0x20),
        ];
        apply_map_evidence(&mut sections, &compressed_region(0x1060));
        assert!(sections.iter().all(|s| s.memory_size.value().is_none()));

        // ZI sizes from a crafted image that overflow when added: Unknown, never a panic.
        let mut sections = vec![
            section(2, "RW_RAM", 0x2000_0000, 0x10, 0x10),
            section(3, "RW_RAM", 0x2000_0100, 0, u64::MAX),
            section(4, "RW_RAM", 0x2000_0200, 0, u64::MAX),
        ];
        apply_map_evidence(&mut sections, &compressed_region(0x1060));
        assert!(sections[0].memory_size.value().is_none());
    }

    fn symbol(name: &str) -> Symbol {
        Symbol {
            name: Fact::known(name.to_owned()),
            address: Fact::known(0x425),
            size: Fact::known(0x64),
            kind: firmwaresight_core::domain::symbol::SymbolKind::Function,
            binding: firmwaresight_core::domain::symbol::SymbolBinding::Global,
            section: firmwaresight_core::domain::symbol::SymbolSectionRef::Index(1),
        }
    }

    #[test]
    fn without_the_map_a_decompressor_makes_rw_data_runtime_size_unknown() {
        let mut sections = vec![
            section(2, "RW_RAM", 0x2000_0000, 0x1ac, 0x1ac),
            section(3, "RW_RAM", 0x2000_0a0c, 0, 0x654),
        ];
        mark_compressed_initialization_data(&mut sections, &[symbol("main")]);
        assert_eq!(
            sections[0].memory_size,
            Fact::known(0x1ac),
            "no decompressor: unchanged"
        );

        mark_compressed_initialization_data(&mut sections, &[symbol("__decompress")]);
        assert!(
            sections[0].memory_size.value().is_none(),
            "the stored size is not the runtime size"
        );
        assert_eq!(
            sections[0].file_size, 0x1ac,
            "Flash still holds the stored bytes"
        );
        assert_eq!(
            sections[1].memory_size,
            Fact::known(0x654),
            "ZI is not compressed"
        );
    }
}
