//! The History trend read (ADR-0041 D2): the newest stored builds, oldest first, with what Core's trend
//! needs and what a point's details show.
//!
//! Rows only, like the rest of History: no file is opened, no number is re-derived. The series themselves
//! are decided in `firmwaresight_core::domain::trend`; this module only reads and labels.

use std::collections::{BTreeMap, HashMap};

use rusqlite::params_from_iter;
use rusqlite::types::ToSql;

use firmwaresight_core::domain::diff::display_file_name;
use firmwaresight_core::domain::memory::RegionKind;
use firmwaresight_core::domain::trend::{
    TrendBudget, TrendBuild, TrendObject, TrendRegion, TrendRegions,
};

use crate::Database;
use crate::compare::{StoredBudget, in_placeholders, read_err};
use crate::error::StorageError;

/// How many builds a trend reads when the caller does not say.
pub const DEFAULT_TREND_LIMIT: i64 = 100;
/// The most builds one trend read returns, whatever the caller asked for.
pub const MAX_TREND_LIMIT: i64 = 500;

/// One stored build, as a trend point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredTrendPoint {
    pub build_id: String,
    pub snapshot_id: String,
    /// File name only. Never a host path.
    pub file_name: String,
    /// `builds.created_at`: when FirmwareSight stored it, not when it was built.
    pub stored_at: String,
    pub nonvolatile: StoredBudget,
    pub runtime_ram: StoredBudget,
    pub regions: TrendRegions,
    /// `None`: no object rows.
    pub objects: Option<Vec<TrendObject>>,
    /// The newest Gate disposition stored for this build (`PASS`, `REVIEW`, `BLOCK`), if any.
    pub gate_disposition: Option<String>,
    /// Release versions recorded for this build, oldest first.
    pub release_versions: Vec<String>,
    /// Whether this build is the project's pinned baseline now.
    pub pinned: bool,
}

impl StoredTrendPoint {
    /// The part of this point Core's trend reads.
    #[must_use]
    pub fn trend_build(&self) -> TrendBuild {
        TrendBuild {
            nonvolatile: trend_budget(&self.nonvolatile),
            runtime_ram: trend_budget(&self.runtime_ram),
            regions: self.regions.clone(),
            objects: self.objects.clone(),
        }
    }
}

fn trend_budget(stored: &StoredBudget) -> TrendBudget {
    match stored.state.as_str() {
        "exact" => TrendBudget {
            bytes: stored.bytes,
            exact: stored.bytes.is_some(),
        },
        "partial" => TrendBudget {
            bytes: stored.bytes,
            exact: false,
        },
        _ => TrendBudget::unknown(),
    }
}

/// The trend read's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredTrend {
    /// Oldest first.
    pub points: Vec<StoredTrendPoint>,
    /// How many complete builds the project holds in all.
    pub total_builds: i64,
    /// The limit actually applied.
    pub limit: i64,
}

impl Database {
    /// The newest `limit` complete builds of these projects, oldest first (ADR-0041 D2).
    ///
    /// An empty `project_ids` reads nothing, as every History read does.
    ///
    /// # Errors
    ///
    /// A storage error for a failed read, or a stored word this build does not recognize.
    pub fn history_trend(
        &self,
        project_ids: &[String],
        limit: i64,
    ) -> Result<StoredTrend, StorageError> {
        let limit = limit.clamp(1, MAX_TREND_LIMIT);
        if project_ids.is_empty() {
            return Ok(StoredTrend {
                points: Vec::new(),
                total_builds: 0,
                limit,
            });
        }
        let scope = in_placeholders(project_ids.len())?;
        let mut bound: Vec<&dyn ToSql> = project_ids.iter().map(|id| id as &dyn ToSql).collect();

        let total_builds: i64 = self
            .connection()
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM builds b
                       JOIN artifacts a ON a.id = b.id || '#0'
                       JOIN memory_footprints m ON m.build_id = b.id
                      WHERE b.state = 'COMPLETE' AND b.project_id IN ({scope})"
                ),
                params_from_iter(bound.iter()),
                |row| row.get(0),
            )
            .map_err(read_err)?;

        bound.push(&limit);
        // Newest `limit` by import order (stored time, then insertion order), turned oldest first.
        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT * FROM (
                     SELECT b.id, b.snapshot_id, a.path, b.created_at,
                            m.nonvolatile_state, m.nonvolatile_bytes, m.runtime_state, m.runtime_bytes,
                            b.rowid AS seq
                       FROM builds b
                       JOIN artifacts a ON a.id = b.id || '#0'
                       JOIN memory_footprints m ON m.build_id = b.id
                      WHERE b.state = 'COMPLETE' AND b.project_id IN ({scope})
                      ORDER BY b.created_at DESC, b.rowid DESC
                      LIMIT ?)
                  ORDER BY created_at ASC, seq ASC"
            ))
            .map_err(read_err)?;
        let mut points: Vec<StoredTrendPoint> = stmt
            .query_map(params_from_iter(bound.iter()), |row| {
                Ok(StoredTrendPoint {
                    build_id: row.get(0)?,
                    snapshot_id: row.get(1)?,
                    file_name: display_file_name(&row.get::<_, String>(2)?),
                    stored_at: row.get(3)?,
                    nonvolatile: StoredBudget::from_parts(row.get(4)?, row.get(5)?),
                    runtime_ram: StoredBudget::from_parts(row.get(6)?, row.get(7)?),
                    regions: TrendRegions::NotRecorded,
                    objects: None,
                    gate_disposition: None,
                    release_versions: Vec::new(),
                    pinned: false,
                })
            })
            .map_err(read_err)?
            .collect::<Result<_, _>>()
            .map_err(read_err)?;
        if points.is_empty() {
            return Ok(StoredTrend {
                points,
                total_builds,
                limit,
            });
        }

        let ids: Vec<String> = points.iter().map(|p| p.build_id.clone()).collect();
        let mut regions = self.trend_regions(&ids)?;
        let mut objects = self.trend_objects(&ids)?;
        let gates = self.trend_gates(&ids)?;
        let mut releases = self.trend_releases(&ids)?;
        let mut pinned: Vec<String> = Vec::new();
        for project in project_ids {
            if let Some(pin) = self.current_baseline_pin(project)? {
                pinned.push(pin.build_id);
            }
        }

        for point in &mut points {
            if let Some(found) = regions.remove(&point.build_id) {
                point.regions = found;
            }
            point.objects = objects.remove(&point.build_id);
            point.gate_disposition = gates.get(&point.build_id).cloned();
            point.release_versions = releases.remove(&point.build_id).unwrap_or_default();
            point.pinned = pinned.contains(&point.build_id);
        }

        Ok(StoredTrend {
            points,
            total_builds,
            limit,
        })
    }

    fn trend_regions(&self, ids: &[String]) -> Result<HashMap<String, TrendRegions>, StorageError> {
        let scope = in_placeholders(ids.len())?;
        let mut sets: HashMap<String, TrendRegions> = HashMap::new();
        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT build_id, region_count FROM region_usage_sets WHERE build_id IN ({scope})"
            ))
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params_from_iter(ids.iter()), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(read_err)?;
        for row in rows {
            let (build_id, count) = row.map_err(read_err)?;
            sets.insert(
                build_id,
                if count == 0 {
                    TrendRegions::NoRegionTable
                } else {
                    TrendRegions::Recorded(Vec::new())
                },
            );
        }

        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT build_id, name, kind, capacity_bytes, used_bytes, state
                   FROM region_usage WHERE build_id IN ({scope})
                  ORDER BY build_id, ordinal"
            ))
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params_from_iter(ids.iter()), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(read_err)?;
        for row in rows {
            let (build_id, name, kind, capacity, used, state) = row.map_err(read_err)?;
            let kind = match kind.as_str() {
                "nonvolatile" => RegionKind::Nonvolatile,
                "volatile_ram" => RegionKind::VolatileRam,
                "unknown" => RegionKind::Unknown,
                other => {
                    return Err(StorageError::Invariant {
                        detail: format!("unrecognized stored region kind {other:?}"),
                    });
                }
            };
            if let Some(TrendRegions::Recorded(list)) = sets.get_mut(&build_id) {
                list.push(TrendRegion {
                    name,
                    kind,
                    capacity: capacity.and_then(|v| u64::try_from(v).ok()),
                    used: used.and_then(|v| u64::try_from(v).ok()),
                    // A partial usage is a floor (ADR-0041 D3); only `exact` may become headroom.
                    exact: state == "exact",
                });
            }
        }
        Ok(sets)
    }

    /// Per build, per object: the summed charge of each budget. A sum over a row with no charge is
    /// Unknown, never a smaller number.
    fn trend_objects(
        &self,
        ids: &[String],
    ) -> Result<HashMap<String, Vec<TrendObject>>, StorageError> {
        let scope = in_placeholders(ids.len())?;
        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT build_id, object_key,
                        SUM(nonvolatile_bytes), COUNT(*) - COUNT(nonvolatile_bytes),
                        SUM(runtime_ram_bytes), COUNT(*) - COUNT(runtime_ram_bytes)
                   FROM object_contributions WHERE build_id IN ({scope})
                  GROUP BY build_id, object_key
                  ORDER BY build_id, object_key"
            ))
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params_from_iter(ids.iter()), |row| {
                let sum = |value: Option<i64>, missing: i64| {
                    if missing > 0 {
                        None
                    } else {
                        value.and_then(|v| u64::try_from(v).ok()).or(Some(0))
                    }
                };
                Ok((
                    row.get::<_, String>(0)?,
                    TrendObject {
                        object: row.get(1)?,
                        nonvolatile: sum(row.get(2)?, row.get(3)?),
                        runtime_ram: sum(row.get(4)?, row.get(5)?),
                    },
                ))
            })
            .map_err(read_err)?;
        let mut out: HashMap<String, Vec<TrendObject>> = HashMap::new();
        for row in rows {
            let (build_id, object) = row.map_err(read_err)?;
            out.entry(build_id).or_default().push(object);
        }
        Ok(out)
    }

    fn trend_gates(&self, ids: &[String]) -> Result<HashMap<String, String>, StorageError> {
        let scope = in_placeholders(ids.len())?;
        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT build_id, overall_effective_severity FROM gate_runs
                  WHERE build_id IN ({scope})
                  ORDER BY created_at ASC, rowid ASC"
            ))
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params_from_iter(ids.iter()), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(read_err)?;
        let mut out = HashMap::new();
        for row in rows {
            // Ascending order: the newest run for a build is the one left standing.
            let (build_id, disposition) = row.map_err(read_err)?;
            out.insert(build_id, disposition);
        }
        Ok(out)
    }

    fn trend_releases(
        &self,
        ids: &[String],
    ) -> Result<BTreeMap<String, Vec<String>>, StorageError> {
        let scope = in_placeholders(ids.len())?;
        let mut stmt = self
            .connection()
            .prepare(&format!(
                "SELECT build_id, release_version FROM release_records
                  WHERE build_id IN ({scope})
                  ORDER BY created_at ASC, rowid ASC"
            ))
            .map_err(read_err)?;
        let rows = stmt
            .query_map(params_from_iter(ids.iter()), |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(read_err)?;
        let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for row in rows {
            let (build_id, version) = row.map_err(read_err)?;
            out.entry(build_id).or_default().push(version);
        }
        Ok(out)
    }
}
