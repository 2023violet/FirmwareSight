//! The P5 History surface: three bounded reads over what this store already holds.
//!
//! History is a viewer of persisted facts, not a second source of them. Everything here follows the
//! rules the Compare and Detail layers already established, because the same four boundaries apply:
//!
//! 1. **Rust fixes the scope.** The WebView never names a project. Each command reads the one
//!    project this application files user artifacts under, so a page in the window cannot enumerate
//!    history belonging to somebody else (`AGENTS.md` 7, prompt §44).
//! 2. **Rust clamps the page.** A request for more rows than a page is answered with a page, and the
//!    limit actually applied is reported back rather than the one asked for (prompt §17).
//! 3. **No path crosses.** A stored artifact `path` is reduced to its file name inside
//!    `firmwaresight-storage`, before it reaches this module. Nothing here re-derives a directory.
//! 4. **Nothing is re-decided.** A disposition, a state count or a footprint read here is the stored
//!    value. The Gate verdict is not recomputed for the screen, and no size, hash or budget rule is
//!    re-implemented in this layer (`AGENTS.md` 3).
//!
//! Reads are read-only by construction: there is no command in this module that writes, edits or
//! deletes, and the tables behind them already refuse an UPDATE to a Gate run or a release record.

use std::sync::Arc;

use firmwaresight_storage::{
    DEFAULT_HISTORY_LIMIT, HistoryGateRun, HistoryQuery, HistoryReleaseRecord, MAX_HISTORY_LIMIT,
};
use tauri::State;

use crate::ipc::{
    CandidatePageDto, ErrorEnvelopeDto, GateCountsDto, HistoryGateRunPageDto, HistoryGateRunRowDto,
    HistoryPageRequestDto, HistoryReleasePageDto, HistoryReleaseRowDto,
};
use crate::{LOCAL_PROJECT_ID, Session, envelope_from_storage, next_operation_id, task_join_error};

impl Session {
    /// The builds this application stored, one bounded page at a time.
    ///
    /// # Errors
    ///
    /// The storage envelope for a failed read, or an internal envelope for a poisoned lock.
    pub fn list_history_builds(
        &self,
        request: &HistoryPageRequestDto,
        operation_id: &str,
    ) -> Result<CandidatePageDto, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        let page = db
            .list_history_builds(&self.page_query(request))
            .map_err(|err| envelope_from_storage(&err, operation_id))?;

        Ok(CandidatePageDto {
            rows: page
                .rows
                .into_iter()
                .map(crate::compare::candidate_row)
                .collect(),
            total: to_count(page.total),
            offset: to_count(page.offset),
            limit: to_count(page.limit),
            next_offset: page.next_offset.map(to_count),
        })
    }

    /// The Gate runs stored against those builds, newest recorded verdict first.
    ///
    /// # Errors
    ///
    /// As [`Session::list_history_builds`].
    pub fn list_history_gate_runs(
        &self,
        request: &HistoryPageRequestDto,
        operation_id: &str,
    ) -> Result<HistoryGateRunPageDto, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        let page = db
            .list_history_gate_runs(&self.page_query(request))
            .map_err(|err| envelope_from_storage(&err, operation_id))?;

        Ok(HistoryGateRunPageDto {
            rows: page.rows.into_iter().map(gate_row).collect(),
            total: to_count(page.total),
            offset: to_count(page.offset),
            limit: to_count(page.limit),
            next_offset: page.next_offset.map(to_count),
        })
    }

    /// The published release records, newest stored audit time first.
    ///
    /// # Errors
    ///
    /// As [`Session::list_history_builds`].
    pub fn list_history_releases(
        &self,
        request: &HistoryPageRequestDto,
        operation_id: &str,
    ) -> Result<HistoryReleasePageDto, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        let page = db
            .list_history_releases(&self.page_query(request))
            .map_err(|err| envelope_from_storage(&err, operation_id))?;

        Ok(HistoryReleasePageDto {
            rows: page.rows.into_iter().map(release_row).collect(),
            total: to_count(page.total),
            offset: to_count(page.offset),
            limit: to_count(page.limit),
            next_offset: page.next_offset.map(to_count),
        })
    }

    /// The scope and the page, with Rust's project and Rust's ceiling.
    ///
    /// Two different absences are answered differently, in the shape the Compare candidate command
    /// already uses so the two bounded reads cannot disagree: a field that was not sent at all means
    /// this table's default page, while a number too large to fit an `i64` means the ceiling rather
    /// than wrapping into a small or negative limit. The storage layer clamps again and reports the
    /// limit it actually applied.
    fn page_query(&self, request: &HistoryPageRequestDto) -> HistoryQuery {
        HistoryQuery {
            project_ids: vec![LOCAL_PROJECT_ID.to_owned()],
            offset: request
                .offset
                .map_or(0, |value| i64::try_from(value).unwrap_or(MAX_HISTORY_LIMIT)),
            limit: request.limit.map_or(DEFAULT_HISTORY_LIMIT, |value| {
                i64::try_from(value).unwrap_or(MAX_HISTORY_LIMIT)
            }),
            filter: request.filter.clone().filter(|text| !text.is_empty()),
        }
    }
}

fn to_count(value: i64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn gate_row(row: HistoryGateRun) -> HistoryGateRunRowDto {
    HistoryGateRunRowDto {
        run_id: row.run_id,
        build_id: row.build_id,
        file_name: row.file_name,
        baseline_build_id: row.baseline_build_id,
        baseline_file_name: row.baseline_file_name,
        disposition: row.disposition,
        counts: GateCountsDto {
            pass: row.counts.pass,
            review: row.counts.review,
            block: row.counts.block,
            unknown: row.counts.unknown,
            not_applicable: row.counts.not_applicable,
        },
        stored_at: row.stored_at,
    }
}

fn release_row(row: HistoryReleaseRecord) -> HistoryReleaseRowDto {
    HistoryReleaseRowDto {
        release_id: row.release_id,
        release_version: row.release_version,
        build_id: row.build_id,
        file_name: row.file_name,
        gate_run_id: row.gate_run_id,
        manifest_sha256: row.manifest_sha256,
        stored_at: row.stored_at,
    }
}

/// One bounded page of stored builds.
#[tauri::command]
pub(crate) async fn list_history_builds(
    request: HistoryPageRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<CandidatePageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.list_history_builds(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

/// One bounded page of stored Gate runs.
#[tauri::command]
pub(crate) async fn list_history_gate_runs(
    request: HistoryPageRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<HistoryGateRunPageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.list_history_gate_runs(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

/// One bounded page of stored release records.
#[tauri::command]
pub(crate) async fn list_history_releases(
    request: HistoryPageRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<HistoryReleasePageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.list_history_releases(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}
