//! The P2 Compare surface: pick two stored builds, get a bounded summary, page through the changed
//! rows, and export the same facts the summary was computed from.
//!
//! Three rules shape this module.
//!
//! 1. **Diff semantics belong to Core.** Nothing here compares, subtracts or classifies. The rows come
//!    from `firmwaresight_core::domain::diff::compare`, and this layer only filters, orders and
//!    re-projects what Core already decided (`AGENTS.md` 3).
//! 2. **The whole diff never crosses the boundary.** A build can change thousands of symbols, so
//!    `compare_snapshots` returns a bounded summary plus a session-local `cmp-<pid>-<n>` handle, and
//!    rows are fetched one page at a time (prompt §20, §21). The handle is not persisted, not
//!    portable and not part of the diff's content identity; after a restart the same two snapshot ids
//!    recompute the same diff from SQLite.
//! 3. **Export is a Rust-side decision.** The save path comes from a native dialog the shell opened,
//!    never from the WebView, and it is not returned to the WebView. Cancel and "keep my existing
//!    file" are normal outcomes, not errors (prompt §35, §36).
//!
//! Ordering and filtering are presentation decisions over an already-computed list. Where they are not
//! obvious they are stated: a row with no number for the field being sorted on has no place in the
//! value ordering, so it stays behind every measured row whichever way the column points.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use firmwaresight_core::domain::diff::{
    ByteChange, ChangeKind, Contributor, DiffArtifact, DiffError, DiffResult, DiffSection,
    DiffSideMemory, DiffSymbol, SectionChange, SymbolChange, compare,
};
use firmwaresight_report::{DiffResultDto, diff_render};
use firmwaresight_storage::{
    CandidateQuery, CompareCandidate, DEFAULT_CANDIDATE_LIMIT, MAX_CANDIDATE_LIMIT, StoredBudget,
};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::ipc::{
    ByteDeltaDto, CandidatePageDto, CandidatePageRequestDto, ChangeKindCountsDto,
    ChangeKindFilterDto, CompareCandidateDto, CompareRequestDto, CompareSummaryDto, ContributorDto,
    DiffCountsDto, DiffMemoryDto, DiffSideDto, DiffSideMemoryDto, DiffWarningDto, ErrorEnvelopeDto,
    ExportOutcomeDto, ObjectAttributionDto, SectionChangePageDto, SectionChangeQueryDto,
    SectionChangeRowDto, SectionChangeSortDto, SectionSideDto, SortDirDto, StoredBudgetDto,
    SymbolChangePageDto, SymbolChangeQueryDto, SymbolChangeRowDto, SymbolChangeSortDto,
    SymbolSideDto,
};
use crate::{
    LOCAL_PROJECT_ID, Session, envelope_from_storage, next_operation_id, service, task_join_error,
};

/// The two ranking caps prompt §22 states. They are ceilings on a navigation aid, not a slice of the
/// evidence: the paged change tables still carry every row.
const MAX_TOP_SECTIONS: usize = 5;
const MAX_TOP_SYMBOLS: usize = 10;

/// The page bounds for change queries, mirroring the Analyze detail bounds (prompt §21).
const DEFAULT_CHANGE_LIMIT: usize = 100;
const MAX_CHANGE_LIMIT: usize = 500;

impl Session {
    /// The builds Compare may offer: analyzed, persisted, complete, and filed under the project this
    /// application stores user artifacts in.
    ///
    /// The project scope is fixed here rather than asked for, so the WebView cannot enumerate
    /// history that belongs to another project, and the P0 demo builds never appear as someone's
    /// firmware record (prompt §17).
    ///
    /// # Errors
    ///
    /// The storage envelope for a failed read, or an internal envelope for a poisoned lock.
    pub fn list_compare_candidates(
        &self,
        request: &CandidatePageRequestDto,
        operation_id: &str,
    ) -> Result<CandidatePageDto, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        let page = db
            .list_compare_candidates(&CandidateQuery {
                project_ids: vec![LOCAL_PROJECT_ID.to_owned()],
                offset: i64::try_from(request.offset.unwrap_or(0)).unwrap_or(MAX_CANDIDATE_LIMIT),
                // Storage clamps the page again; this cast only keeps an absurd request from
                // becoming a negative number.
                limit: request.limit.map_or(DEFAULT_CANDIDATE_LIMIT, |value| {
                    i64::try_from(value).unwrap_or(MAX_CANDIDATE_LIMIT)
                }),
            })
            .map_err(|err| envelope_from_storage(&err, operation_id))?;

        Ok(CandidatePageDto {
            rows: page.rows.into_iter().map(candidate_row).collect(),
            total: to_count(page.total),
            offset: to_count(page.offset),
            limit: to_count(page.limit),
            next_offset: page.next_offset.map(to_count),
        })
    }

    /// Load two stored builds, compare them in Core, keep the result for this session, and return the
    /// bounded summary.
    ///
    /// # Errors
    ///
    /// `ERR-STORAGE-4005` when either snapshot id names no stored build, `ERR-STORAGE-4006` when a
    /// stored record cannot be hydrated, `ERR-DIFF-5001` when both ids name the same build, and the
    /// storage envelope for a failed read.
    pub fn compare_snapshots(
        &self,
        request: &CompareRequestDto,
        operation_id: &str,
    ) -> Result<CompareSummaryDto, ErrorEnvelopeDto> {
        let (base, target) = {
            let db = self.lock_db(operation_id)?;
            let base = db
                .load_diff_input(&request.base_snapshot_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
            let target = db
                .load_diff_input(&request.target_snapshot_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
            (base, target)
        };

        let result =
            compare(&base, &target).map_err(|err| envelope_from_diff(&err, operation_id))?;
        let diff_id = self.remember_diff(result, operation_id)?;
        self.with_diff(&diff_id, operation_id, |stored| summary(&diff_id, stored))
    }

    /// One bounded page of section changes from a diff this session computed.
    ///
    /// # Errors
    ///
    /// `ERR-DIFF-5002` when the diff id was never issued or has been evicted.
    pub fn query_section_changes(
        &self,
        request: &SectionChangeQueryDto,
        operation_id: &str,
    ) -> Result<SectionChangePageDto, ErrorEnvelopeDto> {
        self.with_diff(&request.diff_id, operation_id, |result| {
            let mut rows: Vec<SectionChangeRowDto> = result
                .section_changes
                .iter()
                .filter(|row| accepts_section(row, request))
                .map(section_row)
                .collect();
            let window = sort_and_split(
                &mut rows,
                request.direction,
                request.offset,
                request.limit,
                |row| match request.sort {
                    SectionChangeSortDto::Key => RowKey::text(&row.key),
                    SectionChangeSortDto::ChangeKind => {
                        RowKey::text(&format!("{}|{}", row.change_kind, row.key))
                    }
                    SectionChangeSortDto::Delta => RowKey::optional(row.file_size.delta, &row.key),
                    SectionChangeSortDto::FileSize => RowKey::optional_u64(
                        first_known(row.file_size.target, row.file_size.base),
                        &row.key,
                    ),
                },
            );

            SectionChangePageDto {
                total: window.total,
                offset: window.offset,
                limit: window.limit,
                next_offset: window.next_offset,
                rows: window.rows,
            }
        })
    }

    /// One bounded page of symbol changes from a diff this session computed.
    ///
    /// # Errors
    ///
    /// `ERR-DIFF-5002` when the diff id was never issued or has been evicted.
    pub fn query_symbol_changes(
        &self,
        request: &SymbolChangeQueryDto,
        operation_id: &str,
    ) -> Result<SymbolChangePageDto, ErrorEnvelopeDto> {
        self.with_diff(&request.diff_id, operation_id, |result| {
            let mut rows: Vec<SymbolChangeRowDto> = result
                .symbol_changes
                .iter()
                .filter(|row| accepts_symbol(row, request))
                .map(symbol_row)
                .collect();
            let window = sort_and_split(
                &mut rows,
                request.direction,
                request.offset,
                request.limit,
                |row| match request.sort {
                    SymbolChangeSortDto::Name => RowKey::text(&row.name),
                    SymbolChangeSortDto::ChangeKind => {
                        RowKey::text(&format!("{}|{}", row.change_kind, row.name))
                    }
                    SymbolChangeSortDto::SizeDelta => RowKey::optional(row.size.delta, &row.name),
                    SymbolChangeSortDto::Size => {
                        RowKey::optional_u64(first_known(row.size.target, row.size.base), &row.name)
                    }
                },
            );

            SymbolChangePageDto {
                total: window.total,
                offset: window.offset,
                limit: window.limit,
                next_offset: window.next_offset,
                rows: window.rows,
            }
        })
    }

    /// Render the diff behind `diff_id` into the portable text for one format.
    ///
    /// This is the only projection the export path uses, so an exported document cannot disagree with
    /// the summary the screen is showing.
    ///
    /// # Errors
    ///
    /// `ERR-DIFF-5002` for an unknown or evicted diff id.
    pub fn render_export(
        &self,
        diff_id: &str,
        format: ExportFormat,
        operation_id: &str,
    ) -> Result<(String, String), ErrorEnvelopeDto> {
        self.with_diff(diff_id, operation_id, |result| {
            let dto = DiffResultDto::from_diff(result);
            let text = match format {
                ExportFormat::Json => diff_render::render_json(&dto),
                ExportFormat::Html => diff_render::render_html(&dto),
            };
            (text, suggested_file_name(result, format))
        })
    }
}

/// Turn a Core refusal into the envelope the shell presents.
fn envelope_from_diff(err: &DiffError, operation_id: &str) -> ErrorEnvelopeDto {
    let message = match err {
        DiffError::SameSnapshot { .. } => {
            "Both sides name the same build, so there is nothing to compare."
        }
    };
    ErrorEnvelopeDto {
        code: err.code().to_owned(),
        message: message.to_owned(),
        operation_id: operation_id.to_owned(),
        details: None,
        remediation: Some(
            "Choose a different base or target. A build compared with itself reports nothing, which \
             is not the same as nothing having changed."
                .to_owned(),
        ),
    }
}

/// What the user asked to write. The format is carried by the command name, so the WebView never
/// sends a path, a file name or a format string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Html,
}

impl ExportFormat {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Html => "html",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Html => "html",
        }
    }
}

/// Write `text` to a path a person chose, or report that they did not.
///
/// `confirm_overwrite` runs only when the target already exists. Injecting it keeps the whole
/// decision testable without a window, and keeps the rule honest: nothing is opened, truncated or
/// replaced until the answer is yes (prompt §36).
///
/// The bytes land in a sibling temporary file first and are renamed into place, so an export that
/// fails halfway leaves the existing file exactly as it was.
///
/// # Errors
///
/// `ERR-EXPORT-6001` when the file could not be written.
pub fn write_export(
    picked: Option<PathBuf>,
    text: &str,
    format: ExportFormat,
    mut confirm_overwrite: impl FnMut(&Path) -> bool,
) -> Result<ExportOutcomeDto, ErrorEnvelopeDto> {
    let Some(path) = picked else {
        return Ok(outcome("cancelled", None, format));
    };

    if path.exists() && !confirm_overwrite(&path) {
        return Ok(outcome("kept-existing", None, format));
    }

    let temp = temporary_next_to(&path);
    std::fs::write(&temp, text)
        .and_then(|()| std::fs::rename(&temp, &path))
        .map_err(|error| {
            // A half-written temporary must not outlive the attempt.
            let _ = std::fs::remove_file(&temp);
            ErrorEnvelopeDto {
                code: firmwaresight_report::render::EXPORT_FAILED_CODE.to_owned(),
                message: "The export could not be written.".to_owned(),
                operation_id: next_operation_id(),
                // The name only. The directory the person navigated to is not a fact the UI needs.
                details: Some(format!("{}: {error}", service::display_name(&path))),
                remediation: Some(
                    "Check that the folder exists and is writable, then export again. Any file that \
                     was already there is unchanged."
                        .to_owned(),
                ),
            }
        })?;

    Ok(outcome(
        "written",
        Some(service::display_name(&path)),
        format,
    ))
}

fn outcome(status: &str, file_name: Option<String>, format: ExportFormat) -> ExportOutcomeDto {
    ExportOutcomeDto {
        status: status.to_owned(),
        file_name,
        format: format.label().to_owned(),
    }
}

/// A scratch name in the same directory as the target, so the rename never crosses a volume.
fn temporary_next_to(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "export".to_owned());
    path.with_file_name(format!(".{name}.tmp-{:x}", std::process::id()))
}

/// A default name for the save dialog: which two builds, in eight hex characters each. No directory,
/// because choosing the directory is what the dialog is for.
fn suggested_file_name(result: &DiffResult, format: ExportFormat) -> String {
    format!(
        "firmwaresight-diff-{}-{}.{}",
        short(&result.base_artifact.sha256),
        short(&result.target_artifact.sha256),
        format.extension()
    )
}

fn short(sha: &str) -> &str {
    &sha[..8.min(sha.len())]
}

/// Open the native save dialog. Modal, so it only ever runs on a blocking thread.
fn save_path(
    app: &AppHandle,
    title: &str,
    format: ExportFormat,
    suggested: &str,
) -> Option<PathBuf> {
    app.dialog()
        .file()
        .set_title(title)
        .add_filter(
            format!("FirmwareSight {} export", format.label()),
            &[format.extension()],
        )
        .set_file_name(suggested)
        .blocking_save_file()
        .and_then(|picked| picked.into_path().ok())
}

/// Ask, in a person's words, whether an existing file may be replaced.
fn confirm_replacement(app: &AppHandle, file_name: &str) -> bool {
    app.dialog()
        .message(format!(
            "{file_name} already exists. Replacing it overwrites the file you have."
        ))
        .title("Replace existing file?")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Replace it".to_owned(),
            "Keep mine".to_owned(),
        ))
        .blocking_show()
}

/// The whole export flow for one format, shared by both commands.
fn export_now(
    app: &AppHandle,
    session: &Session,
    diff_id: &str,
    format: ExportFormat,
    title: &str,
    operation_id: &str,
) -> Result<ExportOutcomeDto, ErrorEnvelopeDto> {
    let (text, suggested) = session.render_export(diff_id, format, operation_id)?;
    let picked = save_path(app, title, format, &suggested);
    write_export(picked, &text, format, |path| {
        confirm_replacement(app, &service::display_name(path))
    })
}

/// The builds a comparison can start from, one bounded page at a time.
#[tauri::command]
pub(crate) async fn list_compare_candidates(
    request: CandidatePageRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<CandidatePageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.list_compare_candidates(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

/// Compare two stored builds and get the bounded summary plus a session-local diff handle.
#[tauri::command]
pub(crate) async fn compare_snapshots(
    request: CompareRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<CompareSummaryDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.compare_snapshots(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// The section changes of a computed diff, one page at a time.
#[tauri::command]
pub(crate) async fn query_section_changes(
    request: SectionChangeQueryDto,
    state: State<'_, Arc<Session>>,
) -> Result<SectionChangePageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.query_section_changes(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

/// The symbol changes of a computed diff, one page at a time.
#[tauri::command]
pub(crate) async fn query_symbol_changes(
    request: SymbolChangeQueryDto,
    state: State<'_, Arc<Session>>,
) -> Result<SymbolChangePageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.query_symbol_changes(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

/// Export the comparison as the portable JSON document. Cancelling the dialog resolves to
/// `status: "cancelled"`, which is not an error.
#[tauri::command]
pub(crate) async fn export_compare_json(
    app: AppHandle,
    diff_id: String,
    state: State<'_, Arc<Session>>,
) -> Result<ExportOutcomeDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || {
        export_now(
            &app,
            &session,
            &diff_id,
            ExportFormat::Json,
            "Export the build diff as JSON",
            &operation_id,
        )
    })
    .await
    .map_err(task_join_error)?
}

/// Export the comparison as one self-contained HTML file.
#[tauri::command]
pub(crate) async fn export_compare_html(
    app: AppHandle,
    diff_id: String,
    state: State<'_, Arc<Session>>,
) -> Result<ExportOutcomeDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || {
        export_now(
            &app,
            &session,
            &diff_id,
            ExportFormat::Html,
            "Export the build diff as HTML",
            &operation_id,
        )
    })
    .await
    .map_err(task_join_error)?
}

/// The stored builds, as the selector shows them.
fn candidate_row(row: CompareCandidate) -> CompareCandidateDto {
    CompareCandidateDto {
        build_id: row.build_id,
        snapshot_id: row.snapshot_id,
        file_name: row.file_name,
        sha256: row.sha256,
        byte_size: row.byte_size,
        architecture: row.architecture,
        imported_at: row.imported_at,
        nonvolatile: stored_budget(row.nonvolatile),
        runtime_ram: stored_budget(row.runtime_ram),
    }
}

fn stored_budget(budget: StoredBudget) -> StoredBudgetDto {
    StoredBudgetDto {
        state: budget.state,
        bytes: budget.bytes,
    }
}

fn to_count(value: i64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

/// The bounded summary prompt §22 asks for: identities, the two budget deltas with each side's own
/// state, counts, attribution, capped top lists, and the warnings. No change table.
fn summary(diff_id: &str, result: &DiffResult) -> CompareSummaryDto {
    CompareSummaryDto {
        diff_id: diff_id.to_owned(),
        base: side(&result.base_snapshot_id, &result.base_artifact),
        target: side(&result.target_snapshot_id, &result.target_artifact),
        memory: memory(result),
        counts: counts(result),
        object_changes: ObjectAttributionDto {
            available: result.attribution.available,
            reason: result.attribution.reason.clone(),
        },
        top_sections: result
            .top_section_growth(MAX_TOP_SECTIONS)
            .into_iter()
            .map(contributor)
            .collect(),
        top_symbols: result
            .top_symbol_growth(MAX_TOP_SYMBOLS)
            .into_iter()
            .map(contributor)
            .collect(),
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

fn side(snapshot_id: &str, artifact: &DiffArtifact) -> DiffSideDto {
    DiffSideDto {
        snapshot_id: snapshot_id.to_owned(),
        file_name: artifact.file_name.clone(),
        sha256: artifact.sha256.clone(),
    }
}

fn memory(result: &DiffResult) -> DiffMemoryDto {
    let memory = &result.memory;
    DiffMemoryDto {
        nonvolatile: delta(&memory.nonvolatile),
        runtime_ram: delta(&memory.runtime_ram),
        comparability: memory.comparability.label().to_owned(),
        base: side_memory(&memory.base),
        target: side_memory(&memory.target),
        evidence_warning: memory.evidence_warning.clone(),
    }
}

fn side_memory(side: &DiffSideMemory) -> DiffSideMemoryDto {
    DiffSideMemoryDto {
        footprint_row_present: side.footprint_row_present,
        nonvolatile: StoredBudgetDto {
            state: side.nonvolatile.state.as_str().to_owned(),
            bytes: side.nonvolatile.bytes,
        },
        runtime_ram: StoredBudgetDto {
            state: side.runtime_ram.state.as_str().to_owned(),
            bytes: side.runtime_ram.bytes,
        },
        map_backed: side.evidence.map_backed,
        layout_source: side.evidence.layout_source.clone(),
        weakest_evidence_basis: side.evidence.weakest_basis.clone(),
    }
}

fn delta(change: &ByteChange) -> ByteDeltaDto {
    ByteDeltaDto {
        base: change.base,
        target: change.target,
        delta: change.delta,
        comparability: change.comparability.label().to_owned(),
        reason: change.reason.clone(),
    }
}

fn counts(result: &DiffResult) -> DiffCountsDto {
    DiffCountsDto {
        sections: ChangeKindCountsDto {
            added: result.counts.sections_added,
            removed: result.counts.sections_removed,
            changed: result.counts.sections_changed,
            ambiguous: result.counts.sections_ambiguous,
        },
        symbols: ChangeKindCountsDto {
            added: result.counts.symbols_added,
            removed: result.counts.symbols_removed,
            changed: result.counts.symbols_changed,
            ambiguous: result.counts.symbols_ambiguous,
        },
        unchanged_sections: result.unchanged.sections,
        unchanged_symbols: result.unchanged.symbols,
    }
}

fn contributor(row: Contributor) -> ContributorDto {
    ContributorDto {
        key: row.key,
        change_kind: row.change.label().to_owned(),
        delta: row.delta,
        bytes: row.bytes,
    }
}

fn accepts_section(row: &SectionChange, request: &SectionChangeQueryDto) -> bool {
    matches_kind(row.change, request.change_kind)
        && request
            .filter
            .as_deref()
            .is_none_or(|needle| contains_case_insensitive(&row.key, needle))
}

fn accepts_symbol(row: &SymbolChange, request: &SymbolChangeQueryDto) -> bool {
    matches_kind(row.change, request.change_kind)
        && request
            .filter
            .as_deref()
            .is_none_or(|needle| contains_case_insensitive(&row.name, needle))
}

fn matches_kind(change: ChangeKind, filter: Option<ChangeKindFilterDto>) -> bool {
    let Some(wanted) = filter else {
        return true;
    };
    change
        == match wanted {
            ChangeKindFilterDto::Added => ChangeKind::Added,
            ChangeKindFilterDto::Removed => ChangeKind::Removed,
            ChangeKindFilterDto::Changed => ChangeKind::Changed,
        }
}

/// SQLite's `LIKE` is case-insensitive for ASCII, which is how the Analyze detail filters behave. The
/// change filters match that, so one substring means the same thing on both screens.
fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Where a row belongs in an ordering. `measured` separates rows that carry a value for the sorted
/// field from those that do not, so a descending sort cannot pull an unknown to the top: an absent
/// number is not a small number.
#[derive(Debug)]
struct RowKey {
    measured: bool,
    value: i128,
    text: String,
}

impl RowKey {
    /// A text ordering: every row is placed, and the tie-break is the text itself.
    fn text(text: &str) -> Self {
        Self {
            measured: true,
            value: 0,
            text: text.to_owned(),
        }
    }

    /// A signed quantity ordering, with the row name as tie-break.
    fn optional(value: Option<i64>, tie: &str) -> Self {
        Self {
            measured: value.is_some(),
            value: i128::from(value.unwrap_or(0)),
            text: tie.to_owned(),
        }
    }

    /// An unsigned quantity ordering. A row the diff has no number for on either side sorts last.
    fn optional_u64(value: Option<u64>, tie: &str) -> Self {
        Self {
            measured: value.is_some(),
            value: value.map_or(0, i128::from),
            text: tie.to_owned(),
        }
    }
}

/// The page geometry after ordering.
#[derive(Debug)]
struct Window<T> {
    rows: Vec<T>,
    total: usize,
    offset: usize,
    limit: usize,
    next_offset: Option<usize>,
}

/// Order the rows by a projected key, then cut the requested page out of them.
///
/// The ordering is total and independent of the input order, so the same query twice returns the same
/// page, and a requested limit is clamped to the boundary rather than honoured.
fn sort_and_split<T>(
    rows: &mut Vec<T>,
    direction: SortDirDto,
    offset: usize,
    limit: Option<usize>,
    mut key: impl FnMut(&T) -> RowKey,
) -> Window<T> {
    let descending = direction == SortDirDto::Desc;
    rows.sort_by(|a, b| {
        let (left, right) = (key(a), key(b));
        // Unmeasured rows go last in both directions, so flipping the column never promotes one.
        let measured = right.measured.cmp(&left.measured);
        let ordering = if descending {
            right
                .value
                .cmp(&left.value)
                .then_with(|| right.text.cmp(&left.text))
        } else {
            left.value
                .cmp(&right.value)
                .then_with(|| left.text.cmp(&right.text))
        };
        measured.then(ordering)
    });

    let total = rows.len();
    let limit = limit
        .unwrap_or(DEFAULT_CHANGE_LIMIT)
        .clamp(1, MAX_CHANGE_LIMIT);
    let start = offset.min(total);
    let end = (start + limit).min(total);
    Window {
        rows: rows.drain(start..end).collect(),
        total,
        offset: start,
        limit,
        next_offset: if end < total { Some(end) } else { None },
    }
}

fn section_row(row: &SectionChange) -> SectionChangeRowDto {
    SectionChangeRowDto {
        change_kind: row.change.label().to_owned(),
        key: row.key.clone(),
        name_known: row.name_known,
        ambiguous: row.ambiguous,
        file_size: delta(&row.file_size),
        memory_size: delta(&row.memory_size),
        base: row.base.as_ref().map(section_side),
        target: row.target.as_ref().map(section_side),
        differing_fields: row.differing_fields.clone(),
        indeterminate_fields: row.indeterminate_fields.clone(),
    }
}

fn section_side(row: &DiffSection) -> SectionSideDto {
    SectionSideDto {
        index: to_count(row.index),
        role: row.role.clone(),
        region: row.region.value().cloned(),
        virtual_address: row.virtual_address.value().copied().map(hex_value),
        load_address: row.load_address.value().copied().map(hex_value),
        file_offset: row.file_offset.map(hex_value),
        file_size: row.file_size,
        memory_size: row.memory_size.value().copied(),
    }
}

fn symbol_row(row: &SymbolChange) -> SymbolChangeRowDto {
    SymbolChangeRowDto {
        change_kind: row.change.label().to_owned(),
        name: row.name.clone(),
        kind: row.kind.clone(),
        binding: row.binding.clone(),
        ambiguous: row.ambiguous,
        size: delta(&row.size),
        base: row.base.as_ref().map(symbol_side),
        target: row.target.as_ref().map(symbol_side),
        differing_fields: row.differing_fields.clone(),
        indeterminate_fields: row.indeterminate_fields.clone(),
    }
}

fn symbol_side(row: &DiffSymbol) -> SymbolSideDto {
    SymbolSideDto {
        ordinal: to_count(row.ordinal),
        address: row.address.map(hex_value),
        size: row.size.value().copied(),
        section_ref: row.section_ref.clone(),
    }
}

/// The same hexadecimal form the Analyze tables and the CLI print, so one address looks identical
/// wherever a reader meets it.
fn hex_value(value: u64) -> String {
    format!("{value:#010x}")
}

fn first_known(target: Option<u64>, base: Option<u64>) -> Option<u64> {
    target.or(base)
}
