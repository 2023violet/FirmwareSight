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
//!
//! P3's Gate payloads stay inside the same discipline. A run carries ten findings, which is a bounded
//! set by construction, so the whole finding list may cross the boundary — but no config text, no
//! project root and no artifact path does, and the numbers a table shows are the ones Rust read out of
//! the stored build (prompt §46, §51–§53).

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
    pub file_offset: Option<String>,
    pub file_offset_unknown_reason: Option<String>,
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
    pub address_unknown_reason: Option<String>,
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
    /// Memory size, taking whichever side carries a number. This is the quantity the diff ranks top
    /// growth on, so the Compare screen's "largest additions" list and its growth list can be read
    /// against each other.
    MemorySize,
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

// --------------------------------------------------------------------------- Release Gate (P3)

/// What may be done with an `UNKNOWN` rule, as the release owner's policy says (ADR-0023).
///
/// Two words, because "pass it anyway" is not one of them, and the UI must not be able to send it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum UnknownDispositionDto {
    Review,
    Block,
}

/// `[gate.on_unknown]`: one disposition per rule that can lose its evidence.
///
/// `null` on a field means the config said nothing about that rule, and `04_TECH/08:60-62`'s default is
/// what applies — a budget answers to a block, everything else to a review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UnknownPolicyDto {
    pub git_clean: Option<UnknownDispositionDto>,
    pub commit_matches_release: Option<UnknownDispositionDto>,
    pub version_match: Option<UnknownDispositionDto>,
    pub flash_budget: Option<UnknownDispositionDto>,
    pub ram_budget: Option<UnknownDispositionDto>,
    pub baseline_growth: Option<UnknownDispositionDto>,
    pub release_notes: Option<UnknownDispositionDto>,
}

/// The policy fields the Release page reads and edits (prompt §41).
///
/// This is a *policy*, not a config file: the raw TOML text, the keys this build does not understand
/// and the project root all stay on the Rust side, so the front end can name a budget but cannot name
/// a file (`AGENTS.md` 7, prompt §40, §46).
/// Every field is optional in the same sense a `firmwaresight.toml` key is optional: `null` means
/// "this page stated nothing", and the default the product documents is what applies. That is why no
/// default lives in the front end — the shell resolves each unset field against `GatePolicy::default()`
/// on the way in, so a screen and a config file cannot disagree about what an empty box means.
///
/// A policy that came *from* a loaded config arrives fully resolved, because what a run judged with is a
/// fact the reader is entitled to see.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProjectPolicyDto {
    pub project_name: String,
    /// The v1 vocabulary: `elf`, `map`, `bin`, `hex`. `null` keeps the documented default (`elf`).
    pub required_artifact_kinds: Option<Vec<String>>,
    #[ts(type = "number | null")]
    pub flash_budget: Option<u64>,
    #[ts(type = "number | null")]
    pub ram_budget: Option<u64>,
    pub require_clean_git: Option<bool>,
    pub require_release_notes: Option<bool>,
    /// Project-relative, never absolute (prompt §53). `null` keeps `RELEASE_NOTES.md`.
    pub release_notes_path: Option<String>,
    /// `git_tag`, or `null` when no `[version]` section applies to this project.
    pub version_source: Option<String>,
    pub version_pattern: Option<String>,
    pub expected_version: Option<String>,
    pub expected_commit: Option<String>,
    #[ts(type = "number | null")]
    pub flash_growth_review_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub ram_growth_review_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub unknown_evidence_review_count: Option<u64>,
    pub on_unknown: UnknownPolicyDto,
}

/// The project this session loaded a policy from.
///
/// `warnings` and `unknown_keys` are facts about the file, not about the build: a warning never fails a
/// Gate run (prompt §49), and an unknown key is listed here rather than silently kept or silently
/// dropped (prompt §42).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProjectContextDto {
    pub project_name: String,
    /// `firmwaresight.toml`. The file's name, never the directory holding it.
    pub config_file_name: String,
    /// The `schema_version` the config declared.
    #[ts(type = "number")]
    pub config_schema_version: i64,
    pub policy: ProjectPolicyDto,
    pub policy_sha256: String,
    pub warnings: Vec<String>,
    pub unknown_keys: Vec<String>,
}

/// The two builds one Gate run judges. Snapshot ids and nothing else — no path, no statement, and the
/// project whose policy applies is the one this session loaded (`AGENTS.md` 7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateRunRequestDto {
    pub snapshot_id: String,
    pub baseline_snapshot_id: Option<String>,
}

/// Accept that one `REVIEW` finding is disposed of, and by whom.
///
/// `actor` and `reason` are required text, not optional decoration: an acceptance without a name and a
/// reason is not an audit record (prompt §48).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AcceptReviewRequestDto {
    pub run_id: String,
    pub finding_id: String,
    pub actor: String,
    pub reason: String,
}

/// What an acceptance did to one run: the row it accepted, and the aggregate that now applies.
///
/// The finding itself is unchanged and stays `REVIEW` forever; the display patches two places from this,
/// which is why it is not a whole `GateRunDto` (`04_TECH/27`, prompt §48).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AcceptReviewOutcomeDto {
    pub run_id: String,
    pub finding_id: String,
    /// The finding's state, which is still `REVIEW`. Carried so the screen cannot render an accepted
    /// review as a pass.
    pub state: String,
    pub acceptance: AcceptedReviewDto,
    pub disposition_effective_severity: String,
}

/// One accepted review, as stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AcceptedReviewDto {
    pub finding_id: String,
    pub actor: String,
    /// When SQLite recorded the decision. Not a build time and not the reviewer's clock.
    pub accepted_at: String,
    pub reason: String,
    /// Always `REVIEW`: the state that was accepted (ADR-0023).
    pub original_state: String,
}

/// How many findings landed in each factual state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateCountsDto {
    pub pass: usize,
    pub review: usize,
    pub block: usize,
    pub unknown: usize,
    pub not_applicable: usize,
}

/// One rule's answer, with the acceptance beside it if a person recorded one.
///
/// `acceptable` is Rust's decision, not a style choice: only a `REVIEW` finding can be accepted, so the
/// screen renders no control for anything else (prompt §48).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateFindingRowDto {
    pub id: String,
    /// The stable rule identity, shown in mono: `git.clean`, `memory.flash_budget`, …
    pub rule_id: String,
    /// `PASS` / `REVIEW` / `BLOCK` / `UNKNOWN` / `N/A`.
    pub state: String,
    /// The separate ADR-0023 field. An `UNKNOWN` row can carry `BLOCK`.
    pub effective_severity: String,
    pub summary: String,
    pub evidence_refs: Vec<String>,
    pub remediation: Option<String>,
    pub acceptable: bool,
    pub acceptance: Option<AcceptedReviewDto>,
}

/// The workspace provenance this run read.
///
/// Wording matters here: these are facts about the *workspace*, never proof of how an artifact was
/// built, so the summary names the workspace and no string in this payload says otherwise (prompt §50).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateGitDto {
    pub available: bool,
    pub head_commit: Option<String>,
    pub exact_tag: Option<String>,
    /// `None` means the probe did not learn whether the workspace is dirty — not that it is clean.
    pub dirty: Option<bool>,
    /// Rust's own sentence about what was observed, from a fixed set.
    pub summary: String,
    pub reason: Option<String>,
}

/// One artifact row of the policy-readiness tables: required, present, digested (prompt §51).
///
/// A required BIN or HEX is listed as required because the config says so. FirmwareSight does not
/// analyze those formats in P3 and the screen must not imply that it does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateArtifactRowDto {
    /// `elf`, `map`, `bin`, `hex` or `unknown`.
    pub kind: String,
    pub required: bool,
    pub present: bool,
    pub sha256: Option<String>,
    #[ts(type = "number | null")]
    pub byte_size: Option<u64>,
}

/// One budget, with the number it was measured at and the evidence that number rests on (prompt §52).
///
/// `headroom_bytes` and `over_bytes` are filled in at most one of, and only for a rule that reached a
/// deterministic verdict. An `UNKNOWN` row carries no invented number: both stay `null` and `reason`
/// says what is missing (US-001).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateBudgetRowDto {
    /// `flash` or `ram`.
    pub side: String,
    pub label: String,
    pub rule_id: String,
    pub state: String,
    pub effective_severity: String,
    #[ts(type = "number | null")]
    pub budget_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub actual_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub headroom_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub over_bytes: Option<u64>,
    /// Whether the total is a complete attribution rather than a floor.
    pub exact: bool,
    pub admissible: bool,
    pub basis: Option<String>,
    pub reason: Option<String>,
}

/// Growth against the baseline, moved here from P2's Core diff and never recomputed (prompt §52).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateGrowthRowDto {
    pub side: String,
    pub label: String,
    pub rule_id: String,
    /// The state of the single `diff.growth` rule, shared by both sides.
    pub state: String,
    pub effective_severity: String,
    #[ts(type = "number | null")]
    pub old_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub new_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub delta_bytes: Option<i64>,
    #[ts(type = "number | null")]
    pub threshold_bytes: Option<u64>,
    /// `exact`, `partial` or `unknown` — P2's comparability of this delta.
    pub comparability: String,
    pub reason: Option<String>,
}

/// The Release Notes file the policy points at, by project-relative path only (prompt §53).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateNotesDto {
    pub required: bool,
    pub state: String,
    pub effective_severity: String,
    pub relative_path: Option<String>,
    pub present: Option<bool>,
    pub sha256: Option<String>,
    pub reason: Option<String>,
}

/// The number-backed tables under the findings.
///
/// `null` in [`GateRunDto::tables`] for a run read back from history: the stored record keeps each
/// finding's own text and evidence, but the policy that produced these numbers is not part of it, and
/// a table rebuilt against a different policy would state a verdict the run never made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateTablesDto {
    pub artifacts: Vec<GateArtifactRowDto>,
    pub budgets: Vec<GateBudgetRowDto>,
    pub growth: Vec<GateGrowthRowDto>,
    pub notes: GateNotesDto,
}

/// One Gate run: the aggregate, the findings, and the facts they were judged from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GateRunDto {
    pub run_id: String,
    pub snapshot_id: String,
    pub baseline_snapshot_id: Option<String>,
    /// The project whose policy produced this run, or `null` for a run read back from history whose
    /// policy is not loaded. A default-policy run has no project name to state (§40, §46).
    pub project_name: Option<String>,
    /// Which policy produced this run: the loaded config's file name, or the stated FirmwareSight
    /// default. A default-policy run is never labelled as a project's own policy (prompt §40).
    pub policy_source: String,
    pub policy_sha256: String,
    /// When FirmwareSight stored this run. Not a build time (`04_TECH/24` Time).
    pub created_at: String,
    /// The aggregate with no acceptances counted.
    pub overall_effective_severity: String,
    /// The aggregate once this run's accepted reviews are counted. Core derives both; neither is
    /// recomputed here.
    pub disposition_effective_severity: String,
    pub counts: GateCountsDto,
    /// Findings in canonical rule order; the screen groups them by state.
    pub findings: Vec<GateFindingRowDto>,
    /// Config warnings. A warning is not a failure and never appears as a finding (prompt §49).
    pub warnings: Vec<String>,
    pub git: GateGitDto,
    /// The policy this run judged with, resolved by Rust. `null` for a run read back from history whose
    /// policy is not loaded: the fingerprint is stored, the values are not, and a policy invented for the
    /// display would be a second claim about the run.
    pub policy: Option<ProjectPolicyDto>,
    pub tables: Option<GateTablesDto>,
    /// Set only for a run read back from history, and says what such a record does and does not show.
    pub record_note: Option<String>,
}

// --------------------------------------------------------------------------- Release Bundle (P4)

/// The three ids one bundle is assembled from, and nothing else (§26).
///
/// No path, no artifact bytes, no destination: the build and the baseline are named by the snapshot ids the
/// screen already holds, and the Gate run by the id it was stored under. Which project policy applies is not
/// in the request either — it is the config a person chose in a dialog the WebView cannot see, which is the
/// only way a policy hash can be a fact rather than an assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BundlePlanRequestDto {
    pub snapshot_id: String,
    pub baseline_snapshot_id: Option<String>,
    pub gate_run_id: String,
}

/// One file the bundle will hold (§27, §49).
///
/// `path` is bundle-relative with `/` separators — it is the name inside the folder, never where the folder
/// is. `role` says what the file is for, so the screen can group without inferring it from a name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BundleFileRowDto {
    pub path: String,
    pub role: String,
    /// Byte counts in a bundle are far below 2^53 — the whole bundle is a firmware image and its documents —
    /// so `number` is exact and `bigint` would claim a precision the payload does not need (`ArtifactDto`).
    #[ts(type = "number")]
    pub byte_size: u64,
    pub sha256: String,
}

/// What a plan would write, bounded to what a release owner can authorize (§27).
///
/// `plan_id` is session-local: it names this plan to this process, is not persisted, is not portable, and is
/// not the release id. Nothing here carries a host path, a source path or a destination — the preview is what
/// a person reads *before* choosing a folder, so a path in it would be an answer to a question nobody asked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BundlePreviewDto {
    pub plan_id: String,
    pub release_id: String,
    pub release_version: String,
    pub snapshot_id: String,
    pub baseline_snapshot_id: Option<String>,
    pub gate_run_id: String,
    /// `PASS`, always: §8 refuses to plan anything else, so this states a promise rather than a status.
    pub disposition: String,
    /// How many of this run's REVIEW findings this session loaded acceptances for. Core's aggregate decided
    /// the disposition; the count is shown so a person can see what the bundle is standing on (§49).
    pub accepted_review_count: usize,
    pub files: Vec<BundleFileRowDto>,
    pub warnings: Vec<String>,
    /// The directory name the bundle proposes inside the folder the release owner will choose (§29). A
    /// display name, not a path.
    pub bundle_folder_name: String,
}

/// A destination the release owner chose in the native dialog, as the WebView is allowed to know it (§29).
///
/// The chosen folder itself never crosses the boundary: the UI gets a token that names it to this process,
/// the folder name the bundle will be created under, and whether something of that name is already there —
/// which is the whole of what an overwrite confirmation needs to say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BundleDestinationDto {
    pub destination_token: String,
    pub bundle_folder_name: String,
    pub exists: bool,
    /// True when the folder that exists there is readable as a FirmwareSight bundle, which is the only kind
    /// a replacement is allowed to touch (§32). Not a judgement about any other directory.
    pub recognizable_bundle: bool,
}

/// What an export produced (§50).
///
/// `folder_display_name` is the bundle's own directory name, not where it is: a person who chose the parent
/// already knows. `manifest_sha256` is the digest of `release-manifest.json` as written, which is the fact
/// that ties the release record to the directory without naming it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BundleExportDto {
    pub release_id: String,
    pub release_version: String,
    pub folder_display_name: String,
    pub manifest_sha256: String,
    pub file_count: usize,
    #[ts(type = "number")]
    pub total_bytes: u64,
    pub artifact_count: usize,
    /// True when this export replaced a recognized bundle under explicit confirmation (§30).
    pub replaced: bool,
    /// True when the release record is now in the validation database. A record write that fails after a
    /// successful export is reported, never quietly skipped (§57).
    pub record_written: bool,
}

// ------------------------------------------------------------------------ History (P5)

/// The page whose name the window title carries.
///
/// A closed enum rather than a string for the same reason `FixtureKey` is one: it is the boundary.
/// The title text is Rust's, so no page name — and certainly no file name or directory — can be put
/// into the window title from the WebView (prompt §14, `AGENTS.md` 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum MainWindowPage {
    Analyze,
    Compare,
    Release,
    History,
    Help,
}

impl MainWindowPage {
    /// The full title for this page. `FirmwareSight` stays first so a taskbar with several windows
    /// still reads as one product.
    #[must_use]
    pub const fn window_title(self) -> &'static str {
        match self {
            Self::Analyze => "FirmwareSight - Analyze",
            Self::Compare => "FirmwareSight - Compare",
            Self::Release => "FirmwareSight - Release",
            Self::History => "FirmwareSight - History",
            Self::Help => "FirmwareSight - Help",
        }
    }
}

/// A bounded page request for one History table.
///
/// Which project's history is in scope is not askable, exactly as in `CandidatePageRequestDto`: Rust
/// fixes it, so the WebView cannot enumerate another project's records (`AGENTS.md` 7, prompt §44).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryPageRequestDto {
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
    /// Text to narrow the table to. Rust decides which columns it searches, escapes LIKE syntax,
    /// and never searches a stored path (prompt §16, §44).
    #[serde(default)]
    pub filter: Option<String>,
}

/// One stored Gate run, as History lists it.
///
/// `disposition` is the verdict that was recorded, and `counts` is how its findings were stored;
/// neither is recomputed for the screen. `fileName` is a name, never a path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryGateRunRowDto {
    pub run_id: String,
    pub build_id: String,
    pub file_name: String,
    pub baseline_build_id: Option<String>,
    pub baseline_file_name: Option<String>,
    pub disposition: String,
    pub counts: GateCountsDto,
    /// When this application recorded the verdict, exactly as stored.
    pub stored_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryGateRunPageDto {
    pub rows: Vec<HistoryGateRunRowDto>,
    pub total: usize,
    pub offset: usize,
    /// The page size Rust actually applied, after clamping.
    pub limit: usize,
    pub next_offset: Option<usize>,
}

/// One published release, as History lists it. The manifest digest ties the record to a directory
/// without naming one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryReleaseRowDto {
    pub release_id: String,
    pub release_version: String,
    pub build_id: String,
    pub file_name: String,
    /// The run whose disposition qualified this release.
    pub gate_run_id: String,
    pub manifest_sha256: String,
    pub stored_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryReleasePageDto {
    pub rows: Vec<HistoryReleaseRowDto>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
    pub next_offset: Option<usize>,
}

/// What the running application says about itself on the Help screen (prompt §14).
///
/// Every field is the runtime's own answer, not text the WebView supplied and not a number the front
/// end kept a copy of. `storeFileName` is a file name on purpose: no surface of this product reports
/// the directory the store sits in, and Diagnostics is one of the surfaces that does not, because
/// prompt §19 puts an absolute path outside that payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppIdentityDto {
    /// The bundled product name, or the crate name if a build did not set one.
    pub product_name: String,
    /// The executable's own name, which is what an installer puts on disk.
    pub binary_name: String,
    /// The version this binary was compiled from.
    pub app_version: String,
    /// The application identifier the packages are keyed by.
    pub identifier: String,
    /// This machine's operating system and CPU, in one short token, for the Support Matrix question
    /// a reader is most likely asking.
    pub platform: String,
    /// The schema version the store this application opened carries.
    pub storage_schema_version: i64,
    /// The store's file name, never its directory.
    pub store_file_name: String,
}

// --------------------------------------------------------------------------- Diagnostics (P5 Commit D)

/// The support payload prompt §19 defines: bounded, explicit fields, every one filled by Rust from a
/// source that is not the WebView.
///
/// This is a Desktop/Application contract, not a portable firmware schema — it is versioned by its own
/// `schema` label and nothing outside this shell reads it. The shape is an allowlist made of closed
/// structs precisely so that "what a Diagnostics file contains" cannot grow by an accident of
/// serialization: there is no map here, no `serde_json::Value`, and no field whose type is a path.
/// Prompt §7's prohibitions (no artifact path, no project root, no database directory, no home, no
/// username, no remote, no bytes, no notes text, no symbol names, no environment dump, no SQL, no raw
/// rows) are all enforced by the simple fact that no field of this struct can hold such a thing, and
/// tested by `tests/diagnostics.rs`, which plants each of them in the session first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiagnosticsDto {
    /// This payload's own contract label, so a support file states what shape it is.
    pub schema: String,
    /// When this was captured, as read by the engine. Observation metadata only: FirmwareSight hashes
    /// no field of this payload, and no identity claim reads it (prompt §18).
    pub generated_at: Option<String>,
    pub product: DiagnosticsProductDto,
    pub runtime: DiagnosticsRuntimeDto,
    pub store: DiagnosticsStoreDto,
    pub git: DiagnosticsGitDto,
    pub support: DiagnosticsSupportDto,
    /// The policy the Release page would run against, when one is loaded. `null` is the honest answer
    /// for a session that has not opened a project (prompt §6: a policy *may* be reported, not must).
    pub policy: Option<DiagnosticsPolicyDto>,
    /// Stable error codes this process produced, newest last, capped. Codes only — never a message,
    /// because a message can quote the path it failed on (prompt §24).
    pub recent_error_codes: Vec<String>,
}

/// What the product is. Names and versions, no locations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiagnosticsProductDto {
    pub product_name: String,
    pub app_version: String,
    pub binary_name: String,
    /// The application identifier the packages are keyed by: a reverse-domain constant, not a path.
    pub identifier: String,
    /// The logical store name (prompt §8): `firmwaresight-p0.sqlite`, file name only, no parent.
    pub store_file_name: String,
}

/// What the program is running on. An unavailable fact says `not_reported`; nothing here is guessed
/// and nothing here needed a new dependency to read (prompt §26).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiagnosticsRuntimeDto {
    /// `windows`, `macos`, `linux` — the operating system family.
    pub os_family: String,
    /// `x86_64`, `aarch64` and friends.
    pub architecture: String,
    /// The two joined, which is the form the Support Matrix speaks in.
    pub platform: String,
    /// `not_reported`. This build has no OS-version source it trusts: the standard library does not
    /// carry one, and prompt §26 forbids a platform-specific dependency for a single field.
    pub os_version: String,
    /// The Tauri version this binary was compiled against.
    pub tauri_version: String,
    /// The WebView runtime, when the platform reports one; `not_reported` otherwise.
    pub webview_version: String,
}

/// The store's own health and shape. `health` is the point of the section; the rest says what a
/// support conversation would otherwise have to ask a person to read off a screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiagnosticsStoreDto {
    /// The schema the *file* carries, read from its bookkeeping. `null` when the store would not
    /// answer, which is itself a fact worth having.
    #[ts(type = "number | null")]
    pub schema_version: Option<i64>,
    /// The schema this build speaks. Equal to the field above for any store this application opened.
    #[ts(type = "number")]
    pub supported_schema_version: i64,
    /// `healthy`, `unhealthy`, or `unknown` when the check itself could not be asked.
    pub health: String,
    /// The bounded, directory-free problems, present only for `unhealthy`.
    pub health_summary: Option<String>,
    /// A stable code, present only for `unknown`. A code and never a message.
    pub health_error_code: Option<String>,
    /// `wal`, `delete`, `memory`, `off`, `truncate`, `persist` or `other`.
    pub journal_mode: String,
    pub counts: DiagnosticsCountsDto,
    /// File names of the pre-migration snapshots kept beside the store, in name order, capped. A
    /// snapshot's existence is the recovery story a support file has to be able to tell (prompt §17),
    /// and a name is not a path.
    pub backup_files: Vec<String>,
}

/// The five object classes prompt §6 names. A count that SQLite would not answer for is `null` rather
/// than `0`: "no builds" and "this damaged store would not say" are different facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiagnosticsCountsDto {
    #[ts(type = "number | null")]
    pub projects: Option<i64>,
    #[ts(type = "number | null")]
    pub builds: Option<i64>,
    #[ts(type = "number | null")]
    pub gate_runs: Option<i64>,
    #[ts(type = "number | null")]
    pub accepted_reviews: Option<i64>,
    #[ts(type = "number | null")]
    pub release_records: Option<i64>,
}

/// Whether Git can be reached from this application at all, and which Git it is. Nothing about any
/// repository: no remote, no path, no branch, no commit (prompt §25).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiagnosticsGitDto {
    pub available: bool,
    /// The first line of `git --version`, bounded and shape-checked, or `not_reported`.
    pub version: String,
}

/// What this product claims to read, and how it was delivered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiagnosticsSupportDto {
    /// The supported input cohort, stated by this build, and nothing the parser was asked to infer.
    pub input_cohort: String,
    /// The bundle the running binary carries its marker in (`msi`, `nsis`, `deb`, `rpm`, `appimage`,
    /// `app`), or `unknown` when no marker was written. A path string was never used to guess this
    /// (prompt §27).
    pub install_channel: String,
}

/// The loaded project's policy, reduced to the flags that decide a verdict.
///
/// `require_clean_git` is reported because it changes what the Gate believes, and it travels with the
/// caveat the Architect attached to it (prompt §0, §6's L22 note): a reader who sees `false` has to be
/// able to learn what that does and does not mean without opening the source. No project name, no root,
/// no config path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiagnosticsPolicyDto {
    #[ts(type = "number")]
    pub config_schema_version: i64,
    pub require_clean_git: bool,
    pub require_clean_git_note: String,
    pub require_release_notes: bool,
}
