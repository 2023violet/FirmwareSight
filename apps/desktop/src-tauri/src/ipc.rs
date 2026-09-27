//! The Desktop/Application IPC boundary: the only place in this workspace where a TypeScript
//! shape is declared (`ADR-0019`, `04_TECH/25_INTERFACE_CONTRACT_BASELINE.md`).
//!
//! Deliberately bounded. Nothing here carries a symbol table, a section list, or a raw byte
//! buffer: an IPC payload that grows with the artifact is how a desktop shell stops responding.
//! The full normalized record stays in Core and in SQLite, where it is inspectable.
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

/// Everything the P0 summary screen shows.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AnalysisSummaryDto {
    pub fixture: String,
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
    #[ts(type = "number")]
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
    /// Whether any part of this summary rests on MAP or region-config evidence.
    pub from_map: bool,
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
