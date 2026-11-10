//! D8 (ADR-0041): the History trend, one bounded read over stored builds.
//!
//! The same four boundaries as the rest of History hold here: Rust fixes the project, Rust clamps the
//! read, no path crosses (file names only, from `firmwaresight-storage`), and nothing is re-decided. The
//! series, deltas, headroom and object movers come from `firmwaresight_core::domain::trend`; this module
//! only maps them to DTOs.

use std::sync::Arc;

use firmwaresight_core::domain::memory::RegionKind;
use firmwaresight_core::domain::trend::{
    BudgetPoint, ObjectSeries, RegionCell, RegionSeries, Trend, TrendRegions,
};
use firmwaresight_storage::{DEFAULT_TREND_LIMIT, MAX_TREND_LIMIT, StoredBudget};
use tauri::State;

use crate::ipc::{
    ErrorEnvelopeDto, HeadroomDto, HistoryTrendDto, HistoryTrendRequestDto, ObjectTrendDto,
    RegionCellDto, RegionTrendDto, TrendPointDto, TrendValueDto,
};
use crate::{LOCAL_PROJECT_ID, Session, envelope_from_storage, next_operation_id, task_join_error};

/// Object movers kept per budget.
const MOVERS: usize = 5;

impl Session {
    /// The trend over this project's newest stored builds.
    ///
    /// # Errors
    ///
    /// The storage envelope for a failed read, or an internal envelope for a poisoned lock.
    pub fn history_trend(
        &self,
        request: &HistoryTrendRequestDto,
        operation_id: &str,
    ) -> Result<HistoryTrendDto, ErrorEnvelopeDto> {
        let limit = request.limit.map_or(DEFAULT_TREND_LIMIT, |value| {
            i64::try_from(value).unwrap_or(MAX_TREND_LIMIT)
        });
        let stored = {
            let db = self.lock_db(operation_id)?;
            db.history_trend(&[LOCAL_PROJECT_ID.to_owned()], limit)
                .map_err(|err| envelope_from_storage(&err, operation_id))?
        };
        let builds: Vec<_> = stored.points.iter().map(|p| p.trend_build()).collect();
        let trend = Trend::compute(&builds, MOVERS);

        let points = stored
            .points
            .iter()
            .enumerate()
            .map(|(index, point)| TrendPointDto {
                build_id: point.build_id.clone(),
                snapshot_id: point.snapshot_id.clone(),
                file_name: point.file_name.clone(),
                stored_at: point.stored_at.clone(),
                flash: value(&point.nonvolatile, trend.nonvolatile[index]),
                ram: value(&point.runtime_ram, trend.runtime_ram[index]),
                gate: point.gate_disposition.clone(),
                releases: point.release_versions.clone(),
                pinned: point.pinned,
                regions: match point.regions {
                    TrendRegions::NotRecorded => "notRecorded",
                    TrendRegions::NoRegionTable => "noRegionTable",
                    TrendRegions::Recorded(_) => "recorded",
                }
                .to_owned(),
            })
            .collect();

        Ok(HistoryTrendDto {
            points,
            regions: trend.regions.iter().map(region).collect(),
            movers_flash: trend.movers_nonvolatile.iter().map(object).collect(),
            movers_ram: trend.movers_runtime_ram.iter().map(object).collect(),
            total_builds: usize::try_from(stored.total_builds).unwrap_or(usize::MAX),
            limit: usize::try_from(stored.limit).unwrap_or(usize::MAX),
        })
    }
}

fn value(stored: &StoredBudget, point: BudgetPoint) -> TrendValueDto {
    TrendValueDto {
        state: stored.state.clone(),
        bytes: point.bytes,
        delta: point.delta,
    }
}

fn region(series: &RegionSeries) -> RegionTrendDto {
    RegionTrendDto {
        name: series.name.clone(),
        kind: match series.kind {
            RegionKind::Nonvolatile => "flash",
            RegionKind::VolatileRam => "ram",
            RegionKind::Unknown => "unknown",
        }
        .to_owned(),
        cells: series
            .cells
            .iter()
            .map(|cell| match *cell {
                RegionCell::NotRecorded => empty_cell("notRecorded"),
                RegionCell::NoRegionTable => empty_cell("noRegionTable"),
                RegionCell::Absent => empty_cell("absent"),
                RegionCell::Recorded {
                    capacity,
                    used,
                    remaining,
                    exceeded,
                    exact,
                } => RegionCellDto {
                    state: "recorded".to_owned(),
                    capacity,
                    used,
                    remaining,
                    exceeded,
                    exact,
                },
            })
            .collect(),
        latest: series.latest.map(|h| HeadroomDto {
            point_index: h.build_index,
            capacity: h.capacity,
            used: h.used,
            remaining: h.remaining,
            remaining_per_mille: h.remaining_per_mille,
            exceeded: h.exceeded,
            exact: h.exact,
        }),
    }
}

fn empty_cell(state: &str) -> RegionCellDto {
    RegionCellDto {
        state: state.to_owned(),
        capacity: None,
        used: None,
        remaining: None,
        exceeded: false,
        exact: false,
    }
}

fn object(series: &ObjectSeries) -> ObjectTrendDto {
    ObjectTrendDto {
        object: series.object.clone(),
        values: series.values.clone(),
        first: series.first,
        last: series.last,
        delta: series.delta,
    }
}

/// The History trend over this project's stored builds.
#[tauri::command]
pub(crate) async fn query_history_trend(
    request: HistoryTrendRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<HistoryTrendDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || session.history_trend(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}
