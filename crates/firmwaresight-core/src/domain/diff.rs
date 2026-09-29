//! Deterministic build-to-build diff.
//!
//! `04_TECH/02_DOMAIN_MODEL.md` names `Diff` as a first-class entity with a base snapshot, a
//! target snapshot, totals and per-collection change lists, and states as an invariant that a diff
//! never mutates either snapshots (`:105`). This module is the single owner of those semantics:
//! matching, change kinds, delta arithmetic, comparability, ambiguity and ordering. Storage
//! retrieves persisted facts, and the shell, the report and the UI only project what is decided
//! here (AGENTS.md 3).
//!
//! Two rules shape everything below. Absence is not zero: a row that exists on one side is an
//! addition or a removal, never a change from or to `0` (`DESIGN.md` §5 Diff Row). And weaker
//! evidence never borrows the confidence of stronger evidence: an exact total compared against a
//! partial one yields a partial delta, not an exact one
//! (`04_TECH/23_MEMORY_ACCOUNTING_MODEL.md`).

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::domain::identity::Fact;

/// How a row appears in a diff. Rows that matched and differ in nothing verifiable are counted but
/// never listed, so a change table holds changes.
///
/// These words are diff vocabulary. They are not Gate states and must not be rendered as `PASS`,
/// `REVIEW` or `BLOCK`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChangeKind {
    Added,
    Changed,
    Removed,
}

impl ChangeKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::Changed => "Changed",
            Self::Removed => "Removed",
        }
    }
}

/// How much of a comparison actually rests on numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparability {
    /// Both sides carry an exactly accounted-for number.
    Exact,
    /// At least one side is a floor rather than a complete total.
    Partial,
    /// At least one side has no number, so no delta exists either.
    Unknown,
}

impl Comparability {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Partial => "partial",
            Self::Unknown => "unknown",
        }
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Exact => 0,
            Self::Partial => 1,
            Self::Unknown => 2,
        }
    }
}

/// The persisted state of one memory budget, as `memory_footprints` stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetState {
    Exact,
    Partial,
    Unknown,
}

impl BudgetState {
    /// Reads back the `exact` / `partial` / `unknown` check constraint. Anything else stays
    /// unknown rather than being guessed into a stronger class.
    #[must_use]
    pub fn parse(stored: &str) -> Self {
        match stored {
            "exact" => Self::Exact,
            "partial" => Self::Partial,
            _ => Self::Unknown,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Partial => "partial",
            Self::Unknown => "unknown",
        }
    }

    const fn comparability(self) -> Comparability {
        match self {
            Self::Exact => Comparability::Exact,
            Self::Partial => Comparability::Partial,
            Self::Unknown => Comparability::Unknown,
        }
    }
}

/// One numeric comparison, built so that absent, zero and unknown stay three different things.
///
/// `delta` exists only when both sides carry a number and the difference fits an `i64`. The
/// arithmetic is checked, so a pathological `u64` pair produces no delta instead of a wrap or a
/// panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteChange {
    pub base: Option<u64>,
    pub target: Option<u64>,
    pub delta: Option<i64>,
    pub comparability: Comparability,
    pub reason: Option<String>,
}

impl ByteChange {
    /// Compare two quantities of the same kind. `delta = target - base`, so positive means the
    /// target side is bigger.
    #[must_use]
    pub fn between(base: Option<u64>, target: Option<u64>, comparability: Comparability) -> Self {
        let delta = match (base, target) {
            (Some(from), Some(to)) => checked_delta(from, to),
            _ => None,
        };
        Self {
            base,
            target,
            delta,
            comparability,
            reason: change_reason(base, target, delta, comparability),
        }
    }

    /// A row that exists only on the target side. `0 -> n` would claim the base build held a zero
    /// sized thing, and it held nothing at all.
    #[must_use]
    pub fn added(bytes: Option<u64>) -> Self {
        Self {
            base: None,
            target: bytes,
            delta: None,
            comparability: Comparability::Unknown,
            reason: Some(
                "present only in the target build: an addition is not a change from zero"
                    .to_owned(),
            ),
        }
    }

    /// The mirror image of [`ByteChange::added`].
    #[must_use]
    pub fn removed(bytes: Option<u64>) -> Self {
        Self {
            base: bytes,
            target: None,
            delta: None,
            comparability: Comparability::Unknown,
            reason: Some(
                "present only in the base build: a removal is not a change to zero".to_owned(),
            ),
        }
    }

    /// A quantity that exists on both sides but has no number on at least one of them.
    #[must_use]
    pub fn unknown(base: Option<u64>, target: Option<u64>, reason: impl Into<String>) -> Self {
        Self {
            base,
            target,
            delta: None,
            comparability: Comparability::Unknown,
            reason: Some(reason.into()),
        }
    }

    #[must_use]
    pub fn has_delta(&self) -> bool {
        self.delta.is_some()
    }
}

fn checked_delta(base: u64, target: u64) -> Option<i64> {
    i64::try_from(i128::from(target) - i128::from(base)).ok()
}

fn change_reason(
    base: Option<u64>,
    target: Option<u64>,
    delta: Option<i64>,
    comparability: Comparability,
) -> Option<String> {
    if base.is_none() || target.is_none() {
        let missing = if base.is_none() { "base" } else { "target" };
        return Some(format!(
            "no {missing} number is recorded, so no delta exists; that absence is not a zero"
        ));
    }
    if delta.is_none() {
        return Some("the difference does not fit a signed 64-bit count".to_owned());
    }
    match comparability {
        Comparability::Exact => None,
        Comparability::Partial => Some(
            "at least one side is a partially accounted total, so this delta is a floor rather than an exact figure"
                .to_owned(),
        ),
        Comparability::Unknown => Some("at least one side is unknown".to_owned()),
    }
}

/// The persisted identity of the primary artifact behind one side of a comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffArtifact {
    /// File name only: a host path never enters a diff, an export or the UI (AGENTS.md 7).
    pub file_name: String,
    pub sha256: String,
    pub byte_size: u64,
}

/// One budget's persisted total and how completely it is accounted for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffBudget {
    pub state: BudgetState,
    pub bytes: Option<u64>,
}

impl DiffBudget {
    #[must_use]
    pub fn unknown_total() -> Self {
        Self {
            state: BudgetState::Unknown,
            bytes: None,
        }
    }
}

/// How strongly one side's memory numbers are evidenced, read back from stored facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideEvidence {
    /// Whether a supported GNU ld MAP is stored as an artifact of this build.
    pub map_backed: bool,
    /// The persisted layout source label, for example `map`.
    pub layout_source: String,
    /// The persisted weakest accounting basis, for example `MapRegionAndElfLoad`.
    pub weakest_basis: Option<String>,
}

/// One side's persisted memory footprint, in the two budgets ADR-0021 keeps separate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffMemory {
    pub nonvolatile: DiffBudget,
    pub runtime_ram: DiffBudget,
    pub excluded_metadata_bytes: u64,
    pub evidence: SideEvidence,
}

/// One side's memory totals as the diff reports them.
///
/// Prompt §14 requires each side to keep its own exact / partial / unknown state: a floor of 100
/// bytes against an exact 100 must not read as exact against exact. A side with no stored footprint
/// row is [`DiffSideMemory::absent`], which is still not the same claim as a row that says zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSideMemory {
    /// False when this build has no stored footprint row at all.
    pub footprint_row_present: bool,
    pub nonvolatile: DiffBudget,
    pub runtime_ram: DiffBudget,
    /// Device metadata excluded from both budgets on this side, absent when there is no row.
    pub excluded_metadata_bytes: Option<u64>,
    pub evidence: SideEvidence,
}

impl DiffSideMemory {
    #[must_use]
    pub fn from_memory(memory: &DiffMemory) -> Self {
        Self {
            footprint_row_present: true,
            nonvolatile: memory.nonvolatile,
            runtime_ram: memory.runtime_ram,
            excluded_metadata_bytes: Some(memory.excluded_metadata_bytes),
            evidence: memory.evidence.clone(),
        }
    }

    /// The side that has no footprint row: unknown totals, and an evidence record that says so.
    #[must_use]
    pub fn absent() -> Self {
        Self {
            footprint_row_present: false,
            nonvolatile: DiffBudget::unknown_total(),
            runtime_ram: DiffBudget::unknown_total(),
            excluded_metadata_bytes: None,
            evidence: SideEvidence {
                map_backed: false,
                layout_source: "absent".to_owned(),
                weakest_basis: None,
            },
        }
    }

    fn side(side: &DiffSnapshotInput) -> Self {
        match &side.memory {
            Some(memory) => Self::from_memory(memory),
            None => Self::absent(),
        }
    }
}

/// One persisted section, with every undetermined field still undetermined.
///
/// `file_offset` has no reason column in storage, so it stays an `Option` rather than a `Fact`:
/// the diff may say the offset is unknown, but must not dress up a reason nobody recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSection {
    /// Row position inside one build. Never an identity across builds.
    pub index: i64,
    pub name: Fact<String>,
    pub role: String,
    pub alloc: bool,
    pub write: bool,
    pub execute: bool,
    pub virtual_address: Fact<u64>,
    pub load_address: Fact<u64>,
    pub file_offset: Option<u64>,
    pub file_size: u64,
    pub memory_size: Fact<u64>,
    pub region: Fact<String>,
}

/// One persisted symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSymbol {
    /// Row position inside one build. Never an identity across builds.
    pub ordinal: i64,
    pub name: Fact<String>,
    /// No reason column exists for addresses either, so an unknown address stays an `Option`.
    pub address: Option<u64>,
    pub size: Fact<u64>,
    pub kind: String,
    pub binding: String,
    pub section_ref: String,
}

/// Everything one side of a comparison needs, and nothing else.
///
/// Deliberately narrower than `BuildSnapshot`: a stored snapshot has to stay comparable after the
/// original ELF and MAP have moved or been deleted, so this input is hydrated from persisted facts
/// and never re-reads a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSnapshotInput {
    pub snapshot_id: String,
    pub artifact: DiffArtifact,
    /// `None` when the build has no persisted footprint row, which is reported as unknown rather
    /// than as zero.
    pub memory: Option<DiffMemory>,
    pub sections: Vec<DiffSection>,
    pub symbols: Vec<DiffSymbol>,
}

impl DiffSnapshotInput {
    /// Project a freshly analyzed snapshot into a diff input.
    ///
    /// This is the CLI's path: `fwsight diff a.elf b.elf` compares what the parser just produced,
    /// while the desktop compares what storage kept. Both must hand the matcher the same shape and
    /// the same field labels, or the two surfaces would disagree about what changed. Role, kind,
    /// binding and section-reference labels therefore use the same `Debug` text the storage writer
    /// persists, and a footprint is read through the same three budget states.
    #[must_use]
    pub fn from_snapshot(snapshot: &crate::domain::build_snapshot::BuildSnapshot) -> Self {
        // A sealed snapshot carries at least its own artifact; the analyze projection states the same
        // invariant. Naming it here rather than falling back keeps an empty SHA-256 out of the
        // portable document, where it would fail `schemas/diff.schema.json` instead of naming a side.
        let primary = snapshot
            .primary_artifact()
            .expect("a sealed snapshot carries at least its own artifact");
        let map_backed = snapshot
            .artifacts()
            .iter()
            .any(|artifact| artifact.kind == crate::domain::identity::ArtifactKind::Map);

        Self {
            snapshot_id: snapshot.id().as_str().to_owned(),
            artifact: DiffArtifact {
                file_name: display_file_name(&primary.path),
                sha256: primary.sha256.hex().to_owned(),
                byte_size: primary.byte_size,
            },
            memory: snapshot.memory().map(|footprint| DiffMemory {
                nonvolatile: budget_from(&footprint.nonvolatile),
                runtime_ram: budget_from(&footprint.runtime_ram),
                excluded_metadata_bytes: footprint.excluded_metadata_bytes,
                evidence: SideEvidence {
                    map_backed,
                    layout_source: footprint.layout_source.as_label().to_owned(),
                    weakest_basis: footprint.weakest_basis.map(|basis| format!("{basis:?}")),
                },
            }),
            sections: snapshot
                .sections()
                .iter()
                .map(|section| diff_section(section, section.index as i64))
                .collect(),
            symbols: snapshot
                .symbols()
                .iter()
                .enumerate()
                .map(|(ordinal, symbol)| diff_symbol(symbol, ordinal as i64))
                .collect(),
        }
    }
}

/// The last component of a stored or supplied path, on either separator.
///
/// A path is an intake fact; a diff, an export and a candidate row are content that must not carry
/// a machine's directory layout (`AGENTS.md` 7). Storage and the CLI share this one rule so the two
/// surfaces cannot disagree about what a build is called.
#[must_use]
pub fn display_file_name(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}

fn budget_from(total: &crate::domain::memory::ByteTotal) -> DiffBudget {
    use crate::domain::memory::ByteTotal;
    let state = match total {
        ByteTotal::Exact { .. } => BudgetState::Exact,
        ByteTotal::Partial { .. } => BudgetState::Partial,
        ByteTotal::Unknown { .. } => BudgetState::Unknown,
    };
    DiffBudget {
        state,
        bytes: if state == BudgetState::Unknown {
            None
        } else {
            total.bytes()
        },
    }
}

fn diff_section(section: &crate::domain::section::Section, index: i64) -> DiffSection {
    DiffSection {
        index,
        name: section.name.clone(),
        role: format!("{:?}", section.role),
        alloc: section.flags.alloc,
        write: section.flags.write,
        execute: section.flags.execute,
        virtual_address: section.virtual_address.clone(),
        load_address: section.load_address.clone(),
        file_offset: section.file_offset.value().copied(),
        file_size: section.file_size,
        memory_size: section.memory_size.clone(),
        region: section.region.clone(),
    }
}

fn diff_symbol(symbol: &crate::domain::symbol::Symbol, ordinal: i64) -> DiffSymbol {
    DiffSymbol {
        // A snapshot keeps symbols in parser order, so this is that order's position: a row
        // position used only for stable output ordering, never as an identity.
        ordinal,
        name: symbol.name.clone(),
        address: symbol.address.value().copied(),
        size: symbol.size.clone(),
        kind: format!("{:?}", symbol.kind),
        binding: format!("{:?}", symbol.binding),
        section_ref: format!("{:?}", symbol.section),
    }
}

/// Whether object or module attribution can be compared.
///
/// `symbols` persists name, address, size, kind, binding and section reference, and `sections`
/// persists name, role, flags, addresses, sizes and region. Neither carries an object-file or
/// module field, so there is nothing on either side to attribute a change to and the honest answer
/// is that attribution is unavailable. `01_PRODUCT/01_PRD_MVP.md` P0-3 asks for this delta only
/// "when evidence is sufficient".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectAttribution {
    pub available: bool,
    pub reason: String,
}

impl ObjectAttribution {
    #[must_use]
    pub fn from_inputs(_base: &DiffSnapshotInput, _target: &DiffSnapshotInput) -> Self {
        Self {
            available: false,
            reason: "the persisted symbol and section records carry no object-file or module attribution, so no object or module delta is claimable for either build".to_owned(),
        }
    }
}

/// A section row in the diff, with both sides visible wherever both exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionChange {
    pub change: ChangeKind,
    /// Stable presentation key: the section name when known, otherwise its row position.
    pub key: String,
    /// False when the name itself is unknown, which is why the row could not be matched.
    pub name_known: bool,
    /// True when the row was left unpaired because its name occurs more than once on one or both
    /// sides. Pairing such rows by position would invent a change no evidence supports.
    pub ambiguous: bool,
    pub base: Option<DiffSection>,
    pub target: Option<DiffSection>,
    pub file_size: ByteChange,
    pub memory_size: ByteChange,
    /// Fields that verifiably differ. A field unknown on one side is recorded under
    /// `indeterminate_fields` instead of being claimed as a change.
    pub differing_fields: Vec<String>,
    pub indeterminate_fields: Vec<String>,
}

/// A symbol row in the diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolChange {
    pub change: ChangeKind,
    pub name: String,
    pub name_known: bool,
    pub ambiguous: bool,
    pub kind: String,
    pub binding: String,
    pub base: Option<DiffSymbol>,
    pub target: Option<DiffSymbol>,
    pub size: ByteChange,
    pub differing_fields: Vec<String>,
    pub indeterminate_fields: Vec<String>,
}

/// The two budget deltas, plus what caps their confidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryDiff {
    pub nonvolatile: ByteChange,
    pub runtime_ram: ByteChange,
    /// The weaker side decides: partial against exact is never presented as exact against exact.
    pub comparability: Comparability,
    /// Each side as it was recorded, state included, so the two floors stay visible.
    pub base: DiffSideMemory,
    pub target: DiffSideMemory,
    pub evidence_warning: Option<String>,
}

/// How many rows fell into each change kind, plus what could not be matched.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChangeCounts {
    pub sections_added: usize,
    pub sections_removed: usize,
    pub sections_changed: usize,
    pub sections_ambiguous: usize,
    pub symbols_added: usize,
    pub symbols_removed: usize,
    pub symbols_changed: usize,
    pub symbols_ambiguous: usize,
}

/// What was found identical. Kept as a count so "nothing changed" stays a measured claim.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnchangedCounts {
    pub sections: usize,
    pub symbols: usize,
}

/// A factual note a reader needs in order to interpret the tables correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffWarning {
    pub code: String,
    pub message: String,
}

/// The whole comparison of two persisted snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    pub base_snapshot_id: String,
    pub target_snapshot_id: String,
    pub base_artifact: DiffArtifact,
    pub target_artifact: DiffArtifact,
    pub memory: MemoryDiff,
    pub section_changes: Vec<SectionChange>,
    pub symbol_changes: Vec<SymbolChange>,
    pub counts: ChangeCounts,
    pub unchanged: UnchangedCounts,
    pub attribution: ObjectAttribution,
    pub warnings: Vec<DiffWarning>,
}

/// The ways a comparison is refused rather than answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffError {
    /// Both sides name the same snapshot. There is no diff of a build against itself to show, and
    /// answering with empty tables would read as "nothing changed" instead of "nothing compared".
    SameSnapshot { snapshot_id: String },
}

impl DiffError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::SameSnapshot { .. } => "ERR-DIFF-5001",
        }
    }
}

/// Compare two persisted snapshots. Both inputs are shared: a diff never mutates a snapshot.
pub fn compare(
    base: &DiffSnapshotInput,
    target: &DiffSnapshotInput,
) -> Result<DiffResult, DiffError> {
    if base.snapshot_id == target.snapshot_id {
        return Err(DiffError::SameSnapshot {
            snapshot_id: base.snapshot_id.clone(),
        });
    }

    let (section_changes, sections_unchanged) = diff_sections(base, target);
    let (symbol_changes, symbols_unchanged) = diff_symbols(base, target);
    let counts = count_changes(&section_changes, &symbol_changes);

    let memory = diff_memory(base, target);
    let warnings = build_warnings(&memory, &counts, &section_changes);

    Ok(DiffResult {
        base_snapshot_id: base.snapshot_id.clone(),
        target_snapshot_id: target.snapshot_id.clone(),
        base_artifact: base.artifact.clone(),
        target_artifact: target.artifact.clone(),
        memory,
        section_changes,
        symbol_changes,
        counts,
        unchanged: UnchangedCounts {
            sections: sections_unchanged,
            symbols: symbols_unchanged,
        },
        attribution: ObjectAttribution::from_inputs(base, target),
        warnings,
    })
}

fn build_warnings(
    memory: &MemoryDiff,
    counts: &ChangeCounts,
    sections: &[SectionChange],
) -> Vec<DiffWarning> {
    let mut warnings = Vec::new();

    if let Some(message) = &memory.evidence_warning {
        warnings.push(DiffWarning {
            code: "MAP-EVIDENCE".to_owned(),
            message: message.clone(),
        });
    }
    if counts.sections_ambiguous > 0 {
        let names = sections
            .iter()
            .filter(|row| row.ambiguous)
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        warnings.push(DiffWarning {
            code: "SECTION-AMBIGUOUS".to_owned(),
            message: format!(
                "{} section row(s) repeat a name on one or both sides and were left unpaired rather than matched by position: {names}",
                counts.sections_ambiguous
            ),
        });
    }
    if counts.symbols_ambiguous > 0 {
        warnings.push(DiffWarning {
            code: "SYMBOL-AMBIGUOUS".to_owned(),
            message: format!(
                "{} symbol row(s) repeat a name, kind and binding combination on one or both sides and were left unpaired; a row position is not an identity",
                counts.symbols_ambiguous
            ),
        });
    }
    if counts.sections_added
        + counts.sections_removed
        + counts.sections_changed
        + counts.symbols_added
        + counts.symbols_removed
        + counts.symbols_changed
        == 0
    {
        warnings.push(DiffWarning {
            code: "NO-CHANGES".to_owned(),
            message: "no section or symbol row differs between these two snapshots".to_owned(),
        });
    }
    warnings
}

fn count_changes(sections: &[SectionChange], symbols: &[SymbolChange]) -> ChangeCounts {
    let mut counts = ChangeCounts::default();
    for row in sections {
        match row.change {
            ChangeKind::Added => counts.sections_added += 1,
            ChangeKind::Removed => counts.sections_removed += 1,
            ChangeKind::Changed => counts.sections_changed += 1,
        }
        if row.ambiguous {
            counts.sections_ambiguous += 1;
        }
    }
    for row in symbols {
        match row.change {
            ChangeKind::Added => counts.symbols_added += 1,
            ChangeKind::Removed => counts.symbols_removed += 1,
            ChangeKind::Changed => counts.symbols_changed += 1,
        }
        if row.ambiguous {
            counts.symbols_ambiguous += 1;
        }
    }
    counts
}

/// A section's matching key. An unnamed section has none, because a match invented from a row
/// position is not a fact.
fn section_key(section: &DiffSection) -> Option<&str> {
    section.name.value().map(String::as_str)
}

fn diff_sections(
    base: &DiffSnapshotInput,
    target: &DiffSnapshotInput,
) -> (Vec<SectionChange>, usize) {
    let base_groups = group_by(&base.sections, section_key);
    let target_groups = group_by(&target.sections, section_key);

    let mut rows = Vec::new();
    let mut unchanged = 0_usize;

    for (name, base_rows) in &base_groups {
        let target_rows = target_groups.get(name).map_or(&[][..], Vec::as_slice);
        match (base_rows.len(), target_rows.len()) {
            (1, 1) => {
                let row = section_pair(&base_rows[0], &target_rows[0]);
                if row.differing_fields.is_empty() && row.indeterminate_fields.is_empty() {
                    unchanged += 1;
                } else {
                    rows.push(row);
                }
            }
            // One name, one row, only on the base side: that is a clean removal (§10), and calling
            // it ambiguous would tell the reader the name repeats when it does not.
            (1, 0) => rows.push(removed_section(&base_rows[0], false)),
            _ => {
                for section in base_rows {
                    rows.push(removed_section(section, true));
                }
                for section in target_rows {
                    rows.push(added_section(section, true));
                }
            }
        }
    }

    for (name, target_rows) in &target_groups {
        if base_groups.contains_key(name) {
            continue;
        }
        for section in target_rows {
            rows.push(added_section(section, false));
        }
    }

    // Rows with no name cannot be matched at all. They are reported unpaired and marked ambiguous.
    for section in base.sections.iter().filter(|s| section_key(s).is_none()) {
        rows.push(removed_section(section, true));
    }
    for section in target.sections.iter().filter(|s| section_key(s).is_none()) {
        rows.push(added_section(section, true));
    }

    sort_sections(&mut rows);
    (rows, unchanged)
}

fn group_by<T, K>(rows: &[T], key: K) -> BTreeMap<String, Vec<T>>
where
    T: Clone,
    K: Fn(&T) -> Option<&str>,
{
    let mut groups: BTreeMap<String, Vec<T>> = BTreeMap::new();
    for row in rows {
        if let Some(name) = key(row) {
            groups.entry(name.to_owned()).or_default().push(row.clone());
        }
    }
    groups
}

fn section_pair(base: &DiffSection, target: &DiffSection) -> SectionChange {
    let (differing, indeterminate) = compare_section_fields(base, target);
    SectionChange {
        change: ChangeKind::Changed,
        key: base.name.value().cloned().unwrap_or_default(),
        name_known: true,
        ambiguous: false,
        base: Some(base.clone()),
        target: Some(target.clone()),
        file_size: ByteChange::between(
            Some(base.file_size),
            Some(target.file_size),
            Comparability::Exact,
        ),
        memory_size: match (base.memory_size.value(), target.memory_size.value()) {
            (Some(from), Some(to)) => {
                ByteChange::between(Some(*from), Some(*to), Comparability::Exact)
            }
            (from, to) => ByteChange::unknown(
                from.copied(),
                to.copied(),
                "at least one side records no runtime size for this section",
            ),
        },
        differing_fields: differing,
        indeterminate_fields: indeterminate,
    }
}

fn compare_section_fields(base: &DiffSection, target: &DiffSection) -> (Vec<String>, Vec<String>) {
    let mut differing = Vec::new();
    let mut indeterminate = Vec::new();

    if base.role != target.role {
        differing.push("role".to_owned());
    }
    compare_values(
        "virtual_address",
        base.virtual_address.value(),
        target.virtual_address.value(),
        &mut differing,
        &mut indeterminate,
    );
    compare_values(
        "load_address",
        base.load_address.value(),
        target.load_address.value(),
        &mut differing,
        &mut indeterminate,
    );
    compare_values(
        "file_offset",
        base.file_offset,
        target.file_offset,
        &mut differing,
        &mut indeterminate,
    );
    compare_values(
        "region",
        base.region.value(),
        target.region.value(),
        &mut differing,
        &mut indeterminate,
    );
    if base.alloc != target.alloc {
        differing.push("alloc".to_owned());
    }
    if base.write != target.write {
        differing.push("write".to_owned());
    }
    if base.execute != target.execute {
        differing.push("execute".to_owned());
    }
    if base.file_size != target.file_size {
        differing.push("file_size".to_owned());
    }
    compare_values(
        "memory_size",
        base.memory_size.value().copied(),
        target.memory_size.value().copied(),
        &mut differing,
        &mut indeterminate,
    );

    (differing, indeterminate)
}

/// Two optional values of the same field. Differing only when both are known; one side unknown is
/// missing evidence, not a change.
fn compare_values<T: PartialEq + Copy>(
    name: &str,
    base: Option<T>,
    target: Option<T>,
    differing: &mut Vec<String>,
    indeterminate: &mut Vec<String>,
) {
    match (base, target) {
        (Some(from), Some(to)) => {
            if from != to {
                differing.push(name.to_owned());
            }
        }
        (None, None) => {}
        _ => indeterminate.push(name.to_owned()),
    }
}

fn added_section(section: &DiffSection, ambiguous: bool) -> SectionChange {
    unpaired_section(ChangeKind::Added, section, ambiguous)
}

fn removed_section(section: &DiffSection, ambiguous: bool) -> SectionChange {
    unpaired_section(ChangeKind::Removed, section, ambiguous)
}

fn unpaired_section(change: ChangeKind, section: &DiffSection, ambiguous: bool) -> SectionChange {
    let bytes = section.memory_size.value().copied();
    let size = match change {
        ChangeKind::Added => ByteChange::added(Some(section.file_size)),
        ChangeKind::Removed => ByteChange::removed(Some(section.file_size)),
        ChangeKind::Changed => ByteChange::between(None, None, Comparability::Unknown),
    };
    let memory_size = match change {
        ChangeKind::Added => ByteChange::added(bytes),
        ChangeKind::Removed => ByteChange::removed(bytes),
        ChangeKind::Changed => ByteChange::between(None, None, Comparability::Unknown),
    };
    SectionChange {
        change,
        key: section
            .name
            .value()
            .cloned()
            .unwrap_or_else(|| format!("unnamed section {}", section.index)),
        name_known: section.name.is_known(),
        ambiguous,
        file_size: size,
        memory_size,
        base: (change == ChangeKind::Removed).then(|| section.clone()),
        target: (change == ChangeKind::Added).then(|| section.clone()),
        differing_fields: Vec::new(),
        indeterminate_fields: Vec::new(),
    }
}

fn sort_sections(rows: &mut [SectionChange]) {
    // Total order: key, then change kind, then each side's row position. No hashmap iteration
    // order and no tie left to chance, so two runs of the same inputs produce equal output.
    rows.sort_by(|a, b| {
        a.key
            .cmp(&b.key)
            .then_with(|| a.change.cmp(&b.change))
            .then_with(|| row_index(&a.base).cmp(&row_index(&b.base)))
            .then_with(|| row_index(&a.target).cmp(&row_index(&b.target)))
    });
}

fn row_index(section: &Option<DiffSection>) -> i64 {
    section.as_ref().map_or(i64::MIN, |row| row.index)
}

fn symbol_ordinal(symbol: &Option<DiffSymbol>) -> i64 {
    symbol.as_ref().map_or(i64::MIN, |row| row.ordinal)
}

/// A symbol's matching key: known name plus kind plus binding. Address is never part of it,
/// because a moved address is itself a change the diff has to be able to report.
fn symbol_key(symbol: &DiffSymbol) -> Option<(String, String, String)> {
    symbol
        .name
        .value()
        .map(|name| (name.clone(), symbol.kind.clone(), symbol.binding.clone()))
}

fn group_symbols(rows: &[DiffSymbol]) -> BTreeMap<(String, String, String), Vec<DiffSymbol>> {
    let mut groups: BTreeMap<(String, String, String), Vec<DiffSymbol>> = BTreeMap::new();
    for row in rows {
        if let Some(key) = symbol_key(row) {
            groups.entry(key).or_default().push(row.clone());
        }
    }
    groups
}

fn diff_symbols(
    base: &DiffSnapshotInput,
    target: &DiffSnapshotInput,
) -> (Vec<SymbolChange>, usize) {
    let base_groups = group_symbols(&base.symbols);
    let target_groups = group_symbols(&target.symbols);

    let mut rows = Vec::new();
    let mut unchanged = 0_usize;

    for (key, base_rows) in &base_groups {
        let target_rows = target_groups.get(key).map_or(&[][..], Vec::as_slice);
        match (base_rows.len(), target_rows.len()) {
            (1, 1) => {
                let row = symbol_pair(&base_rows[0], &target_rows[0]);
                if row.differing_fields.is_empty() && row.indeterminate_fields.is_empty() {
                    unchanged += 1;
                } else {
                    rows.push(row);
                }
            }
            // A key that occurs once, on one side only, is a clean removal (§11). Ambiguity is
            // reserved for a key that repeats, because that is the case with no honest pairing.
            (1, 0) => rows.push(unpaired_symbol(ChangeKind::Removed, &base_rows[0], false)),
            _ => {
                for symbol in base_rows {
                    rows.push(unpaired_symbol(ChangeKind::Removed, symbol, true));
                }
                for symbol in target_rows {
                    rows.push(unpaired_symbol(ChangeKind::Added, symbol, true));
                }
            }
        }
    }

    for (key, target_rows) in &target_groups {
        if base_groups.contains_key(key) {
            continue;
        }
        for symbol in target_rows {
            rows.push(unpaired_symbol(ChangeKind::Added, symbol, false));
        }
    }

    for symbol in base.symbols.iter().filter(|s| symbol_key(s).is_none()) {
        rows.push(unpaired_symbol(ChangeKind::Removed, symbol, true));
    }
    for symbol in target.symbols.iter().filter(|s| symbol_key(s).is_none()) {
        rows.push(unpaired_symbol(ChangeKind::Added, symbol, true));
    }

    sort_symbols(&mut rows);
    (rows, unchanged)
}

fn symbol_pair(base: &DiffSymbol, target: &DiffSymbol) -> SymbolChange {
    let (differing, indeterminate) = compare_symbol_fields(base, target);
    SymbolChange {
        change: ChangeKind::Changed,
        name: base.name.value().cloned().unwrap_or_default(),
        name_known: true,
        ambiguous: false,
        kind: base.kind.clone(),
        binding: base.binding.clone(),
        base: Some(base.clone()),
        target: Some(target.clone()),
        size: match (base.size.value(), target.size.value()) {
            (Some(from), Some(to)) => {
                ByteChange::between(Some(*from), Some(*to), Comparability::Exact)
            }
            (from, to) => ByteChange::unknown(
                from.copied(),
                to.copied(),
                "at least one side records no size for this symbol",
            ),
        },
        differing_fields: differing,
        indeterminate_fields: indeterminate,
    }
}

fn compare_symbol_fields(base: &DiffSymbol, target: &DiffSymbol) -> (Vec<String>, Vec<String>) {
    let mut differing = Vec::new();
    let mut indeterminate = Vec::new();

    compare_values(
        "address",
        base.address,
        target.address,
        &mut differing,
        &mut indeterminate,
    );
    compare_values(
        "size",
        base.size.value().copied(),
        target.size.value().copied(),
        &mut differing,
        &mut indeterminate,
    );
    if base.section_ref != target.section_ref {
        differing.push("section".to_owned());
    }

    (differing, indeterminate)
}

fn unpaired_symbol(change: ChangeKind, symbol: &DiffSymbol, ambiguous: bool) -> SymbolChange {
    let bytes = symbol.size.value().copied();
    let size = match change {
        ChangeKind::Added => ByteChange::added(bytes),
        ChangeKind::Removed => ByteChange::removed(bytes),
        ChangeKind::Changed => ByteChange::between(None, None, Comparability::Unknown),
    };
    SymbolChange {
        change,
        name: symbol
            .name
            .value()
            .cloned()
            .unwrap_or_else(|| format!("unnamed symbol {}", symbol.ordinal)),
        name_known: symbol.name.is_known(),
        ambiguous,
        kind: symbol.kind.clone(),
        binding: symbol.binding.clone(),
        base: (change == ChangeKind::Removed).then(|| symbol.clone()),
        target: (change == ChangeKind::Added).then(|| symbol.clone()),
        size,
        differing_fields: Vec::new(),
        indeterminate_fields: Vec::new(),
    }
}

fn sort_symbols(rows: &mut [SymbolChange]) {
    rows.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.binding.cmp(&b.binding))
            .then_with(|| a.change.cmp(&b.change))
            .then_with(|| symbol_ordinal(&a.base).cmp(&symbol_ordinal(&b.base)))
            .then_with(|| symbol_ordinal(&a.target).cmp(&symbol_ordinal(&b.target)))
    });
}

fn diff_memory(base: &DiffSnapshotInput, target: &DiffSnapshotInput) -> MemoryDiff {
    let base_totals = DiffSideMemory::side(base);
    let target_totals = DiffSideMemory::side(target);

    let nonvolatile = budget_change(base_totals.nonvolatile, target_totals.nonvolatile);
    let runtime_ram = budget_change(base_totals.runtime_ram, target_totals.runtime_ram);

    let comparability = weakest_of(nonvolatile.comparability, runtime_ram.comparability);

    MemoryDiff {
        evidence_warning: evidence_warning(&base_totals.evidence, &target_totals.evidence),
        base: base_totals,
        target: target_totals,
        comparability,
        nonvolatile,
        runtime_ram,
    }
}

/// The degradation statement prompt §15 asks for: it names what is missing without ever claiming
/// a comparison is impossible when the numbers are actually there.
fn evidence_warning(base: &SideEvidence, target: &SideEvidence) -> Option<String> {
    if base.map_backed && target.map_backed {
        return None;
    }
    let missing = match (base.map_backed, target.map_backed) {
        (false, false) => "neither build has a stored GNU ld MAP",
        (false, true) => "the base build has no stored GNU ld MAP",
        (true, false) => "the target build has no stored GNU ld MAP",
        (true, true) => "both builds have a stored MAP",
    };
    Some(format!(
        "memory change is based on weaker layout evidence for one or both snapshots: {missing}, so the weaker side is charged from ELF section address and flags rather than from a linker region table"
    ))
}

fn budget_change(base: DiffBudget, target: DiffBudget) -> ByteChange {
    let comparability = weakest_of(base.state.comparability(), target.state.comparability());
    if base.bytes.is_none() || target.bytes.is_none() {
        return ByteChange::unknown(base.bytes, target.bytes, budget_reason(base, target));
    }
    ByteChange::between(base.bytes, target.bytes, comparability)
}

/// The weaker side always decides the comparison: exact against partial is reported as partial,
/// and anything against unknown is unknown.
fn weakest_of(a: Comparability, b: Comparability) -> Comparability {
    if a.rank() >= b.rank() { a } else { b }
}

fn budget_reason(base: DiffBudget, target: DiffBudget) -> String {
    fn describe(budget: DiffBudget) -> String {
        match (budget.bytes, budget.state) {
            (None, _) => "stores no number".to_owned(),
            (Some(_), BudgetState::Unknown) => "is recorded as unknown".to_owned(),
            (Some(_), BudgetState::Partial) => "is a partially accounted total".to_owned(),
            (Some(_), BudgetState::Exact) => "is an exact total".to_owned(),
        }
    }
    format!(
        "the base budget {} and the target budget {}, so no delta is claimed",
        describe(base),
        describe(target)
    )
}

/// One entry in a ranked list: which row, and the number that put it there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contributor {
    pub key: String,
    pub change: ChangeKind,
    pub delta: Option<i64>,
    pub bytes: Option<u64>,
}

impl DiffResult {
    /// Largest growth among rows that exist on both sides. A row that exists on one side only is
    /// not growth from zero, so it never appears here; it belongs in `largest_added_sections`.
    #[must_use]
    pub fn top_section_growth(&self, limit: usize) -> Vec<Contributor> {
        growth(
            self.section_changes.iter().filter_map(|row| {
                growth_entry(
                    &row.key,
                    row.change,
                    row.memory_size.delta,
                    row.target
                        .as_ref()
                        .and_then(|s| s.memory_size.value().copied()),
                )
            }),
            limit,
        )
    }

    #[must_use]
    pub fn top_symbol_growth(&self, limit: usize) -> Vec<Contributor> {
        growth(
            self.symbol_changes.iter().filter_map(|row| {
                growth_entry(
                    &row.name,
                    row.change,
                    row.size.delta,
                    row.target.as_ref().and_then(|s| s.size.value().copied()),
                )
            }),
            limit,
        )
    }

    #[must_use]
    pub fn largest_added_sections(&self, limit: usize) -> Vec<Contributor> {
        additions(
            self.section_changes
                .iter()
                .filter(|row| row.change == ChangeKind::Added)
                .map(|row| Contributor {
                    key: row.key.clone(),
                    change: ChangeKind::Added,
                    delta: None,
                    bytes: row
                        .target
                        .as_ref()
                        .and_then(|s| s.memory_size.value().copied()),
                }),
            limit,
        )
    }

    #[must_use]
    pub fn largest_added_symbols(&self, limit: usize) -> Vec<Contributor> {
        additions(
            self.symbol_changes
                .iter()
                .filter(|row| row.change == ChangeKind::Added)
                .map(|row| Contributor {
                    key: row.name.clone(),
                    change: ChangeKind::Added,
                    delta: None,
                    bytes: row.target.as_ref().and_then(|s| s.size.value().copied()),
                }),
            limit,
        )
    }
}

fn growth_entry(
    key: &str,
    change: ChangeKind,
    delta: Option<i64>,
    bytes: Option<u64>,
) -> Option<Contributor> {
    delta.filter(|delta| *delta > 0).map(|delta| Contributor {
        key: key.to_owned(),
        change,
        delta: Some(delta),
        bytes,
    })
}

fn growth<I>(entries: I, limit: usize) -> Vec<Contributor>
where
    I: Iterator<Item = Contributor>,
{
    let mut ranked: Vec<Contributor> = entries.collect();
    ranked.sort_by(|a, b| b.delta.cmp(&a.delta).then_with(|| a.key.cmp(&b.key)));
    ranked.truncate(limit);
    ranked
}

fn additions<I>(entries: I, limit: usize) -> Vec<Contributor>
where
    I: Iterator<Item = Contributor>,
{
    let mut ranked: Vec<Contributor> = entries.collect();
    ranked.sort_by(|a, b| by_size_desc(a.bytes, b.bytes).then_with(|| a.key.cmp(&b.key)));
    ranked.truncate(limit);
    ranked
}

/// Known sizes first, largest first; a row with no size is never treated as size zero.
fn by_size_desc(a: Option<u64>, b: Option<u64>) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => y.cmp(&x),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests;
