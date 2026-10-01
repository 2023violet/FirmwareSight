//! The portable `analysis:1` document — a Release Bundle's record of one build.
//!
//! This is deliberately *not* [`crate::AnalyzeResultDto`] copied into a file. That shape is marked
//! `p0-internal`: it travels between the Rust service and the desktop front end, both of which ship
//! together, and `04_TECH/14_IPC_DATA_CONTRACTS.md` §4 says plainly that IPC needs no public semantic
//! versioning in the MVP. A bundle is the opposite case. It outlives the build that wrote it, it is read by
//! somebody who does not have this repository, and `04_TECH/26_PORTABLE_SCHEMA_POLICY.md` makes every field
//! of a published contract a compatibility promise. Presenting the internal shape as durable would have
//! made an accidental rename a silent break of a document already shipped to a third party.
//!
//! What is reused is the *projection*, not the envelope: the memory block and the evidence rows are the
//! same functions the CLI calls, so a footprint cannot be reported two ways depending on which file a
//! reader opened. What is added is the full section and symbol tables, which the internal summary does not
//! carry because IPC payloads are capped and a file is not.
//!
//! Rules this document keeps:
//! - **file names, never paths.** Every artifact is named by the leaf it has inside the bundle, so the
//!   document cross-checks against `SHA256SUMS` and needs no host to be read.
//! - **no clock.** There is no generation timestamp and no `imported_at`. The audit time a release record
//!   carries lives in SQLite and is not portable content (prompt §42).
//! - **absence stays explicit.** Every `Fact` renders as a value plus, when there is no value, the reason
//!   FirmwareSight could not establish one. Nothing unknown is written as zero or as an empty string.
//! - **addresses are never unit-converted.** They are hex, the way the ELF says them.

use serde::Serialize;

use firmwaresight_core::domain::artifact::Artifact;
use firmwaresight_core::domain::build_snapshot::{BuildSnapshot, NORMALIZATION_VERSION};
use firmwaresight_core::domain::identity::{Bitness, Endianness, Fact};
use firmwaresight_core::domain::release::{ReleaseArtifact, sanitize_leaf_name};
use firmwaresight_core::domain::section::{Section, SectionRole};
use firmwaresight_core::domain::symbol::{Symbol, SymbolBinding, SymbolKind, SymbolSectionRef};

use crate::dto::{CapabilitiesDto, EvidenceDto, MemoryDto, evidence_dto, memory_dto};

/// The stable URN of this contract. A breaking semantic change needs a new major version, not a re-edit.
pub const ANALYSIS_SCHEMA_ID: &str = "urn:firmwaresight:schema:analysis:1";

/// The major version this build emits.
pub const ANALYSIS_SCHEMA_VERSION: u32 = 1;

/// One portable analysis document.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisDocumentDto {
    pub schema: &'static str,
    pub schema_version: u32,
    pub snapshot: SnapshotDto,
    pub artifacts: Vec<PortableArtifactDto>,
    pub memory: MemoryDto,
    pub sections: Vec<SectionRowDto>,
    pub symbols: Vec<SymbolRowDto>,
    pub capabilities: CapabilitiesDto,
    pub evidence: Vec<EvidenceDto>,
}

/// What produced the facts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
    pub snapshot_id: String,
    /// The FirmwareSight build that analyzed the artifact. Kept separate from any project release version,
    /// which is a fact about the firmware and appears only in the manifest.
    pub fwsight_version: String,
    pub normalization_version: &'static str,
}

/// One input file of the build, identified by the name it has inside the bundle.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortableArtifactDto {
    pub file_name: String,
    /// The Gate vocabulary: `elf`, `map`, `bin`, `hex`, `unknown`.
    pub kind: &'static str,
    #[serde(with = "self::serde_hex")]
    pub sha256: firmwaresight_core::domain::identity::Sha256,
    pub byte_size: u64,
    pub parser_id: String,
    /// The domain architecture name. Left as a string because the domain carries an open `Other(_)` case,
    /// and a closed enum here would either drop a real value or pretend the set is finished.
    pub architecture: String,
    pub bitness: &'static str,
    pub endianness: &'static str,
    pub entry_point: Option<String>,
    pub entry_point_unknown_reason: Option<String>,
    pub build_id: Option<String>,
    pub build_id_unknown_reason: Option<String>,
}

/// A normalized section row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionRowDto {
    /// The ELF section header index — the stable locator inside one file.
    pub index: usize,
    pub name: Option<String>,
    pub name_unknown_reason: Option<String>,
    pub role: &'static str,
    pub flags: SectionFlagsDto,
    pub virtual_address: Option<String>,
    pub virtual_address_unknown_reason: Option<String>,
    pub load_address: Option<String>,
    pub load_address_unknown_reason: Option<String>,
    pub file_offset: Option<String>,
    pub file_offset_unknown_reason: Option<String>,
    pub file_size: u64,
    pub memory_size: Option<u64>,
    pub memory_size_unknown_reason: Option<String>,
    pub region: Option<String>,
    pub region_unknown_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionFlagsDto {
    pub alloc: bool,
    pub execute: bool,
    pub write: bool,
    pub merge: bool,
    pub strings: bool,
    pub tls: bool,
    pub compressed: bool,
}

/// A normalized symbol row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolRowDto {
    pub name: Option<String>,
    pub name_unknown_reason: Option<String>,
    pub address: Option<String>,
    pub address_unknown_reason: Option<String>,
    pub size: Option<u64>,
    pub size_unknown_reason: Option<String>,
    pub kind: &'static str,
    pub binding: &'static str,
    /// Which section header index the symbol is relative to, or `undefined` / `absolute` / `unknown`.
    pub section: String,
}

impl AnalysisDocumentDto {
    /// Project a sealed snapshot into the portable document.
    ///
    /// `shipped` is the bundle's own artifact list, so a file name in this document is the name the reader
    /// will find in `artifacts/` and in `SHA256SUMS`. Matching is by content digest, not by order: an
    /// analysis and a bundle agree because they hash the same bytes, not because they were built in the
    /// same sequence.
    #[must_use]
    pub fn from_snapshot(snapshot: &BuildSnapshot, shipped: &[ReleaseArtifact]) -> Self {
        let memory = snapshot
            .memory()
            .expect("a sealed FirmwareSight snapshot always carries a memory footprint");

        Self {
            schema: ANALYSIS_SCHEMA_ID,
            schema_version: ANALYSIS_SCHEMA_VERSION,
            snapshot: SnapshotDto {
                snapshot_id: snapshot.id().as_str().to_owned(),
                fwsight_version: snapshot.created_by_fwsight_version().to_owned(),
                normalization_version: NORMALIZATION_VERSION,
            },
            artifacts: snapshot
                .artifacts()
                .iter()
                .map(|artifact| PortableArtifactDto::from_artifact(artifact, shipped))
                .collect(),
            memory: memory_dto(memory),
            sections: snapshot
                .sections()
                .iter()
                .map(SectionRowDto::from)
                .collect(),
            symbols: snapshot.symbols().iter().map(SymbolRowDto::from).collect(),
            capabilities: CapabilitiesDto::from(snapshot.capabilities()),
            evidence: snapshot.evidence().iter().map(evidence_dto).collect(),
        }
    }
}

impl PortableArtifactDto {
    #[must_use]
    pub fn from_artifact(artifact: &Artifact, shipped: &[ReleaseArtifact]) -> Self {
        let file_name = shipped
            .iter()
            .find(|row| row.sha256 == artifact.sha256)
            .map(|row| row.file_name.clone())
            .unwrap_or_else(|| sanitize_leaf_name(&artifact.path));

        let (entry_point, entry_reason) = match &artifact.entry_point {
            Fact::Known(value) => (Some(hex_u64(*value)), None),
            Fact::Unknown { reason } => (None, Some(reason.clone())),
        };
        let (build_id, build_reason) = match &artifact.build_id {
            Fact::Known(value) => (Some(value.clone()), None),
            Fact::Unknown { reason } => (None, Some(reason.clone())),
        };

        Self {
            file_name,
            kind: ReleaseArtifact::kind_word(artifact.kind),
            sha256: artifact.sha256.clone(),
            byte_size: artifact.byte_size,
            parser_id: artifact.parser_id.0.clone(),
            architecture: format!("{:?}", artifact.architecture),
            bitness: bitness_text(artifact.bitness),
            endianness: endianness_text(artifact.endianness),
            entry_point,
            entry_point_unknown_reason: entry_reason,
            build_id,
            build_id_unknown_reason: build_reason,
        }
    }
}

impl From<&Section> for SectionRowDto {
    fn from(section: &Section) -> Self {
        let (name, name_reason) = text(&section.name);
        let (virtual_address, virtual_reason) =
            section.virtual_address.value().copied().map_or_else(
                || (None, reason_of(&section.virtual_address)),
                |value| (Some(hex_u64(value)), None),
            );
        let (load_address, load_reason) = section.load_address.value().copied().map_or_else(
            || (None, reason_of(&section.load_address)),
            |value| (Some(hex_u64(value)), None),
        );
        let (file_offset, offset_reason) = section.file_offset.value().copied().map_or_else(
            || (None, reason_of(&section.file_offset)),
            |value| (Some(hex_u64(value)), None),
        );
        let (memory_size, size_reason) = fact_number(&section.memory_size);
        let (region, region_reason) = text(&section.region);

        Self {
            index: section.index,
            name,
            name_unknown_reason: name_reason,
            role: role_text(section.role),
            flags: SectionFlagsDto {
                alloc: section.flags.alloc,
                execute: section.flags.execute,
                write: section.flags.write,
                merge: section.flags.merge,
                strings: section.flags.strings,
                tls: section.flags.tls,
                compressed: section.flags.compressed,
            },
            virtual_address,
            virtual_address_unknown_reason: virtual_reason,
            load_address,
            load_address_unknown_reason: load_reason,
            file_offset,
            file_offset_unknown_reason: offset_reason,
            file_size: section.file_size,
            memory_size,
            memory_size_unknown_reason: size_reason,
            region,
            region_unknown_reason: region_reason,
        }
    }
}

impl From<&Symbol> for SymbolRowDto {
    fn from(symbol: &Symbol) -> Self {
        let (name, name_reason) = text(&symbol.name);
        let (address, address_reason) = symbol.address.value().copied().map_or_else(
            || (None, reason_of(&symbol.address)),
            |value| (Some(hex_u64(value)), None),
        );
        let (size, size_reason) = fact_number(&symbol.size);

        Self {
            name,
            name_unknown_reason: name_reason,
            address,
            address_unknown_reason: address_reason,
            size,
            size_unknown_reason: size_reason,
            kind: kind_text(symbol.kind),
            binding: binding_text(symbol.binding),
            section: section_ref_text(symbol.section),
        }
    }
}

// --------------------------------------------------------------------------- vocabulary

/// A lowercase hex address or offset, un-converted.
fn hex_u64(value: u64) -> String {
    format!("0x{value:x}")
}

fn text(fact: &Fact<String>) -> (Option<String>, Option<String>) {
    match fact {
        Fact::Known(value) => (Some(value.clone()), None),
        Fact::Unknown { reason } => (None, Some(reason.clone())),
    }
}

fn fact_number(fact: &Fact<u64>) -> (Option<u64>, Option<String>) {
    (
        fact.value().copied(),
        fact.reason_if_unknown().map(str::to_owned),
    )
}

fn reason_of<T>(fact: &Fact<T>) -> Option<String> {
    fact.reason_if_unknown().map(str::to_owned)
}

const fn role_text(role: SectionRole) -> &'static str {
    match role {
        SectionRole::Code => "code",
        SectionRole::ReadOnlyData => "read-only-data",
        SectionRole::InitializedData => "initialized-data",
        SectionRole::UninitializedData => "uninitialized-data",
        SectionRole::Debug => "debug",
        SectionRole::Note => "note",
        SectionRole::SymbolTable => "symbol-table",
        SectionRole::StringTable => "string-table",
        SectionRole::ArmAttribute => "arm-attribute",
        SectionRole::Other => "other",
        SectionRole::Unknown => "unknown",
    }
}

const fn kind_text(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::NotType => "notype",
        SymbolKind::Object => "object",
        SymbolKind::Function => "function",
        SymbolKind::Section => "section",
        SymbolKind::File => "file",
        SymbolKind::Common => "common",
        SymbolKind::Other(_) => "other",
        SymbolKind::Unknown => "unknown",
    }
}

const fn binding_text(binding: SymbolBinding) -> &'static str {
    match binding {
        SymbolBinding::Local => "local",
        SymbolBinding::Global => "global",
        SymbolBinding::Weak => "weak",
        SymbolBinding::Other(_) => "other",
        SymbolBinding::Unknown => "unknown",
    }
}

fn section_ref_text(section: SymbolSectionRef) -> String {
    match section {
        SymbolSectionRef::Undefined => "undefined".to_owned(),
        SymbolSectionRef::Absolute => "absolute".to_owned(),
        SymbolSectionRef::Index(index) => format!("index:{index}"),
        SymbolSectionRef::Unknown => "unknown".to_owned(),
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

/// Serialize a `Sha256` as its hex string, so the newtype does not force a wrapper field.
mod serde_hex {
    use firmwaresight_core::domain::identity::Sha256;
    use serde::Serializer;

    pub fn serialize<S: Serializer>(value: &Sha256, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(value.hex())
    }
}
