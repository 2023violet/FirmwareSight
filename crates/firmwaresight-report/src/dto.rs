//! The CLI/DTO projection of a `BuildSnapshot`.
//!
//! Field declaration order below *is* the emitted JSON key order, which is what makes the
//! output byte-stable without a custom canonicalizer. Nothing here is a `HashMap`, nothing here
//! carries a wall-clock timestamp, and nothing here carries an absolute host path.
//!
//! Every fact is read off the snapshot; none of it is recomputed here. That is what stops the
//! CLI and the desktop from drifting apart.

use serde::Serialize;

use firmwaresight_core::domain::build_snapshot::{BuildSnapshot, NORMALIZATION_VERSION};
use firmwaresight_core::domain::capability::Capabilities;
use firmwaresight_core::domain::evidence::{Confidence, EvidenceClass, SourceType};
use firmwaresight_core::domain::identity::{Bitness, Endianness, Fact};
use firmwaresight_core::domain::memory::{
    ByteTotal, LayoutSource, MemoryContribution, MemoryEvidenceBasis,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeResultDto {
    pub schema: &'static str,
    pub schema_stability: &'static str,
    pub created_by_fwsight_version: String,
    pub normalization_version: &'static str,
    pub snapshot_id: String,
    pub artifact: ArtifactDto,
    pub memory: MemoryDto,
    pub counts: CountsDto,
    pub capabilities: CapabilitiesDto,
    pub evidence: Vec<EvidenceDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactDto {
    /// File name only. A full path would make the output machine-specific; the human renderer
    /// prints the path exactly as the user supplied it.
    pub file_name: String,
    pub kind: String,
    pub sha256: String,
    pub byte_size: u64,
    pub parser_id: String,
    pub architecture: String,
    pub bitness: String,
    pub endianness: String,
    pub entry_point: Option<String>,
    pub entry_point_unknown_reason: Option<String>,
    pub build_id: Option<String>,
    pub build_id_unknown_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryDto {
    pub accounting_rule: &'static str,
    pub layout_source: &'static str,
    pub weakest_evidence_basis: Option<&'static str>,
    pub admissible_for_hard_block: bool,
    pub nonvolatile_image_footprint: BudgetDto,
    pub runtime_ram_footprint: BudgetDto,
    pub contributions: Vec<ContributionDto>,
    pub excluded_metadata_bytes: u64,
    pub dual_accounted_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetDto {
    /// `exact`, `partial` or `unknown`.
    pub state: &'static str,
    pub classification: &'static str,
    pub bytes: Option<u64>,
    pub unattributed: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionDto {
    pub section_locator: String,
    pub section_name: Option<String>,
    pub nonvolatile_bytes: Option<u64>,
    pub runtime_ram_bytes: Option<u64>,
    pub evidence_basis: &'static str,
    pub classification: &'static str,
    pub accounting_rule: &'static str,
    pub dual_accounted: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CountsDto {
    pub sections: usize,
    pub symbols: usize,
    pub artifacts: usize,
    pub evidence: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesDto {
    pub elf: &'static str,
    pub sections: &'static str,
    pub symbols: &'static str,
    pub debug_info: &'static str,
    pub map: &'static str,
    pub object_attribution: &'static str,
    pub git: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceDto {
    pub id: String,
    pub field: String,
    pub classification: &'static str,
    pub source_type: &'static str,
    pub source_locator: String,
    pub value: String,
    pub rule: String,
    pub confidence: Option<&'static str>,
}

const fn class_text(class: EvidenceClass) -> &'static str {
    match class {
        EvidenceClass::Observed => "observed",
        EvidenceClass::Derived => "derived",
        EvidenceClass::Declared => "declared",
        EvidenceClass::Unknown => "unknown",
    }
}

const fn source_text(source: SourceType) -> &'static str {
    match source {
        SourceType::ElfFileHeader => "elf.file-header",
        SourceType::ElfSectionHeader => "elf.section-header",
        SourceType::ElfProgramHeader => "elf.program-header",
        SourceType::ElfSymbolTable => "elf.symbol-table",
        SourceType::ElfNote => "elf.note",
        SourceType::MapFile => "map",
        SourceType::MemoryRegionConfig => "memory-region-config",
        SourceType::UserDeclaration => "user-declaration",
        SourceType::EmbeddedVersion => "embedded-version",
        SourceType::GitCli => "git-cli",
        SourceType::FileSystem => "filesystem",
        SourceType::RuleEngine => "rule-engine",
    }
}

const fn confidence_text(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
    }
}

const fn basis_text(basis: MemoryEvidenceBasis) -> &'static str {
    match basis {
        MemoryEvidenceBasis::RegionConfigAndElfLoad => "region-config+elf-load",
        MemoryEvidenceBasis::MapRegionAndElfLoad => "map-memory-configuration+elf-load",
        MemoryEvidenceBasis::ElfAddressAndFlags => "elf-address-and-flags",
        MemoryEvidenceBasis::SectionNameHeuristic => "section-name-heuristic",
        MemoryEvidenceBasis::Insufficient => "unattributed",
    }
}

const fn layout_text(source: LayoutSource) -> &'static str {
    match source {
        LayoutSource::ProjectConfig => "project-config",
        LayoutSource::MapMemoryConfiguration => "map",
        LayoutSource::None => "none",
    }
}

const fn bitness_text(bitness: Bitness) -> &'static str {
    match bitness {
        Bitness::Bits32 => "32",
        Bitness::Bits64 => "64",
        Bitness::Unknown => "unknown",
    }
}

const fn endianness_text(endian: Endianness) -> &'static str {
    match endian {
        Endianness::Little => "little",
        Endianness::Big => "big",
        Endianness::Unknown => "unknown",
    }
}

fn budget(total: &ByteTotal) -> BudgetDto {
    match total {
        ByteTotal::Exact { bytes } => BudgetDto {
            state: "exact",
            classification: class_text(total.classification()),
            bytes: Some(*bytes),
            unattributed: Vec::new(),
            reason: None,
        },
        ByteTotal::Partial {
            bytes,
            unattributed,
            reason,
        } => BudgetDto {
            state: "partial",
            classification: class_text(total.classification()),
            bytes: Some(*bytes),
            unattributed: unattributed.clone(),
            reason: Some(reason.clone()),
        },
        ByteTotal::Unknown { reason } => BudgetDto {
            state: "unknown",
            classification: class_text(total.classification()),
            bytes: None,
            unattributed: Vec::new(),
            reason: Some(reason.clone()),
        },
    }
}

fn contribution(c: &MemoryContribution) -> ContributionDto {
    ContributionDto {
        section_locator: c.section_locator.clone(),
        section_name: c.section_name.value().cloned(),
        nonvolatile_bytes: c.nonvolatile_bytes.value().copied(),
        runtime_ram_bytes: c.runtime_ram_bytes.value().copied(),
        evidence_basis: basis_text(c.basis),
        classification: class_text(c.basis.classification()),
        accounting_rule: c.rule,
        dual_accounted: c.is_dual_accounted(),
    }
}

impl From<&Capabilities> for CapabilitiesDto {
    fn from(caps: &Capabilities) -> Self {
        Self {
            elf: match caps.elf {
                firmwaresight_core::domain::capability::FormatSupport::Supported => "supported",
                firmwaresight_core::domain::capability::FormatSupport::Partial => "partial",
                firmwaresight_core::domain::capability::FormatSupport::Unsupported => "unsupported",
                firmwaresight_core::domain::capability::FormatSupport::Unknown => "unknown",
            },
            sections: availability_text(caps.sections),
            symbols: availability_text(caps.symbols),
            debug_info: availability_text(caps.debug_info),
            map: match caps.map {
                firmwaresight_core::domain::capability::Provision::Provided => "provided",
                firmwaresight_core::domain::capability::Provision::NotProvided => "not-provided",
                firmwaresight_core::domain::capability::Provision::Unknown => "unknown",
            },
            object_attribution: availability_text(caps.object_attribution),
            git: availability_text(caps.git),
        }
    }
}

const fn availability_text(
    value: firmwaresight_core::domain::capability::Availability,
) -> &'static str {
    match value {
        firmwaresight_core::domain::capability::Availability::Available => "available",
        firmwaresight_core::domain::capability::Availability::Partial => "partial",
        firmwaresight_core::domain::capability::Availability::Unavailable => "unavailable",
        firmwaresight_core::domain::capability::Availability::Unknown => "unknown",
    }
}

fn fact_string<T: std::fmt::Debug>(fact: &Fact<T>) -> (Option<String>, Option<String>) {
    match fact {
        Fact::Known(value) => (Some(format!("{value:?}")), None),
        Fact::Unknown { reason } => (None, Some(reason.clone())),
    }
}

impl AnalyzeResultDto {
    #[must_use]
    pub fn from_snapshot(snapshot: &BuildSnapshot, file_name: &str) -> Self {
        let memory = snapshot
            .memory()
            .expect("a P0 snapshot always carries a memory footprint");
        let primary = snapshot
            .primary_artifact()
            .expect("a sealed snapshot has at least one artifact");

        let (entry_point, entry_reason) = match &primary.entry_point {
            Fact::Known(value) => (Some(format!("{value:#010x}")), None),
            Fact::Unknown { reason } => (None, Some(reason.clone())),
        };
        let (build_id, build_reason) = fact_string(&primary.build_id);

        Self {
            schema: crate::ANALYZE_SCHEMA_ID,
            schema_stability: crate::SCHEMA_STABILITY,
            created_by_fwsight_version: snapshot.created_by_fwsight_version().to_owned(),
            normalization_version: NORMALIZATION_VERSION,
            snapshot_id: snapshot.id().as_str().to_owned(),
            artifact: ArtifactDto {
                file_name: file_name.to_owned(),
                kind: format!("{:?}", primary.kind).to_ascii_lowercase(),
                sha256: primary.sha256.hex().to_owned(),
                byte_size: primary.byte_size,
                parser_id: primary.parser_id.0.clone(),
                architecture: format!("{:?}", primary.architecture),
                bitness: bitness_text(primary.bitness).to_owned(),
                endianness: endianness_text(primary.endianness).to_owned(),
                entry_point,
                entry_point_unknown_reason: entry_reason,
                build_id,
                build_id_unknown_reason: build_reason,
            },
            memory: MemoryDto {
                accounting_rule: "adr-0021-dual-budget",
                layout_source: layout_text(memory.layout_source),
                weakest_evidence_basis: memory.weakest_basis.map(basis_text),
                admissible_for_hard_block: memory.admissible_for_hard_block(),
                nonvolatile_image_footprint: budget(&memory.nonvolatile),
                runtime_ram_footprint: budget(&memory.runtime_ram),
                contributions: memory.contributions.iter().map(contribution).collect(),
                excluded_metadata_bytes: memory.excluded_metadata_bytes,
                dual_accounted_sections: memory
                    .dual_accounted_sections()
                    .into_iter()
                    .map(|c| c.section_locator.clone())
                    .collect(),
            },
            counts: CountsDto {
                sections: snapshot.section_count(),
                symbols: snapshot.symbol_count(),
                artifacts: snapshot.artifacts().len(),
                evidence: snapshot.evidence().len(),
            },
            capabilities: CapabilitiesDto::from(snapshot.capabilities()),
            evidence: snapshot
                .evidence()
                .iter()
                .map(|item| EvidenceDto {
                    id: item.id.clone(),
                    field: item.field.clone(),
                    classification: class_text(item.classification),
                    source_type: source_text(item.source_type),
                    source_locator: item.source_locator.clone(),
                    value: item.raw_value.clone(),
                    rule: item.rule.clone(),
                    confidence: item.confidence.map(confidence_text),
                })
                .collect(),
        }
    }
}
