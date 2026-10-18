//! Memory accounting.
//!
//! This is the module `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` and `ADR-0021` describe: two
//! budgets that are modelled and budgeted separately, and an evidence ladder that decides how
//! confidently a section may be attributed to either.
//!
//! The rule that matters most: a section may be charged to the nonvolatile image **and** to
//! runtime RAM at the same time, because an initialized global stores its initial value in the
//! load image and its live copy in RAM. Whether that happened is decided by load evidence,
//! never by the section being called `.data`.

use std::collections::HashMap;

use crate::domain::evidence::EvidenceClass;
use crate::domain::identity::Fact;
use crate::domain::section::{Section, SectionRole};

/// Where an attributed byte range physically lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionKind {
    /// Not writable: holds the loaded image.
    Nonvolatile,
    /// Writable storage: counts against runtime RAM.
    VolatileRam,
    /// Attributes did not say.
    Unknown,
}

/// GNU ld / linker-script region attributes, e.g. `xr` for ROM and `xrw` for RAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RegionAttributes {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    /// `i` in a linker script region means initialized data is loaded there.
    pub read_init_data: bool,
    /// `d` marks a region that has file content.
    pub has_content: bool,
}

impl RegionAttributes {
    /// Parse a linker-script attribute string. Unrecognized characters are ignored rather
    /// than guessed at.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        let mut attrs = Self::default();
        for ch in raw.chars() {
            match ch {
                'r' => attrs.read = true,
                'w' => attrs.write = true,
                'x' => attrs.execute = true,
                'i' => attrs.read_init_data = true,
                'd' => attrs.has_content = true,
                _ => {}
            }
        }
        attrs
    }

    /// Writability is the discriminator: a linker-script region declared without `w` cannot
    /// hold live variables, and one declared with `w` is RAM. `x` is carried alongside on RAM
    /// regions (`xrw`), so it must not be read as "this is flash".
    #[must_use]
    pub const fn kind(self) -> RegionKind {
        if self.write {
            RegionKind::VolatileRam
        } else if self.read || self.execute || self.read_init_data || self.has_content {
            RegionKind::Nonvolatile
        } else {
            RegionKind::Unknown
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRegion {
    pub name: String,
    pub origin: u64,
    pub length: u64,
    pub attributes: RegionAttributes,
    pub kind: RegionKind,
    pub evidence_id: String,
}

impl MemoryRegion {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        origin: u64,
        length: u64,
        attributes_raw: &str,
        evidence_id: impl Into<String>,
    ) -> Self {
        let attributes = RegionAttributes::parse(attributes_raw);
        let kind = attributes.kind();
        Self {
            name: name.into(),
            origin,
            length,
            attributes,
            kind,
            evidence_id: evidence_id.into(),
        }
    }

    #[must_use]
    pub fn contains(&self, address: u64) -> bool {
        self.length > 0
            && address >= self.origin
            && address
                .checked_sub(self.origin)
                .is_some_and(|off| off < self.length)
    }
}

/// How the region table was obtained, which caps the evidence class of attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayoutSource {
    /// Project-supplied memory-region configuration.
    ProjectConfig,
    /// A supported MAP adapter produced the `Memory Configuration` block.
    MapMemoryConfiguration,
    /// No region table at all.
    #[default]
    None,
}

impl LayoutSource {
    /// The label this source is written down as. The database column, the analysis screen and the
    /// portable diff document all carry this same string, so a reader never has to know the name of
    /// a Rust enum to follow where a layout came from.
    #[must_use]
    pub const fn as_label(self) -> &'static str {
        match self {
            Self::ProjectConfig => "project-config",
            Self::MapMemoryConfiguration => "map",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemoryLayout {
    pub regions: Vec<MemoryRegion>,
    pub source: LayoutSource,
}

impl MemoryLayout {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn region_containing(&self, address: u64) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(address))
    }

    #[must_use]
    pub fn region_named(&self, name: &str) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.name == name)
    }
}

/// Position on the evidence ladder of `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemoryEvidenceBasis {
    /// Explicit region configuration plus ELF program/load evidence.
    RegionConfigAndElfLoad,
    /// Region evidence from a supported MAP adapter plus ELF load evidence.
    MapRegionAndElfLoad,
    /// Deterministic mapping from ELF section/segment address plus flags.
    ElfAddressAndFlags,
    /// Section-name inference only.
    SectionNameHeuristic,
    /// Nothing sufficient to attribute this section.
    Insufficient,
}

impl MemoryEvidenceBasis {
    /// Stronger evidence has the lower rank.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::RegionConfigAndElfLoad => 0,
            Self::MapRegionAndElfLoad => 1,
            Self::ElfAddressAndFlags => 2,
            Self::SectionNameHeuristic => 3,
            Self::Insufficient => 4,
        }
    }

    /// Only the top two ladder rungs count as an observation. Flags-only mapping is reported
    /// as observed too, but `may_support_hard_block` is what actually gates a verdict.
    #[must_use]
    pub const fn classification(self) -> EvidenceClass {
        match self {
            Self::RegionConfigAndElfLoad | Self::MapRegionAndElfLoad | Self::ElfAddressAndFlags => {
                EvidenceClass::Observed
            }
            Self::SectionNameHeuristic => EvidenceClass::Derived,
            Self::Insufficient => EvidenceClass::Unknown,
        }
    }

    /// A name heuristic or a flags-only mapping may inform a review, but must never be the
    /// foundation of a hard memory BLOCK (ladder levels 3 and 4).
    #[must_use]
    pub const fn may_support_hard_block(self) -> bool {
        matches!(
            self,
            Self::RegionConfigAndElfLoad | Self::MapRegionAndElfLoad
        )
    }
}

/// How much of a budget is actually accounted for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ByteTotal {
    /// Every allocatable section was attributed with a definite number.
    Exact {
        bytes: u64,
    },
    /// A floor: some sections could not be attributed, listed in `unattributed`.
    Partial {
        bytes: u64,
        unattributed: Vec<String>,
        reason: String,
    },
    Unknown {
        reason: String,
    },
}

impl ByteTotal {
    #[must_use]
    pub fn bytes(&self) -> Option<u64> {
        match self {
            Self::Exact { bytes } | Self::Partial { bytes, .. } => Some(*bytes),
            Self::Unknown { .. } => None,
        }
    }

    #[must_use]
    pub fn is_exact(&self) -> bool {
        matches!(self, Self::Exact { .. })
    }

    #[must_use]
    pub fn classification(&self) -> EvidenceClass {
        match self {
            Self::Exact { .. } => EvidenceClass::Observed,
            Self::Partial { .. } => EvidenceClass::Derived,
            Self::Unknown { .. } => EvidenceClass::Unknown,
        }
    }
}

/// One section's charge against each budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryContribution {
    pub section_index: usize,
    /// Deterministic locator so a reader can re-check the claim in the ELF.
    pub section_locator: String,
    pub section_name: Fact<String>,
    pub nonvolatile_bytes: Fact<u64>,
    pub runtime_ram_bytes: Fact<u64>,
    pub basis: MemoryEvidenceBasis,
    pub rule: &'static str,
    /// The section's runtime address and size, when known. Object attribution uses them to tell apart
    /// two output sections that share a name (armlink's RW data and ZI parts of one region, ADR-0036).
    pub runtime_span: Option<(u64, u64)>,
}

impl MemoryContribution {
    /// The property the whole P0 memory check exists to prove: one section can be charged
    /// to both budgets simultaneously.
    #[must_use]
    pub fn is_dual_accounted(&self) -> bool {
        self.nonvolatile_bytes.value().copied().unwrap_or(0) > 0
            && self.runtime_ram_bytes.value().copied().unwrap_or(0) > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionUsageState {
    Exact,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionUsageRisk {
    WithinCapacity,
    Exceeded,
    Review,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionAddressKind {
    Load,
    Virtual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionSectionUsage {
    pub section_locator: String,
    pub section_name: Fact<String>,
    pub address_kind: RegionAddressKind,
    pub address: u64,
    pub bytes: u64,
    pub virtual_address: Fact<u64>,
    pub load_address: Fact<u64>,
    pub evidence_basis: MemoryEvidenceBasis,
}

/// Region occupancy covers observed allocated section intervals; it does not claim allocator space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRegionUsage {
    pub name: String,
    pub kind: RegionKind,
    pub origin: u64,
    pub capacity_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub remaining_bytes: Option<u64>,
    pub remaining_reason: &'static str,
    pub state: RegionUsageState,
    pub risk: RegionUsageRisk,
    pub evidence_source: &'static str,
    pub evidence_id: String,
    pub address_kind: RegionAddressKind,
    pub sections: Vec<RegionSectionUsage>,
    pub reasons: Vec<String>,
}

/// The two separate budgets of ADR-0021.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryFootprint {
    pub nonvolatile: ByteTotal,
    pub runtime_ram: ByteTotal,
    pub contributions: Vec<MemoryContribution>,
    pub excluded_metadata_bytes: u64,
    /// Weakest basis among all counted contributions; caps the whole report.
    pub weakest_basis: Option<MemoryEvidenceBasis>,
    pub layout_source: LayoutSource,
    pub region_usage: Vec<MemoryRegionUsage>,
}

impl MemoryFootprint {
    /// Charge every allocatable section against both budgets using real load evidence.
    #[must_use]
    pub fn compute(sections: &[Section], layout: &MemoryLayout) -> Self {
        let mut contributions = Vec::new();
        let mut excluded_metadata_bytes = 0_u64;
        let mut weakest: Option<MemoryEvidenceBasis> = None;

        for section in sections {
            if section.role.is_device_metadata() {
                excluded_metadata_bytes = excluded_metadata_bytes.saturating_add(section.file_size);
                continue;
            }
            if !section.flags.alloc {
                continue;
            }

            let contribution = attribute(section, layout);
            weakest = Some(match weakest {
                Some(current) if current.rank() > contribution.basis.rank() => current,
                _ => contribution.basis,
            });
            contributions.push(contribution);
        }

        let region_usage = calculate_region_usage(sections, &contributions, layout);

        Self {
            nonvolatile: total(&contributions, ChargeSide::Nonvolatile),
            runtime_ram: total(&contributions, ChargeSide::Runtime),
            contributions,
            excluded_metadata_bytes,
            weakest_basis: weakest,
            layout_source: layout.source,
            region_usage,
        }
    }

    #[must_use]
    pub fn dual_accounted_sections(&self) -> Vec<&MemoryContribution> {
        self.contributions
            .iter()
            .filter(|c| c.is_dual_accounted())
            .collect()
    }

    /// Whether a hard memory verdict is admissible for this footprint.
    #[must_use]
    pub fn admissible_for_hard_block(&self) -> bool {
        self.weakest_basis
            .is_some_and(MemoryEvidenceBasis::may_support_hard_block)
            && self.nonvolatile.is_exact()
            && self.runtime_ram.is_exact()
    }
}

const REMAINING_UNKNOWN_REASON: &str = "the region table and section intervals do not prove contiguous allocatable space; gaps may contain padding or reserved memory";

fn calculate_region_usage(
    sections: &[Section],
    contributions: &[MemoryContribution],
    layout: &MemoryLayout,
) -> Vec<MemoryRegionUsage> {
    let contributions_by_index: HashMap<usize, &MemoryContribution> = contributions
        .iter()
        .map(|contribution| (contribution.section_index, contribution))
        .collect();
    layout
        .regions
        .iter()
        .filter(|region| region.name != "*default*")
        .map(|region| {
            let address_kind = match region.kind {
                RegionKind::Nonvolatile => RegionAddressKind::Load,
                RegionKind::VolatileRam | RegionKind::Unknown => RegionAddressKind::Virtual,
            };
            let capacity_bytes =
                (layout.source != LayoutSource::None && region.length > 0).then_some(region.length);
            let mut usage = MemoryRegionUsage {
                name: region.name.clone(),
                kind: region.kind,
                origin: region.origin,
                capacity_bytes,
                used_bytes: None,
                remaining_bytes: None,
                remaining_reason: REMAINING_UNKNOWN_REASON,
                state: RegionUsageState::Unknown,
                risk: RegionUsageRisk::Unknown,
                evidence_source: layout.source.as_label(),
                evidence_id: region.evidence_id.clone(),
                address_kind,
                sections: Vec::new(),
                reasons: Vec::new(),
            };

            let Some(region_end) = (region.length > 0)
                .then(|| region.origin.checked_add(region.length))
                .flatten()
            else {
                usage.reasons.push(
                    "region origin plus capacity overflows the supported address range".to_owned(),
                );
                return usage;
            };
            if region.kind == RegionKind::Unknown {
                usage
                    .reasons
                    .push("region attributes do not identify Flash or runtime RAM".to_owned());
                return usage;
            }

            let mut ranges = Vec::new();
            let mut unresolved = false;
            let mut exceeded = false;

            for section in sections
                .iter()
                .filter(|section| section.role.admitted_to_budget(section.flags.alloc))
            {
                let Some(contribution) = contributions_by_index.get(&section.index).copied() else {
                    continue;
                };
                let charge = match region.kind {
                    RegionKind::Nonvolatile => &contribution.nonvolatile_bytes,
                    RegionKind::VolatileRam => &contribution.runtime_ram_bytes,
                    RegionKind::Unknown => unreachable!(),
                };
                let address_fact = match region.kind {
                    RegionKind::Nonvolatile => &section.load_address,
                    RegionKind::VolatileRam => &section.virtual_address,
                    RegionKind::Unknown => unreachable!(),
                };
                let address = address_fact.value().copied().or_else(|| {
                    (region.kind == RegionKind::Nonvolatile)
                        .then(|| section.virtual_address.value().copied())
                        .flatten()
                        .filter(|address| {
                            region.contains(*address) && section.load_address.value().is_none()
                        })
                });

                match charge {
                    Fact::Known(0) => continue,
                    Fact::Known(bytes) => {
                        let Some(address) = address else {
                            unresolved = true;
                            usage.reasons.push(format!(
                                "{} has a known charge but no {} address",
                                contribution.section_locator,
                                match address_kind {
                                    RegionAddressKind::Load => "load",
                                    RegionAddressKind::Virtual => "runtime",
                                }
                            ));
                            continue;
                        };
                        if address >= region_end || address.saturating_add(*bytes) <= region.origin
                        {
                            continue;
                        }
                        let Some(section_end) = address.checked_add(*bytes) else {
                            unresolved = true;
                            usage.reasons.push(format!(
                                "{} address plus size overflows the supported address range",
                                contribution.section_locator
                            ));
                            if address < region_end {
                                ranges.push((address.max(region.origin), region_end));
                            }
                            usage.sections.push(region_section_usage(
                                section,
                                contribution,
                                address_kind,
                                address,
                                *bytes,
                            ));
                            continue;
                        };
                        let intersects = address < region_end && section_end > region.origin;
                        if !intersects {
                            continue;
                        }
                        if address < region.origin || section_end > region_end {
                            unresolved = true;
                            if address >= region.origin && address < region_end {
                                exceeded = true;
                            }
                            usage.reasons.push(format!(
                                "{} range crosses the declared region boundary",
                                contribution.section_locator
                            ));
                        }
                        let start = address.max(region.origin);
                        let end = section_end.min(region_end);
                        ranges.push((start, end));
                        usage.sections.push(region_section_usage(
                            section,
                            contribution,
                            address_kind,
                            address,
                            *bytes,
                        ));
                    }
                    Fact::Unknown { .. } => {
                        let could_be_in_region = address
                            .is_none_or(|address| address < region_end && address >= region.origin);
                        if could_be_in_region {
                            unresolved = true;
                            usage.reasons.push(format!(
                                "{} charge or placement is unknown",
                                contribution.section_locator
                            ));
                        }
                    }
                }
            }

            ranges.sort_unstable();
            let mut merged: Vec<(u64, u64)> = Vec::new();
            let mut overlap = false;
            for (start, end) in ranges {
                if let Some((_, previous_end)) = merged.last_mut() {
                    if start < *previous_end {
                        overlap = true;
                        *previous_end = (*previous_end).max(end);
                        continue;
                    }
                    if start == *previous_end {
                        *previous_end = end;
                        continue;
                    }
                }
                merged.push((start, end));
            }
            if overlap {
                unresolved = true;
                usage
                    .reasons
                    .push("allocated section ranges overlap; the union is shown once".to_owned());
            }

            let mapped_bytes = merged.iter().fold(0_u64, |sum, (start, end)| {
                sum.saturating_add(end.saturating_sub(*start))
            });
            usage.used_bytes = if unresolved && merged.is_empty() {
                None
            } else {
                Some(mapped_bytes)
            };
            usage.state = if unresolved {
                if usage.used_bytes.is_some() {
                    RegionUsageState::Partial
                } else {
                    RegionUsageState::Unknown
                }
            } else {
                RegionUsageState::Exact
            };
            usage.risk = if exceeded {
                RegionUsageRisk::Exceeded
            } else if unresolved {
                RegionUsageRisk::Review
            } else {
                RegionUsageRisk::WithinCapacity
            };
            usage
        })
        .collect()
}

fn region_section_usage(
    section: &Section,
    contribution: &MemoryContribution,
    address_kind: RegionAddressKind,
    address: u64,
    bytes: u64,
) -> RegionSectionUsage {
    RegionSectionUsage {
        section_locator: contribution.section_locator.clone(),
        section_name: contribution.section_name.clone(),
        address_kind,
        address,
        bytes,
        virtual_address: section.virtual_address.clone(),
        load_address: section.load_address.clone(),
        evidence_basis: contribution.basis,
    }
}

#[derive(Debug, Clone, Copy)]
enum ChargeSide {
    Nonvolatile,
    Runtime,
}

fn total(contributions: &[MemoryContribution], side: ChargeSide) -> ByteTotal {
    let mut sum = 0_u64;
    let mut unattributed = Vec::new();
    let mut reasons = Vec::new();

    for c in contributions {
        let value = match side {
            ChargeSide::Nonvolatile => &c.nonvolatile_bytes,
            ChargeSide::Runtime => &c.runtime_ram_bytes,
        };
        match value {
            Fact::Known(bytes) => sum = sum.saturating_add(*bytes),
            Fact::Unknown { reason } => {
                unattributed.push(c.section_locator.clone());
                reasons.push(format!("{}: {reason}", c.section_locator));
            }
        }
    }

    if reasons.is_empty() {
        return ByteTotal::Exact { bytes: sum };
    }

    ByteTotal::Partial {
        bytes: sum,
        unattributed,
        reason: reasons.join("; "),
    }
}

/// Resolve one section's charge, choosing the strongest basis the evidence supports.
fn runtime_span(section: &Section) -> Option<(u64, u64)> {
    section
        .virtual_address
        .value()
        .copied()
        .zip(section.memory_size.value().copied())
}

fn attribute(section: &Section, layout: &MemoryLayout) -> MemoryContribution {
    let locator = format!("elf.section_header[{}]", section.index);

    let declared_region = section
        .region
        .value()
        .and_then(|region_name| layout.region_named(region_name));
    let vma_region = section
        .virtual_address
        .value()
        .and_then(|addr| layout.region_containing(*addr));
    let lma_region = section
        .load_address
        .value()
        .and_then(|addr| layout.region_containing(*addr));

    let region_backed = declared_region.is_some() || vma_region.is_some() || lma_region.is_some();

    if region_backed {
        let (basis, rule) = match layout.source {
            LayoutSource::ProjectConfig => (
                MemoryEvidenceBasis::RegionConfigAndElfLoad,
                "region-config+elf-load",
            ),
            LayoutSource::MapMemoryConfiguration => (
                MemoryEvidenceBasis::MapRegionAndElfLoad,
                "map-memory-configuration+elf-load",
            ),
            // Addresses cannot resolve into regions when there is no region table, so this
            // arm is unreachable; keep it total rather than unwrapping.
            LayoutSource::None => (
                MemoryEvidenceBasis::ElfAddressAndFlags,
                "elf-address-and-flags",
            ),
        };

        return MemoryContribution {
            section_index: section.index,
            runtime_span: runtime_span(section),
            section_locator: locator,
            section_name: section.name.clone(),
            nonvolatile_bytes: nonvolatile_from_regions(
                section,
                vma_region,
                lma_region,
                declared_region,
            ),
            runtime_ram_bytes: runtime_from_regions(section, vma_region, declared_region),
            basis,
            rule,
        };
    }

    // No region table. A role that came from section flags is still a deterministic mapping
    // (ladder level 3); a role we could not decide is not something to put a number on.
    if section.role != SectionRole::Unknown {
        return MemoryContribution {
            section_index: section.index,
            runtime_span: runtime_span(section),
            section_locator: locator,
            section_name: section.name.clone(),
            nonvolatile_bytes: role_charge(section, ChargeSide::Nonvolatile),
            runtime_ram_bytes: role_charge(section, ChargeSide::Runtime),
            basis: MemoryEvidenceBasis::ElfAddressAndFlags,
            rule: "elf-address-and-flags",
        };
    }

    let (basis, rule) = if section.name.is_known() {
        (
            MemoryEvidenceBasis::SectionNameHeuristic,
            "section-name-heuristic",
        )
    } else {
        (MemoryEvidenceBasis::Insufficient, "unattributed")
    };

    MemoryContribution {
        section_index: section.index,
        runtime_span: runtime_span(section),
        section_locator: locator,
        section_name: section.name.clone(),
        nonvolatile_bytes: Fact::unknown(
            "no memory-region evidence and the section role could not be classified",
        ),
        runtime_ram_bytes: Fact::unknown(
            "no memory-region evidence and the section role could not be classified",
        ),
        basis,
        rule,
    }
}

fn role_charge(section: &Section, side: ChargeSide) -> Fact<u64> {
    match side {
        ChargeSide::Nonvolatile => {
            if section.role.contributes_nonvolatile_by_default() {
                Fact::known(section.file_size)
            } else {
                Fact::known(0)
            }
        }
        ChargeSide::Runtime => {
            if section.role.contributes_runtime_ram_by_default() {
                match &section.memory_size {
                    Fact::Known(bytes) => Fact::Known(*bytes),
                    Fact::Unknown { reason } => Fact::unknown(reason.clone()),
                }
            } else {
                Fact::known(0)
            }
        }
    }
}

fn runtime_from_regions(
    section: &Section,
    vma_region: Option<&MemoryRegion>,
    declared_region: Option<&MemoryRegion>,
) -> Fact<u64> {
    match declared_region.or(vma_region).map(|r| r.kind) {
        Some(RegionKind::VolatileRam) => match &section.memory_size {
            Fact::Known(bytes) => Fact::Known(*bytes),
            Fact::Unknown { reason } => Fact::unknown(reason.clone()),
        },
        Some(RegionKind::Nonvolatile) => Fact::known(0),
        Some(RegionKind::Unknown) | None => Fact::unknown(
            "the section runtime address did not fall inside any declared memory region",
        ),
    }
}

fn nonvolatile_from_regions(
    section: &Section,
    vma_region: Option<&MemoryRegion>,
    lma_region: Option<&MemoryRegion>,
    declared_region: Option<&MemoryRegion>,
) -> Fact<u64> {
    // A NOBITS section carries no image payload whatever the region says.
    if section.file_size == 0 {
        return Fact::known(0);
    }

    // Image backing is decided by where the bytes are *loaded from*, never by where they run.
    // The declared region names the runtime location, so it may stand in for storage only when
    // it is itself non-writable, which by definition cannot hold live variables.
    let storage = lma_region.or_else(|| {
        declared_region
            .into_iter()
            .chain(vma_region)
            .find(|r| r.kind == RegionKind::Nonvolatile)
    });

    match storage.map(|r| r.kind) {
        Some(RegionKind::Nonvolatile) => Fact::known(section.file_size),
        Some(RegionKind::VolatileRam) => Fact::unknown(
            "the only available region evidence points at writable storage, so no nonvolatile image backing is evidenced",
        ),
        Some(RegionKind::Unknown) | None => {
            Fact::unknown("the section load address did not fall inside any declared memory region")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::section::SectionFlags;

    fn rom() -> MemoryRegion {
        MemoryRegion::new("ROM", 0x0800_0000, 0x0010_0000, "xr", "ev-rom")
    }

    fn ram() -> MemoryRegion {
        MemoryRegion::new("RAM", 0x2000_0000, 0x0004_0000, "xrw", "ev-ram")
    }

    fn layout() -> MemoryLayout {
        MemoryLayout {
            regions: vec![rom(), ram()],
            source: LayoutSource::MapMemoryConfiguration,
        }
    }

    // Test fixture builder: the eight parameters each mirror one independent ELF field, and
    // collapsing them would hide which dimension a given case is varying.
    #[expect(
        clippy::too_many_arguments,
        reason = "section fixture builder varying one ELF field per parameter"
    )]
    fn make(
        index: usize,
        name: &str,
        role: SectionRole,
        vma: Fact<u64>,
        lma: Fact<u64>,
        file_size: u64,
        mem_size: Fact<u64>,
        alloc: bool,
    ) -> Section {
        Section {
            index,
            name: if name.is_empty() {
                Fact::unknown("stripped section name")
            } else {
                Fact::known(name.to_owned())
            },
            role,
            flags: SectionFlags {
                alloc,
                ..SectionFlags::default()
            },
            virtual_address: vma,
            load_address: lma,
            file_offset: Fact::known(0x100),
            file_size,
            memory_size: mem_size,
            region: Fact::unknown("not stated by the section header"),
        }
    }

    #[test]
    fn region_attributes_separate_rom_from_ram() {
        assert_eq!(
            RegionAttributes::parse("xr").kind(),
            RegionKind::Nonvolatile
        );
        // RAM is declared `xrw`; the execute bit must not make it look like flash.
        assert_eq!(
            RegionAttributes::parse("xrw").kind(),
            RegionKind::VolatileRam
        );
        assert_eq!(
            RegionAttributes::parse("rwx").kind(),
            RegionKind::VolatileRam
        );
        assert_eq!(RegionAttributes::parse("").kind(), RegionKind::Unknown);
    }

    #[test]
    fn region_contains_respects_bounds_and_zero_length() {
        let rom_region = rom();
        assert!(rom_region.contains(0x0800_0000));
        assert!(rom_region.contains(0x080F_FFFF));
        assert!(!rom_region.contains(0x0810_0000));
        assert!(!rom_region.contains(0x2000_0000));
        assert!(!MemoryRegion::new("Z", 0, 0, "rw", "ev").contains(0));
    }

    #[test]
    fn initialized_data_is_charged_to_both_budgets_from_load_evidence() {
        // The exact shape of the P0 fixture: VMA in RAM, LMA in ROM.
        let data = make(
            3,
            ".data",
            SectionRole::InitializedData,
            Fact::known(0x2000_0000),
            Fact::known(0x0800_0060),
            4,
            Fact::known(4),
            true,
        );

        let footprint = MemoryFootprint::compute(&[data], &layout());
        let c = &footprint.contributions[0];

        assert_eq!(c.nonvolatile_bytes.value(), Some(&4));
        assert_eq!(c.runtime_ram_bytes.value(), Some(&4));
        assert!(c.is_dual_accounted());
        assert_eq!(c.basis, MemoryEvidenceBasis::MapRegionAndElfLoad);
        assert_eq!(c.rule, "map-memory-configuration+elf-load");
        assert_eq!(footprint.nonvolatile, ByteTotal::Exact { bytes: 4 });
        assert_eq!(footprint.runtime_ram, ByteTotal::Exact { bytes: 4 });
        assert!(footprint.admissible_for_hard_block());
    }

    #[test]
    fn region_usage_keeps_load_image_and_runtime_ranges_separate() {
        let sections = [
            make(
                1,
                ".data",
                SectionRole::InitializedData,
                Fact::known(0x2000_0000),
                Fact::known(0x0800_0060),
                4,
                Fact::known(4),
                true,
            ),
            make(
                2,
                ".bss",
                SectionRole::UninitializedData,
                Fact::known(0x2000_0004),
                Fact::unknown("NOBITS"),
                0,
                Fact::known(4),
                true,
            ),
        ];

        let footprint = MemoryFootprint::compute(&sections, &layout());
        let flash = footprint
            .region_usage
            .iter()
            .find(|region| region.name == "ROM")
            .expect("ROM usage");
        let ram = footprint
            .region_usage
            .iter()
            .find(|region| region.name == "RAM")
            .expect("RAM usage");

        assert_eq!(flash.capacity_bytes, Some(0x0010_0000));
        assert_eq!(flash.used_bytes, Some(4));
        assert_eq!(flash.state, RegionUsageState::Exact);
        assert!(flash.sections.iter().any(|section| {
            section
                .section_name
                .value()
                .is_some_and(|name| name == ".data")
                && section.address_kind == RegionAddressKind::Load
                && section.address == 0x0800_0060
        }));
        assert_eq!(ram.capacity_bytes, Some(0x0004_0000));
        assert_eq!(ram.used_bytes, Some(8));
        assert_eq!(ram.state, RegionUsageState::Exact);
        assert!(ram.sections.iter().any(|section| {
            section
                .section_name
                .value()
                .is_some_and(|name| name == ".bss")
                && section.address_kind == RegionAddressKind::Virtual
                && section.address == 0x2000_0004
        }));
        assert_eq!(flash.remaining_bytes, None);
        assert_eq!(ram.remaining_bytes, None);
    }

    #[test]
    fn overlapping_sections_are_unioned_and_remaining_stays_unknown() {
        let sections = [
            make(
                1,
                ".one",
                SectionRole::UninitializedData,
                Fact::known(0x2000_0000),
                Fact::unknown("NOBITS"),
                0,
                Fact::known(8),
                true,
            ),
            make(
                2,
                ".two",
                SectionRole::UninitializedData,
                Fact::known(0x2000_0004),
                Fact::unknown("NOBITS"),
                0,
                Fact::known(8),
                true,
            ),
        ];

        let footprint = MemoryFootprint::compute(&sections, &layout());
        let ram = footprint
            .region_usage
            .iter()
            .find(|region| region.name == "RAM")
            .expect("RAM usage");

        assert_eq!(ram.used_bytes, Some(12), "overlapping bytes count once");
        assert_eq!(ram.state, RegionUsageState::Partial);
        assert_eq!(ram.risk, RegionUsageRisk::Review);
        assert!(ram.reasons.iter().any(|reason| reason.contains("overlap")));
        assert_eq!(ram.remaining_bytes, None);
    }

    #[test]
    fn section_crossing_region_end_is_flagged_without_claiming_free_space() {
        let mut small = MemoryLayout {
            regions: vec![MemoryRegion::new(
                "SMALL",
                0x2000_0000,
                8,
                "rw",
                "map.line4",
            )],
            source: LayoutSource::MapMemoryConfiguration,
        };
        small.regions[0].kind = RegionKind::VolatileRam;
        let section = make(
            1,
            ".overflow",
            SectionRole::UninitializedData,
            Fact::known(0x2000_0004),
            Fact::unknown("NOBITS"),
            0,
            Fact::known(8),
            true,
        );

        let footprint = MemoryFootprint::compute(&[section], &small);
        let usage = &footprint.region_usage[0];

        assert_eq!(usage.used_bytes, Some(4));
        assert_eq!(usage.risk, RegionUsageRisk::Exceeded);
        assert_eq!(usage.state, RegionUsageState::Partial);
        assert!(
            usage
                .reasons
                .iter()
                .any(|reason| reason.contains("boundary"))
        );
        assert_eq!(usage.remaining_bytes, None);
    }

    #[test]
    fn address_overflow_and_unknown_region_evidence_degrade_to_unknown() {
        let overflow_layout = MemoryLayout {
            regions: vec![MemoryRegion::new("TOP", u64::MAX - 7, 7, "rw", "map.line4")],
            source: LayoutSource::MapMemoryConfiguration,
        };
        let section = make(
            1,
            ".wrap",
            SectionRole::UninitializedData,
            Fact::known(u64::MAX - 3),
            Fact::unknown("NOBITS"),
            0,
            Fact::known(8),
            true,
        );

        let overflow = MemoryFootprint::compute(&[section], &overflow_layout);
        assert_eq!(overflow.region_usage[0].state, RegionUsageState::Partial);
        assert_eq!(overflow.region_usage[0].risk, RegionUsageRisk::Review);
        assert!(
            overflow.region_usage[0]
                .reasons
                .iter()
                .any(|reason| reason.contains("overflow"))
        );

        let unknown_layout = MemoryLayout {
            regions: vec![MemoryRegion::new("MYSTERY", 0x1000, 0x100, "", "map.line4")],
            source: LayoutSource::MapMemoryConfiguration,
        };
        let unknown = MemoryFootprint::compute(&[], &unknown_layout);
        assert_eq!(unknown.region_usage[0].state, RegionUsageState::Unknown);
        assert_eq!(unknown.region_usage[0].risk, RegionUsageRisk::Unknown);
        assert_eq!(unknown.region_usage[0].used_bytes, None);
    }

    #[test]
    fn project_config_regions_outrank_map_regions() {
        let mut map_layout = layout();
        map_layout.source = LayoutSource::ProjectConfig;
        let data = make(
            3,
            ".data",
            SectionRole::InitializedData,
            Fact::known(0x2000_0000),
            Fact::known(0x0800_0060),
            4,
            Fact::known(4),
            true,
        );

        let footprint = MemoryFootprint::compute(&[data], &map_layout);

        assert_eq!(
            footprint.contributions[0].basis,
            MemoryEvidenceBasis::RegionConfigAndElfLoad
        );
    }

    #[test]
    fn bss_costs_ram_but_no_image_bytes() {
        let bss = make(
            4,
            ".bss",
            SectionRole::UninitializedData,
            Fact::known(0x2000_0000),
            Fact::unknown("NOBITS sections have no load address"),
            0,
            Fact::known(1024),
            true,
        );

        let footprint = MemoryFootprint::compute(&[bss], &layout());
        let c = &footprint.contributions[0];

        assert_eq!(c.nonvolatile_bytes.value(), Some(&0));
        assert_eq!(c.runtime_ram_bytes.value(), Some(&1024));
        assert!(!c.is_dual_accounted());
    }

    #[test]
    fn debug_sections_are_excluded_from_both_budgets() {
        let debug = make(
            9,
            ".debug_info",
            SectionRole::Debug,
            Fact::known(0),
            Fact::known(0x0800_9000),
            4096,
            Fact::known(0),
            false,
        );

        let footprint = MemoryFootprint::compute(&[debug], &layout());

        assert!(footprint.contributions.is_empty());
        assert_eq!(footprint.excluded_metadata_bytes, 4096);
        assert_eq!(footprint.nonvolatile, ByteTotal::Exact { bytes: 0 });
        assert_eq!(footprint.runtime_ram, ByteTotal::Exact { bytes: 0 });
    }

    #[test]
    fn custom_section_is_attributed_from_evidence_not_its_name() {
        // Named `.ota_staging`, resident in RAM, but loaded from ROM: the real dual case,
        // decided with an unclassifiable role so only evidence can carry it.
        let ota = make(
            5,
            ".ota_staging",
            SectionRole::Unknown,
            Fact::known(0x2000_0004),
            Fact::known(0x0800_0060),
            64,
            Fact::known(64),
            true,
        );

        let footprint = MemoryFootprint::compute(&[ota], &layout());
        let c = &footprint.contributions[0];

        assert_eq!(c.nonvolatile_bytes.value(), Some(&64));
        assert_eq!(c.runtime_ram_bytes.value(), Some(&64));
        assert_eq!(c.basis, MemoryEvidenceBasis::MapRegionAndElfLoad);
        assert_eq!(footprint.dual_accounted_sections().len(), 1);
    }

    #[test]
    fn declared_region_name_is_stronger_than_address_arithmetic() {
        let mut named = make(
            5,
            ".scratch",
            SectionRole::Unknown,
            Fact::known(0x2000_0004),
            Fact::unknown("not expressible in the section header"),
            16,
            Fact::known(16),
            true,
        );
        named.region = Fact::known("ROM".to_owned());

        let footprint = MemoryFootprint::compute(&[named], &layout());
        let c = &footprint.contributions[0];

        // The named ROM region wins: image bytes, no RAM charge.
        assert_eq!(c.nonvolatile_bytes.value(), Some(&16));
        assert_eq!(c.runtime_ram_bytes.value(), Some(&0));
    }

    #[test]
    fn flags_only_mapping_cannot_support_a_hard_block() {
        // No region table at all: level 3 evidence.
        let text = make(
            1,
            ".text",
            SectionRole::Code,
            Fact::known(0x0800_0000),
            Fact::known(0x0800_0000),
            60,
            Fact::known(60),
            true,
        );

        let footprint = MemoryFootprint::compute(&[text], &MemoryLayout::empty());
        let c = &footprint.contributions[0];

        assert_eq!(c.basis, MemoryEvidenceBasis::ElfAddressAndFlags);
        assert_eq!(c.nonvolatile_bytes.value(), Some(&60));
        assert_eq!(c.runtime_ram_bytes.value(), Some(&0));
        assert!(!c.basis.may_support_hard_block());
        assert!(!footprint.admissible_for_hard_block());
    }

    #[test]
    fn name_heuristic_alone_is_derived_and_offers_no_number() {
        // Unclassifiable role and no region evidence: the name is all there is, so the model
        // refuses to commit a budget rather than upgrade a guess into an observation.
        let mystery = make(
            6,
            ".guessme",
            SectionRole::Unknown,
            Fact::known(0x9999_0000),
            Fact::known(0x9999_0000),
            32,
            Fact::known(32),
            true,
        );

        let footprint = MemoryFootprint::compute(&[mystery], &MemoryLayout::empty());
        let c = &footprint.contributions[0];

        assert_eq!(c.basis, MemoryEvidenceBasis::SectionNameHeuristic);
        assert_eq!(c.basis.classification(), EvidenceClass::Derived);
        assert_eq!(c.nonvolatile_bytes.value(), None);
        assert!(!footprint.nonvolatile.is_exact());
        assert!(!footprint.admissible_for_hard_block());
    }

    #[test]
    fn unattributable_section_degrades_the_total_instead_of_guessing() {
        let mystery = make(
            7,
            "",
            SectionRole::Unknown,
            Fact::known(0xDEAD_BEEF),
            Fact::known(0xDEAD_BEEF),
            32,
            Fact::known(32),
            true,
        );

        let footprint = MemoryFootprint::compute(&[mystery], &layout());

        assert_eq!(
            footprint.contributions[0].basis,
            MemoryEvidenceBasis::Insufficient
        );
        match &footprint.nonvolatile {
            ByteTotal::Partial {
                bytes,
                unattributed,
                ..
            } => {
                assert_eq!(*bytes, 0, "an unattributed section must not be counted");
                assert_eq!(unattributed, &vec!["elf.section_header[7]".to_owned()]);
            }
            other => panic!("expected a degraded total, got {other:?}"),
        }
        assert_eq!(
            footprint.runtime_ram.classification(),
            EvidenceClass::Derived
        );
    }

    #[test]
    fn address_outside_every_region_never_claims_region_evidence() {
        let stray = make(
            8,
            ".stray",
            SectionRole::InitializedData,
            Fact::known(0xdead_beef),
            Fact::known(0xdead_beef),
            8,
            Fact::known(8),
            true,
        );

        let footprint = MemoryFootprint::compute(&[stray], &layout());
        let c = &footprint.contributions[0];

        // The region table was available but matched nothing, so this must fall back to the
        // weaker flags mapping rather than report region-backed attribution.
        assert_ne!(c.basis, MemoryEvidenceBasis::MapRegionAndElfLoad);
        assert_eq!(c.basis, MemoryEvidenceBasis::ElfAddressAndFlags);
        assert_eq!(
            footprint.weakest_basis,
            Some(MemoryEvidenceBasis::ElfAddressAndFlags)
        );
        assert!(
            !footprint.admissible_for_hard_block(),
            "unmatched addresses must not support a hard memory verdict"
        );
    }

    #[test]
    fn non_allocatable_sections_are_not_charged() {
        let note = make(
            2,
            ".note.lto",
            SectionRole::Unknown,
            Fact::known(0),
            Fact::known(0x0800_0000),
            128,
            Fact::known(128),
            false,
        );

        let footprint = MemoryFootprint::compute(&[note], &layout());

        assert!(footprint.contributions.is_empty());
        assert_eq!(footprint.nonvolatile, ByteTotal::Exact { bytes: 0 });
    }

    #[test]
    fn mixed_fixture_totals_are_computed_from_every_allocatable_section() {
        let sections = vec![
            make(
                1,
                ".text",
                SectionRole::Code,
                Fact::known(0x0800_0000),
                Fact::known(0x0800_0000),
                60,
                Fact::known(60),
                true,
            ),
            make(
                2,
                ".rodata",
                SectionRole::ReadOnlyData,
                Fact::known(0x0800_003C),
                Fact::known(0x0800_003C),
                32,
                Fact::known(32),
                true,
            ),
            make(
                3,
                ".data",
                SectionRole::InitializedData,
                Fact::known(0x2000_0000),
                Fact::known(0x0800_005C),
                4,
                Fact::known(4),
                true,
            ),
            make(
                4,
                ".bss",
                SectionRole::UninitializedData,
                Fact::known(0x2000_0004),
                Fact::unknown("NOBITS"),
                0,
                Fact::known(4),
                true,
            ),
        ];

        let footprint = MemoryFootprint::compute(&sections, &layout());

        // Image holds text + rodata + the .data initializer; RAM holds .data + .bss.
        assert_eq!(footprint.nonvolatile, ByteTotal::Exact { bytes: 96 });
        assert_eq!(footprint.runtime_ram, ByteTotal::Exact { bytes: 8 });
        assert_eq!(footprint.dual_accounted_sections().len(), 1);
        assert!(footprint.admissible_for_hard_block());
    }
}
