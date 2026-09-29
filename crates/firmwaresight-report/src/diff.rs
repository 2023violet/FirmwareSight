//! The portable projection of a Core `DiffResult`.
//!
//! This is the shape behind `urn:firmwaresight:schema:diff:1`, so the same rules that govern the
//! analyze DTO apply here: field declaration order *is* emitted key order, nothing is a `HashMap`,
//! nothing carries a wall-clock timestamp, and nothing carries a host path.
//!
//! Nothing is recomputed here either. Every number, every `Added`/`Removed`/`Changed` label and
//! every unknown reason is read off the Core result, which is the only place diff semantics live
//! (`AGENTS.md` 3). A second implementation of the comparison would be a second thing that can be
//! wrong, and the desktop and CLI would drift from the export.
//!
//! Absence is carried as absence. A row that exists only on one side reports the other side as
//! `null` with a reason, never as `0`, because `0` is a measurement that was not taken here.

use serde::Serialize;

use firmwaresight_core::domain::identity::Fact;

use firmwaresight_core::domain::diff::{
    BudgetState, ByteChange, ChangeCounts, DiffArtifact, DiffBudget, DiffResult, DiffSection,
    DiffSideMemory, DiffSymbol, SectionChange, SymbolChange,
};

/// The emitted document's own identity, so a reader can find the schema without side channel.
pub const DIFF_SCHEMA_ID: &str = "urn:firmwaresight:schema:diff:1";

/// The schema major of this document. A semantics change must bump it (`04_TECH/26`).
pub const DIFF_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffResultDto {
    pub schema: &'static str,
    pub schema_version: u32,
    /// The old build. A diff always names which side is which; two sides are never symmetric.
    pub base: DiffSideDto,
    /// The new build.
    pub target: DiffSideDto,
    pub memory: MemoryDiffDto,
    pub counts: ChangeCountsDto,
    pub unchanged: UnchangedCountsDto,
    pub section_changes: Vec<SectionChangeDto>,
    pub symbol_changes: Vec<SymbolChangeDto>,
    pub object_changes: ObjectChangesDto,
    pub warnings: Vec<DiffWarningDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffSideDto {
    pub snapshot_id: String,
    pub artifact: DiffArtifactDto,
    pub memory: DiffSideMemoryDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffArtifactDto {
    pub file_name: String,
    pub sha256: String,
    pub byte_size: u64,
}

/// One side's own two budgets, each with the state it was accounted under.
///
/// The state matters independently of the pair's comparability: `partial` means the number is a
/// floor, and a floor against an exact total is not an exact-against-exact comparison.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffSideMemoryDto {
    /// False when this build has no stored footprint row. That is not a build of zero size.
    pub footprint_row_present: bool,
    pub nonvolatile: SideBudgetDto,
    pub runtime_ram: SideBudgetDto,
    /// Device metadata bytes held out of both budgets, `null` when there is no row to say so.
    pub excluded_metadata_bytes: Option<u64>,
    pub evidence: SideEvidenceDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideBudgetDto {
    /// `exact`, `partial` or `unknown`.
    pub state: &'static str,
    /// Evidence class implied by the state, spelled out so a reader need not infer it.
    pub classification: &'static str,
    /// `null` when the state is `unknown`: the budget exists as a question, not as a number.
    pub bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SideEvidenceDto {
    /// Whether a supported GNU ld MAP is stored for this build. Never inferred.
    pub map_backed: bool,
    pub layout_source: String,
    pub weakest_evidence_basis: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryDiffDto {
    pub nonvolatile: ByteChangeDto,
    pub runtime_ram: ByteChangeDto,
    /// The weaker side caps the whole pair.
    pub comparability: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ByteChangeDto {
    /// `null` for an addition: the base side held nothing, which is not the same as holding zero.
    pub base: Option<u64>,
    /// `null` for a removal, for the same reason.
    pub target: Option<u64>,
    /// `delta = target - base`, signed, present only when both sides carry a number that fits.
    pub delta: Option<i64>,
    pub comparability: &'static str,
    /// Why there is no delta, or why the delta is weaker than it looks.
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeCountsDto {
    pub sections_added: usize,
    pub sections_removed: usize,
    pub sections_changed: usize,
    pub sections_ambiguous: usize,
    pub symbols_added: usize,
    pub symbols_removed: usize,
    pub symbols_changed: usize,
    pub symbols_ambiguous: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnchangedCountsDto {
    pub sections: usize,
    pub symbols: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionChangeDto {
    /// Diff vocabulary: `Added` / `Changed` / `Removed`. Never a Gate state.
    pub change: &'static str,
    pub key: String,
    pub name_known: bool,
    pub ambiguous: bool,
    pub base: Option<SectionRowDto>,
    pub target: Option<SectionRowDto>,
    pub file_size: ByteChangeDto,
    pub memory_size: ByteChangeDto,
    pub differing_fields: Vec<String>,
    pub indeterminate_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionRowDto {
    /// Row position within this build. Not an identity across builds.
    pub index: i64,
    pub name: Option<String>,
    pub name_unknown_reason: Option<String>,
    pub role: String,
    pub alloc: bool,
    pub write: bool,
    pub execute: bool,
    /// Hex text, because a firmware reader checks an address against a linker script by eye.
    pub virtual_address: Option<String>,
    pub virtual_address_unknown_reason: Option<String>,
    pub load_address: Option<String>,
    pub load_address_unknown_reason: Option<String>,
    /// `null` means no offset was recorded; storage keeps no reason column for it.
    pub file_offset: Option<u64>,
    pub file_size: u64,
    pub memory_size: Option<u64>,
    pub memory_size_unknown_reason: Option<String>,
    pub region: Option<String>,
    pub region_unknown_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolChangeDto {
    pub change: &'static str,
    pub name: String,
    pub name_known: bool,
    pub ambiguous: bool,
    pub kind: String,
    pub binding: String,
    pub base: Option<SymbolRowDto>,
    pub target: Option<SymbolRowDto>,
    pub size: ByteChangeDto,
    pub differing_fields: Vec<String>,
    pub indeterminate_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolRowDto {
    pub ordinal: i64,
    pub name: Option<String>,
    pub name_unknown_reason: Option<String>,
    /// `null` means no address was recorded; storage keeps no reason column for it.
    pub address: Option<String>,
    pub size: Option<u64>,
    pub size_unknown_reason: Option<String>,
    pub kind: String,
    pub binding: String,
    pub section_ref: String,
}

/// Whether a change can be attributed to an object file or module.
///
/// P2 reports the absence rather than a table: the persisted section and symbol records carry no
/// object-file field, so there is nothing on either side to attribute. `01_PRODUCT/01_PRD_MVP.md`
/// P0-3 asks for this delta only when evidence is sufficient, and here it is not.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectChangesDto {
    pub available: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffWarningDto {
    pub code: String,
    pub message: String,
}

fn hex(value: u64) -> String {
    format!("{value:#x}")
}

fn fact_text<T: Clone>(fact: &Fact<T>) -> (Option<T>, Option<String>) {
    match fact {
        Fact::Known(value) => (Some(value.clone()), None),
        Fact::Unknown { reason } => (None, Some(reason.clone())),
    }
}

fn fact_hex(fact: &Fact<u64>) -> (Option<String>, Option<String>) {
    match fact {
        Fact::Known(value) => (Some(hex(*value)), None),
        Fact::Unknown { reason } => (None, Some(reason.clone())),
    }
}

fn byte_change(change: &ByteChange) -> ByteChangeDto {
    ByteChangeDto {
        base: change.base,
        target: change.target,
        delta: change.delta,
        comparability: change.comparability.label(),
        reason: change.reason.clone(),
    }
}

fn side_memory(side: &DiffSideMemory) -> DiffSideMemoryDto {
    DiffSideMemoryDto {
        footprint_row_present: side.footprint_row_present,
        nonvolatile: side_budget(side.nonvolatile),
        runtime_ram: side_budget(side.runtime_ram),
        excluded_metadata_bytes: side.excluded_metadata_bytes,
        evidence: SideEvidenceDto {
            map_backed: side.evidence.map_backed,
            layout_source: side.evidence.layout_source.clone(),
            weakest_evidence_basis: side.evidence.weakest_basis.clone(),
        },
    }
}

fn side_budget(budget: DiffBudget) -> SideBudgetDto {
    SideBudgetDto {
        state: budget.state.as_str(),
        // Exact is the only state that says every byte is accounted for; a partial total is a
        // floor we derived, and an unknown one is a question. Mirrors `ByteTotal::classification`.
        classification: match budget.state {
            BudgetState::Exact => "observed",
            BudgetState::Partial => "derived",
            BudgetState::Unknown => "unknown",
        },
        bytes: budget.bytes,
    }
}

fn artifact(identity: &DiffArtifact) -> DiffArtifactDto {
    DiffArtifactDto {
        file_name: identity.file_name.clone(),
        sha256: identity.sha256.clone(),
        byte_size: identity.byte_size,
    }
}

fn section_row(row: &DiffSection) -> SectionRowDto {
    let (name, name_reason) = fact_text(&row.name);
    let (virtual_address, virtual_reason) = fact_hex(&row.virtual_address);
    let (load_address, load_reason) = fact_hex(&row.load_address);
    let (memory_size, memory_reason) = fact_text(&row.memory_size);
    let (region, region_reason) = fact_text(&row.region);

    SectionRowDto {
        index: row.index,
        name,
        name_unknown_reason: name_reason,
        role: row.role.clone(),
        alloc: row.alloc,
        write: row.write,
        execute: row.execute,
        virtual_address,
        virtual_address_unknown_reason: virtual_reason,
        load_address,
        load_address_unknown_reason: load_reason,
        file_offset: row.file_offset,
        file_size: row.file_size,
        memory_size,
        memory_size_unknown_reason: memory_reason,
        region,
        region_unknown_reason: region_reason,
    }
}

fn section_change(row: &SectionChange) -> SectionChangeDto {
    SectionChangeDto {
        change: row.change.label(),
        key: row.key.clone(),
        name_known: row.name_known,
        ambiguous: row.ambiguous,
        base: row.base.as_ref().map(section_row),
        target: row.target.as_ref().map(section_row),
        file_size: byte_change(&row.file_size),
        memory_size: byte_change(&row.memory_size),
        differing_fields: row.differing_fields.clone(),
        indeterminate_fields: row.indeterminate_fields.clone(),
    }
}

fn symbol_row(row: &DiffSymbol) -> SymbolRowDto {
    let (name, name_reason) = fact_text(&row.name);
    let (size, size_reason) = fact_text(&row.size);

    SymbolRowDto {
        ordinal: row.ordinal,
        name,
        name_unknown_reason: name_reason,
        address: row.address.map(hex),
        size,
        size_unknown_reason: size_reason,
        kind: row.kind.clone(),
        binding: row.binding.clone(),
        section_ref: row.section_ref.clone(),
    }
}

fn symbol_change(row: &SymbolChange) -> SymbolChangeDto {
    SymbolChangeDto {
        change: row.change.label(),
        name: row.name.clone(),
        name_known: row.name_known,
        ambiguous: row.ambiguous,
        kind: row.kind.clone(),
        binding: row.binding.clone(),
        base: row.base.as_ref().map(symbol_row),
        target: row.target.as_ref().map(symbol_row),
        size: byte_change(&row.size),
        differing_fields: row.differing_fields.clone(),
        indeterminate_fields: row.indeterminate_fields.clone(),
    }
}

fn counts(values: &ChangeCounts) -> ChangeCountsDto {
    ChangeCountsDto {
        sections_added: values.sections_added,
        sections_removed: values.sections_removed,
        sections_changed: values.sections_changed,
        sections_ambiguous: values.sections_ambiguous,
        symbols_added: values.symbols_added,
        symbols_removed: values.symbols_removed,
        symbols_changed: values.symbols_changed,
        symbols_ambiguous: values.symbols_ambiguous,
    }
}

impl DiffResultDto {
    /// Project a Core result. This is the only construction path, so no caller can publish a diff
    /// that Core did not compute.
    #[must_use]
    pub fn from_diff(result: &DiffResult) -> Self {
        Self {
            schema: DIFF_SCHEMA_ID,
            schema_version: DIFF_SCHEMA_VERSION,
            base: DiffSideDto {
                snapshot_id: result.base_snapshot_id.clone(),
                artifact: artifact(&result.base_artifact),
                memory: side_memory(&result.memory.base),
            },
            target: DiffSideDto {
                snapshot_id: result.target_snapshot_id.clone(),
                artifact: artifact(&result.target_artifact),
                memory: side_memory(&result.memory.target),
            },
            memory: MemoryDiffDto {
                nonvolatile: byte_change(&result.memory.nonvolatile),
                runtime_ram: byte_change(&result.memory.runtime_ram),
                comparability: result.memory.comparability.label(),
            },
            counts: counts(&result.counts),
            unchanged: UnchangedCountsDto {
                sections: result.unchanged.sections,
                symbols: result.unchanged.symbols,
            },
            section_changes: result.section_changes.iter().map(section_change).collect(),
            symbol_changes: result.symbol_changes.iter().map(symbol_change).collect(),
            object_changes: ObjectChangesDto {
                available: result.attribution.available,
                reason: result.attribution.reason.clone(),
            },
            warnings: result
                .warnings
                .iter()
                .map(|warning| DiffWarningDto {
                    code: warning.code.clone(),
                    message: warning.message.clone(),
                })
                .collect(),
        }
    }
}
