//! End-to-end analysis for the P0 slice: intake, parse, normalize, classify, account.
//!
//! Stage order is fixed by `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md`. Everything returned to
//! callers is core domain types plus this crate's own evidence, never `object` or MAP parser
//! types.

use std::path::PathBuf;

use crate::elf;
use crate::error::ArtifactError;
use crate::intake::{DetectedFormat, GuardConfig, GuardedInput};
use crate::map::{self, MapEvidence};
use firmwaresight_core::domain::artifact::{Artifact, ParserId};
use firmwaresight_core::domain::capability::{Availability, Capabilities, Provision};
use firmwaresight_core::domain::evidence::{EvidenceClass, EvidenceItem, SourceType};
use firmwaresight_core::domain::identity::{
    ArtifactKind, ArtifactTimes, BuildIdentity, Fact, Sha256,
};
use firmwaresight_core::domain::memory::MemoryFootprint;
use firmwaresight_core::domain::section::Section;

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

    let mut facts = elf::parse(&input.bytes, &input.sha256)?;

    let mut map_input = None;
    let mut map_evidence = None;

    if let Some(map_path) = &request.map_path {
        let candidate = GuardedInput::load(map_path, request.guard)?;
        let text =
            std::str::from_utf8(&candidate.bytes).map_err(|_| ArtifactError::MapUnsupported {
                detected: "the MAP file is not UTF-8 text".to_owned(),
            })?;
        let evidence = map::parse(text)?;
        apply_map_evidence(&mut facts.sections, &evidence);
        map_input = Some(candidate);
        map_evidence = Some(evidence);
    }

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

    let snapshot =
        firmwaresight_core::domain::build_snapshot::SnapshotBuilder::new(FWSIGHT_VERSION)
            .project_id("p0-technical-slice")
            .artifact(artifact)
            .memory(memory.clone())
            .sections(facts.sections.clone())
            .symbols(facts.symbols.clone())
            .evidence(evidence)
            .capabilities(capabilities.clone())
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
    })
}

/// Attach MAP-derived region and load-address evidence to the parsed sections.
fn apply_map_evidence(sections: &mut [Section], evidence: &MapEvidence) {
    for section in sections {
        let Some(name) = section.name.value() else {
            continue;
        };
        let Some(placement) = evidence.placement_for(name) else {
            continue;
        };

        // GNU ld prints `load address` only when it differs from the runtime address, so an
        // absent clause legitimately means the two are equal.
        let load_address = placement.load_address.unwrap_or(placement.address);
        section.load_address = Fact::known(load_address);

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
            if map_evidence.is_some() {
                Availability::Available
            } else {
                Availability::Unavailable
            },
        )
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
        format!("file:{}", crate::error::display_path(&input.path)),
        "sha256",
        input.sha256.hex().to_owned(),
        "streaming-sha256/64KiB",
    ));

    items.push(EvidenceItem::new(
        "ev-byte-size",
        EvidenceClass::Observed,
        SourceType::FileSystem,
        format!("stat:{}", crate::error::display_path(&input.path)),
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
        items.push(EvidenceItem::new(
            format!("ev-memory-{field}"),
            total.classification(),
            SourceType::ElfProgramHeader,
            "memory-accounting/totals",
            field,
            value,
            memory
                .weakest_basis
                .map_or("unattributed", |basis| basis_rule_name(basis)),
        ));
    }

    // The dual-accounted sections are the interesting provenance: name the exact lines.
    for contribution in memory.dual_accounted_sections() {
        let mut item = EvidenceItem::new(
            format!("ev-dual-{}", contribution.section_index),
            EvidenceClass::Observed,
            SourceType::MapFile,
            format!("{} + map:load-address", contribution.section_locator),
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
        if contribution.basis.classification() != EvidenceClass::Observed {
            item = item.with_confidence(firmwaresight_core::domain::evidence::Confidence::Low);
        }
        items.push(item);
    }

    if let Some(evidence) = map_evidence {
        items.push(EvidenceItem::new(
            "ev-map-regions",
            EvidenceClass::Observed,
            SourceType::MapFile,
            "map:Memory Configuration",
            "memory_regions",
            evidence
                .regions
                .iter()
                .map(|r| format!("{}@{:#x}+{:#x}", r.name, r.origin, r.length))
                .collect::<Vec<_>>()
                .join(","),
            map::ADAPTER_ID,
        ));
    }

    items
}

const fn basis_rule_name(
    basis: firmwaresight_core::domain::memory::MemoryEvidenceBasis,
) -> &'static str {
    match basis {
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::RegionConfigAndElfLoad => {
            "region-config+elf-load"
        }
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::MapRegionAndElfLoad => {
            "map-memory-configuration+elf-load"
        }
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::ElfAddressAndFlags => {
            "elf-address-and-flags"
        }
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::SectionNameHeuristic => {
            "section-name-heuristic"
        }
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::Insufficient => "unattributed",
    }
}
