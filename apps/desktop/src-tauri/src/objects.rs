//! M2 object and library attribution (ADR-0033): the Analyze "Objects & libraries" table of one
//! stored build, and the "By object/library" change table of one computed diff.
//!
//! Both are bounded pages over facts Core already decided. Storage keeps each MAP row's charge from
//! import time and Core aggregates and diffs them; this module only filters, orders and cuts a page,
//! the same way the section and symbol change tables do. A request names a snapshot or a diff id and
//! never a path (`AGENTS.md` 7), and every key it returns is a normalized object name, never the
//! path the linker printed.

use std::sync::Arc;

use firmwaresight_core::domain::attribution::ObjectUsage;
use firmwaresight_core::domain::diff::{ChangeKind, DiffResult, ObjectChange};
use tauri::State;

use crate::compare::{
    RowKey, contains_case_insensitive, contributor, delta, matches_kind, sort_and_split,
};
use crate::ipc::{
    ErrorEnvelopeDto, ObjectAttributionDto, ObjectChangeCountsDto, ObjectChangePageDto,
    ObjectChangeQueryDto, ObjectChangeRowDto, ObjectChangeSortDto, ObjectPageDto, ObjectRequestDto,
    ObjectRowDto, ObjectSortDto,
};
use crate::{Session, envelope_from_storage, next_operation_id, task_join_error};

/// The capped Compare summary lists, matching the section list's size.
const MAX_TOP_OBJECTS: usize = 5;

impl Session {
    /// One bounded page of a stored build's objects or libraries.
    ///
    /// # Errors
    ///
    /// `ERR-STORAGE-4005` when the snapshot id names no build, the storage envelope for a failed
    /// read, and an internal envelope for a poisoned lock.
    pub fn query_objects(
        &self,
        request: &ObjectRequestDto,
        operation_id: &str,
    ) -> Result<ObjectPageDto, ErrorEnvelopeDto> {
        let (attribution, map_present) = {
            let db = self.lock_db(operation_id)?;
            let attribution = db
                .object_attribution(&request.snapshot_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
            let map_present = db
                .snapshot_has_map(&request.snapshot_id)
                .map_err(|err| envelope_from_storage(&err, operation_id))?;
            (attribution, map_present)
        };
        let Some(attribution) = attribution else {
            // ADR-0033 D6: the two absences have different remedies, so they are not one sentence.
            let reason = if map_present {
                "this build was stored with a linker MAP but without object rows: the MAP lists no \
                 input sections, or the build was stored by a FirmwareSight version that did not keep \
                 them (analyzing the same files again reuses that stored build)"
            } else {
                "this build was analyzed without a linker MAP, and only a MAP names the object file \
                 behind each byte"
            };
            return Ok(ObjectPageDto {
                available: false,
                map_present,
                reason: Some(reason.to_owned()),
                rows: Vec::new(),
                total: 0,
                offset: 0,
                limit: 0,
                next_offset: None,
                nonvolatile_total: None,
                runtime_ram_total: None,
                unattributed_nonvolatile: None,
                unattributed_runtime_ram: None,
            });
        };

        let summary = attribution.summary;
        let usages = if request.libraries {
            summary.libraries
        } else {
            summary.objects
        };
        let mut rows: Vec<ObjectRowDto> = usages
            .into_iter()
            .filter(|usage| {
                request
                    .filter
                    .as_deref()
                    .is_none_or(|needle| contains_case_insensitive(&usage.key, needle))
            })
            .map(|usage| object_row(usage, attribution.exact_nonvolatile))
            .collect();
        let window = sort_and_split(
            &mut rows,
            request.direction,
            request.offset,
            request.limit,
            |row| match request.sort {
                ObjectSortDto::Flash => RowKey::optional_u64(row.nonvolatile_bytes, &row.key),
                ObjectSortDto::Ram => RowKey::optional_u64(row.runtime_ram_bytes, &row.key),
                ObjectSortDto::Name => RowKey::text(&row.key),
            },
        );

        Ok(ObjectPageDto {
            available: true,
            map_present,
            reason: None,
            rows: window.rows,
            total: window.total,
            offset: window.offset,
            limit: window.limit,
            next_offset: window.next_offset,
            nonvolatile_total: attribution.exact_nonvolatile,
            runtime_ram_total: attribution.exact_runtime_ram,
            unattributed_nonvolatile: summary.unattributed_nonvolatile,
            unattributed_runtime_ram: summary.unattributed_runtime_ram,
        })
    }

    /// One bounded page of object or library changes from a diff this session computed.
    ///
    /// # Errors
    ///
    /// `ERR-DIFF-5002` when the diff id was never issued or has been evicted.
    pub fn query_object_changes(
        &self,
        request: &ObjectChangeQueryDto,
        operation_id: &str,
    ) -> Result<ObjectChangePageDto, ErrorEnvelopeDto> {
        self.with_diff(&request.diff_id, operation_id, |result| {
            let source = if request.libraries {
                &result.attribution.libraries
            } else {
                &result.attribution.objects
            };
            let mut rows: Vec<ObjectChangeRowDto> = source
                .iter()
                .filter(|row| {
                    matches_kind(row.change, request.change_kind)
                        && request
                            .filter
                            .as_deref()
                            .is_none_or(|needle| contains_case_insensitive(&row.key, needle))
                })
                .map(change_row)
                .collect();
            let window = sort_and_split(
                &mut rows,
                request.direction,
                request.offset,
                request.limit,
                |row| match request.sort {
                    ObjectChangeSortDto::FlashDelta => {
                        RowKey::optional(row.nonvolatile.delta, &row.key)
                    }
                    ObjectChangeSortDto::RamDelta => {
                        RowKey::optional(row.runtime_ram.delta, &row.key)
                    }
                    ObjectChangeSortDto::Key => RowKey::text(&row.key),
                    ObjectChangeSortDto::ChangeKind => {
                        RowKey::text(&format!("{}|{}", row.change_kind, row.key))
                    }
                },
            );
            ObjectChangePageDto {
                rows: window.rows,
                total: window.total,
                offset: window.offset,
                limit: window.limit,
                next_offset: window.next_offset,
            }
        })
    }
}

/// The Compare summary's attribution block: availability, counts and Core's capped rankings.
pub(crate) fn attribution_summary(result: &DiffResult) -> ObjectAttributionDto {
    let attribution = &result.attribution;
    ObjectAttributionDto {
        available: attribution.available,
        reason: attribution.reason.clone(),
        objects: counts(&attribution.objects, attribution.unchanged_objects),
        libraries: counts(&attribution.libraries, attribution.unchanged_libraries),
        top_objects: result
            .top_object_growth(false, MAX_TOP_OBJECTS)
            .into_iter()
            .map(contributor)
            .collect(),
        added_objects: result
            .largest_added_objects(false, MAX_TOP_OBJECTS)
            .into_iter()
            .map(contributor)
            .collect(),
        top_libraries: result
            .top_object_growth(true, MAX_TOP_OBJECTS)
            .into_iter()
            .map(contributor)
            .collect(),
    }
}

fn counts(rows: &[ObjectChange], unchanged: usize) -> ObjectChangeCountsDto {
    let count = |kind: ChangeKind| rows.iter().filter(|row| row.change == kind).count();
    ObjectChangeCountsDto {
        changed: count(ChangeKind::Changed),
        added: count(ChangeKind::Added),
        removed: count(ChangeKind::Removed),
        unchanged,
    }
}

fn object_row(usage: ObjectUsage, exact_total: Option<u64>) -> ObjectRowDto {
    ObjectRowDto {
        key: usage.key,
        archive: usage.archive,
        nonvolatile_bytes: usage.nonvolatile,
        runtime_ram_bytes: usage.runtime_ram,
        nonvolatile_share_basis_points: share_basis_points(usage.nonvolatile, exact_total),
        map_rows: usage.rows,
    }
}

/// `part / total` in hundredths of a percent, rounded down so a share is never overstated.
fn share_basis_points(part: Option<u64>, total: Option<u64>) -> Option<u32> {
    let (part, total) = (part?, total?);
    if total == 0 || part > total {
        return None;
    }
    u32::try_from(u128::from(part) * 10_000 / u128::from(total)).ok()
}

fn change_row(row: &ObjectChange) -> ObjectChangeRowDto {
    ObjectChangeRowDto {
        change_kind: row.change.label().to_owned(),
        key: row.key.clone(),
        archive: row.archive.clone(),
        nonvolatile: delta(&row.nonvolatile),
        runtime_ram: delta(&row.runtime_ram),
        ambiguous: row.ambiguous,
    }
}

/// The objects or libraries of one stored build, one page at a time.
#[tauri::command]
pub(crate) async fn query_objects(
    request: ObjectRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<ObjectPageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.query_objects(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// The object or library changes of a computed diff, one page at a time.
#[tauri::command]
pub(crate) async fn query_object_changes(
    request: ObjectChangeQueryDto,
    state: State<'_, Arc<Session>>,
) -> Result<ObjectChangePageDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.query_object_changes(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}

#[cfg(test)]
mod tests {
    use super::share_basis_points;

    #[test]
    fn a_share_needs_both_numbers_and_rounds_down() {
        assert_eq!(share_basis_points(Some(1), Some(3)), Some(3333));
        assert_eq!(share_basis_points(Some(1), Some(100_000)), Some(0));
        assert_eq!(share_basis_points(Some(5), Some(5)), Some(10_000));
        assert_eq!(share_basis_points(None, Some(5)), None);
        assert_eq!(share_basis_points(Some(5), None), None);
        assert_eq!(share_basis_points(Some(5), Some(0)), None);
        assert_eq!(
            share_basis_points(Some(6), Some(5)),
            None,
            "never above the whole"
        );
        assert_eq!(
            share_basis_points(Some(u64::MAX), Some(u64::MAX)),
            Some(10_000)
        );
    }
}
