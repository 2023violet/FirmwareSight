//! The Desktop/Application IPC boundary: the only place in this workspace where a TypeScript
//! shape is declared (`ADR-0019`, `04_TECH/25_INTERFACE_CONTRACT_BASELINE.md`).
//!
//! Deliberately bounded. Nothing here carries a raw byte buffer, and no payload is a whole table:
//! the detail shapes are one page of at most 500 rows, named by a snapshot id, and the full
//! normalized record stays in Core and in SQLite, where it is inspectable. The P2 compare summary is
//! bounded the same way: it carries the two budget deltas, the counts and at most five top sections
//! and ten top symbols, and every changed row is fetched by page through a session-local diff id.
//! An IPC payload that grows with the artifact is how a desktop shell stops responding.
//!
//! Generated `.ts` files land in `apps/desktop/ui/src/ipc/generated/` (see `.cargo/config.toml`,
//! which sets `TS_RS_EXPORT_DIR`) and are never hand-edited; CI regenerates them and fails on
//! drift.
//!
//! Unlike the CLI's JSON document, an optional field here is serialized as `null` rather than
//! omitted. ts-rs can only emit `foo: string | null`, and a payload that drops the key while the
//! type promises it would make the generated contract lie about what the UI will read.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The closed set of artifacts the P0 desktop can analyze.
///
/// A fixed enum rather than a path is the security boundary: the WebView cannot ask the shell to
/// read an arbitrary file, and there is no `read_file`/`get_any_path` command to misuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum FixtureKey {
    #[serde(rename = "p0_basic")]
    #[ts(rename = "p0_basic")]
    P0Basic,
    #[serde(rename = "p0_dual_region")]
    #[ts(rename = "p0_dual_region")]
    P0DualRegion,
}

impl FixtureKey {
    /// The key exactly as it appears on the wire and in the generated union.
    #[must_use]
    pub const fn as_key_str(self) -> &'static str {
        match self {
            Self::P0Basic => "p0_basic",
            Self::P0DualRegion => "p0_dual_region",
        }
    }

    /// Human label for the selection control. Presentation text, not a fact.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::P0Basic => "P0 basic (ELF only, no MAP)",
            Self::P0DualRegion => "P0 dual region (ELF + GNU ld MAP)",
        }
    }
}

/// One selectable entry in the closed fixture set. The label is Rust-owned so the generated
/// union and the text the user sees cannot drift apart.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FixtureOptionDto {
    pub key: String,
    pub label: String,
}

/// A selection the shell is holding on behalf of the WebView.
///
/// `selection_id` is an opaque, session-local handle. It is not a security token, it is not written
/// to the deterministic payload or the portable schema, and the paths it stands for stay on the Rust
/// side: the UI can neither read them back nor write them.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SelectionDto {
    pub selection_id: String,
    pub file_name: String,
    pub map_file_name: Option<String>,
    pub map_attached: bool,
}

/// Everything the Analyze screen shows.
///
/// `source` says how the payload arrived: `fixture` is the P0 engineering path, a committed file the
/// shell itself chose; `artifact` is the P1-A0 product path, a file the user selected in a native
/// dialog. Provenance is a fact, so it crosses the boundary instead of being inferred from a label.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AnalysisSummaryDto {
    /// `fixture` or `artifact`.
    pub source: String,
    pub artifact: ArtifactDto,
    pub identity: IdentityDto,
    pub memory: MemorySummaryDto,
    pub section_count: usize,
    pub symbol_count: usize,
    pub capabilities: CapabilitiesDto,
    pub evidence_summary: EvidenceSummaryDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ArtifactDto {
    /// File name only; a full path would make the payload machine-specific.
    pub file_name: String,
    pub kind: String,
    pub sha256: String,
    // Byte counts are far below 2^53 in P0, so `number` is exact. `bigint` would be a lie about
    // precision the artifact crate never claims.
    #[ts(type = "number")]
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

/// Which contract produced this payload, and from what.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IdentityDto {
    pub schema: String,
    pub schema_stability: String,
    pub snapshot_id: String,
    pub normalization_version: String,
    pub created_by_fwsight_version: String,
}

/// The two ADR-0021 budgets, with their evidence quality attached to the numbers rather than in
/// a footnote.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MemorySummaryDto {
    pub accounting_rule: String,
    pub layout_source: String,
    pub weakest_evidence_basis: Option<String>,
    pub admissible_for_hard_block: bool,
    pub nonvolatile_image_footprint: BudgetDto,
    pub runtime_ram_footprint: BudgetDto,
    pub dual_accounted_sections: Vec<String>,
    #[ts(type = "number")]
    pub excluded_metadata_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BudgetDto {
    /// `exact`, `partial` or `unknown`.
    pub state: String,
    pub classification: String,
    /// A `u64` is `number | bigint` to ts-rs, which would promise a precision the artifact crate
    /// never claims; `number` is exact at these magnitudes. An *optional* byte count still has to
    /// say it can be `null`, or the UI cannot render Unknown without the compiler refusing the
    /// check (US-001).
    #[ts(type = "number | null")]
    pub bytes: Option<u64>,
    pub unattributed: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CapabilitiesDto {
    pub elf: String,
    pub sections: String,
    pub symbols: String,
    pub debug_info: String,
    pub map: String,
    pub object_attribution: String,
    pub git: String,
}

/// Counts by evidence class, so the UI can state how much of what it shows was observed. The
/// evidence items themselves stay in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EvidenceSummaryDto {
    pub total: usize,
    pub observed: usize,
    pub derived: usize,
    pub declared: usize,
    pub unknown: usize,
}

/// The stable error shape, mirroring `firmwaresight_report::render::ErrorEnvelope` across the
/// IPC boundary.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ErrorEnvelopeDto {
    pub code: String,
    pub message: String,
    pub operation_id: String,
    pub details: Option<String>,
    pub remediation: Option<String>,
}

/// Which end of an ordering the reader asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SortDirDto {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SectionSortDto {
    /// The section header index: the locator inside one artifact, and the untouched default.
    #[default]
    Index,
    Name,
    FileSize,
    MemorySize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SymbolSortDto {
    /// Storage position within this build.
    #[default]
    Ordinal,
    Name,
    Address,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum EvidenceSortDto {
    #[default]
    Field,
    /// Evidence strength: observed, then derived, then declared, then unknown.
    Classification,
}

/// One of the four evidence classes, named the way SQLite stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum EvidenceClassDto {
    Observed,
    Derived,
    Declared,
    Unknown,
}

/// A request for one page of sections.
///
/// A snapshot id and a page, and nothing else: no path, no table name, no SQL. The three detail
/// requests share that shape on purpose - the surface is use-case oriented (`AGENTS.md` 7), so
/// there is nothing here a caller could widen.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SectionRequestDto {
    pub snapshot_id: String,
    /// A literal substring of a section name. `%` and `_` are characters, not syntax.
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub sort: SectionSortDto,
    #[serde(default)]
    pub direction: SortDirDto,
    #[serde(default)]
    pub offset: usize,
    /// `None` asks for the default page; Rust clamps any larger request to the hard maximum.
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SymbolRequestDto {
    pub snapshot_id: String,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub sort: SymbolSortDto,
    #[serde(default)]
    pub direction: SortDirDto,
    #[serde(default)]
    pub offset: usize,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EvidenceRequestDto {
    pub snapshot_id: String,
    /// A literal substring of the field an item is evidence about.
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub classification: Option<EvidenceClassDto>,
    #[serde(default)]
    pub sort: EvidenceSortDto,
    #[serde(default)]
    pub direction: SortDirDto,
    #[serde(default)]
    pub offset: usize,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// One section, exactly as Core recorded it.
///
/// Every value that pairs with an unknown reason is `null` in the value slot and text in the reason
/// slot; the UI renders `Unknown` from the reason and never a zero (US-001). Addresses and file
/// offsets are hexadecimal text because they are locators, and byte counts are numbers because the
/// bytes/KiB switch is presentation the UI applies to them.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SectionRowDto {
    pub index: usize,
    pub name: Option<String>,
    pub name_unknown_reason: Option<String>,
    pub role: String,
    pub alloc: bool,
    pub write: bool,
    pub execute: bool,
    pub virtual_address: Option<String>,
    pub virtual_address_unknown_reason: Option<String>,
    pub load_address: Option<String>,
    pub load_address_unknown_reason: Option<String>,
    /// The schema records no reason for an absent file offset, so only the absence crosses.
    pub file_offset: Option<String>,
    #[ts(type = "number")]
    pub file_size: u64,
    #[ts(type = "number | null")]
    pub memory_size: Option<u64>,
    pub memory_size_unknown_reason: Option<String>,
    pub region: Option<String>,
    pub region_unknown_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SymbolRowDto {
    /// A row position in this build, not an identity the UI can compare across builds.
    pub ordinal: usize,
    pub name: Option<String>,
    pub name_unknown_reason: Option<String>,
    pub address: Option<String>,
    #[ts(type = "number | null")]
    pub size: Option<u64>,
    pub size_unknown_reason: Option<String>,
    pub kind: String,
    pub binding: String,
    pub section_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EvidenceRowDto {
    pub id: String,
    pub field: String,
    /// `observed`, `derived`, `declared` or `unknown`.
    pub classification: String,
    pub source_type: String,
    /// A deterministic locator the reader can go back and re-check, never a host path.
    pub source_locator: String,
    pub raw_value: String,
    pub rule: String,
    pub confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SectionPageDto {
    pub rows: Vec<SectionRowDto>,
    pub total: usize,
    pub offset: usize,
    /// The page size Rust actually applied, after clamping.
    pub limit: usize,
    pub next_offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SymbolPageDto {
    pub rows: Vec<SymbolRowDto>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub next_offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EvidencePageDto {
    pub rows: Vec<EvidenceRowDto>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub next_offset: Option<usize>,
}

// --------------------------------------------------------------------------- Compare (P2)

/// One stored build the Compare selectors may offer.
///
/// A candidate exists only if the build was analyzed and persisted by this application: the list is
/// read from SQLite, never from the filesystem, so it stays valid after the original ELF and MAP
/// have moved or been deleted. Names only - the stored path is an intake fact and does not cross
/// this boundary.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CompareCandidateDto {
    pub build_id: String,
    pub snapshot_id: String,
    pub file_name: String,
    pub sha256: String,
    #[ts(type = "number")]
    pub byte_size: u64,
    pub architecture: String,
    /// When the build entered history, exactly as stored. Presentation of a recorded fact, not a
    /// value this run generated.
    pub imported_at: String,
    pub nonvolatile: StoredBudgetDto,
    pub runtime_ram: StoredBudgetDto,
}

/// A recorded budget and how completely it was accounted for. `partial` is a floor.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StoredBudgetDto {
    /// `exact`, `partial` or `unknown`.
    pub state: String,
    #[ts(type = "number | null")]
    pub bytes: Option<u64>,
}

/// A page request for the candidate list. The project scope is not askable: Rust fixes it, so the
/// WebView cannot enumerate history that belongs to another project (`AGENTS.md` 7).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CandidatePageRequestDto {
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CandidatePageDto {
    pub rows: Vec<CompareCandidateDto>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub next_offset: Option<usize>,
}

/// The two builds to compare, by snapshot id. Which one is old is stated, never inferred from order
/// of appearance.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CompareRequestDto {
    pub base_snapshot_id: String,
    pub target_snapshot_id: String,
}

/// One side of a comparison, as far as the summary needs to name it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiffSideDto {
    pub snapshot_id: String,
    pub file_name: String,
    pub sha256: String,
}

/// One numeric comparison. `base`/`target` are `null` when that side holds no number, which for an
/// added or removed row means *absent*: the UI renders `—`/`Not present`, never `0` (US-002).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ByteDeltaDto {
    #[ts(type = "number | null")]
    pub base: Option<u64>,
    #[ts(type = "number | null")]
    pub target: Option<u64>,
    /// `target - base`, signed. `null` when either side is absent, unknown, or the difference does
    /// not fit.
    #[ts(type = "number | null")]
    pub delta: Option<i64>,
    /// `exact`, `partial` or `unknown` - the strength of this particular comparison.
    pub comparability: String,
    pub reason: Option<String>,
}

/// One side's recorded memory totals with the evidence that produced them.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiffSideMemoryDto {
    /// False when the build has no stored footprint row. Not the same claim as a row of zero bytes.
    pub footprint_row_present: bool,
    pub nonvolatile: StoredBudgetDto,
    pub runtime_ram: StoredBudgetDto,
    pub map_backed: bool,
    pub layout_source: String,
    pub weakest_evidence_basis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiffMemoryDto {
    pub nonvolatile: ByteDeltaDto,
    pub runtime_ram: ByteDeltaDto,
    /// The weaker side caps the pair, and the UI must not present a partial pair like an exact one.
    pub comparability: String,
    pub base: DiffSideMemoryDto,
    pub target: DiffSideMemoryDto,
    pub evidence_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChangeKindCountsDto {
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    /// Rows left unpaired because the name or key repeats, or because there is no name to match on.
    pub ambiguous: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiffCountsDto {
    pub sections: ChangeKindCountsDto,
    pub symbols: ChangeKindCountsDto,
    pub unchanged_sections: usize,
    pub unchanged_symbols: usize,
}

/// Whether an object or module delta can be claimed at all.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ObjectAttributionDto {
    pub available: bool,
    pub reason: String,
}

/// A ranked contributor. Navigation only: the paged change tables are the authoritative list
/// (prompt §25).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ContributorDto {
    pub key: String,
    /// `Added`, `Changed` or `Removed`. Diff vocabulary, never a Gate state.
    pub change_kind: String,
    #[ts(type = "number | null")]
    pub delta: Option<i64>,
    #[ts(type = "number | null")]
    pub bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiffWarningDto {
    pub code: String,
    pub message: String,
}

/// The bounded answer to `compare_snapshots`. It deliberately holds no full change table: a build
/// can change thousands of symbols, and a payload that grows with the artifact is how a desktop
/// shell stops responding. Rows are fetched one page at a time with `diffId`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CompareSummaryDto {
    /// Session-local handle for the computed diff. Not persisted, not portable, and not part of the
    /// diff's content identity.
    pub diff_id: String,
    pub base: DiffSideDto,
    pub target: DiffSideDto,
    pub memory: DiffMemoryDto,
    pub counts: DiffCountsDto,
    pub object_changes: ObjectAttributionDto,
    pub top_sections: Vec<ContributorDto>,
    pub top_symbols: Vec<ContributorDto>,
    pub warnings: Vec<DiffWarningDto>,
}

/// Which change kinds to show. Absent means all three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ChangeKindFilterDto {
    Added,
    Removed,
    Changed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SectionChangeSortDto {
    /// Section name, or the row position when there is no name. The default.
    #[default]
    Key,
    /// File-size delta. Rows with no delta sort last, so an unknown never masquerades as small.
    Delta,
    ChangeKind,
    FileSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SymbolChangeSortDto {
    #[default]
    Name,
    SizeDelta,
    ChangeKind,
    Size,
}

/// One page of a stored diff's change table.
///
/// `diff_id` selects a computation this session already made; there is no table name, statement or
/// path in this request, so it cannot be widened (`AGENTS.md` 7).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SectionChangeQueryDto {
    pub diff_id: String,
    /// A literal substring of the row key.
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub change_kind: Option<ChangeKindFilterDto>,
    #[serde(default)]
    pub sort: SectionChangeSortDto,
    #[serde(default)]
    pub direction: SortDirDto,
    #[serde(default)]
    pub offset: usize,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SymbolChangeQueryDto {
    pub diff_id: String,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub change_kind: Option<ChangeKindFilterDto>,
    #[serde(default)]
    pub sort: SymbolChangeSortDto,
    #[serde(default)]
    pub direction: SortDirDto,
    #[serde(default)]
    pub offset: usize,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// One side of a changed section row. `None` means the section does not exist on that side, which
/// the UI renders as absence, never as zero (prompt §23).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SectionSideDto {
    /// Row position within this build. Never an identity across builds.
    pub index: usize,
    pub role: String,
    pub region: Option<String>,
    pub virtual_address: Option<String>,
    pub load_address: Option<String>,
    pub file_offset: Option<String>,
    #[ts(type = "number")]
    pub file_size: u64,
    #[ts(type = "number | null")]
    pub memory_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SectionChangeRowDto {
    /// `Added`, `Removed` or `Changed`.
    pub change_kind: String,
    pub key: String,
    pub name_known: bool,
    /// True when the name repeats on one or both sides and the rows were therefore left unpaired.
    pub ambiguous: bool,
    pub file_size: ByteDeltaDto,
    pub memory_size: ByteDeltaDto,
    pub base: Option<SectionSideDto>,
    pub target: Option<SectionSideDto>,
    pub differing_fields: Vec<String>,
    pub indeterminate_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SymbolSideDto {
    pub ordinal: usize,
    pub address: Option<String>,
    #[ts(type = "number | null")]
    pub size: Option<u64>,
    pub section_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SymbolChangeRowDto {
    pub change_kind: String,
    pub name: String,
    pub kind: String,
    pub binding: String,
    pub ambiguous: bool,
    pub size: ByteDeltaDto,
    pub base: Option<SymbolSideDto>,
    pub target: Option<SymbolSideDto>,
    pub differing_fields: Vec<String>,
    pub indeterminate_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SectionChangePageDto {
    pub rows: Vec<SectionChangeRowDto>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub next_offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SymbolChangePageDto {
    pub rows: Vec<SymbolChangeRowDto>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub next_offset: Option<usize>,
}

/// What an export command actually did.
///
/// The save path is chosen by a person in a dialog the Rust side opened, and it is not returned:
/// the UI gets the file name, the format and the outcome. `cancelled` and `kept-existing` are normal
/// outcomes, not errors, because deciding not to overwrite a file is a decision rather than a
/// failure (prompt §35, §36).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportOutcomeDto {
    /// `written`, `cancelled` or `kept-existing`.
    pub status: String,
    pub file_name: Option<String>,
    /// `json` or `html`.
    pub format: String,
}
