//! History trends (ADR-0041 D3): the series a person reads across stored builds.
//!
//! Pure and synchronous. The input is what the store holds for each build, oldest first; the output is
//! every number the Trends view draws, so the UI decides no delta, headroom or ranking (AGENTS.md §3).
//!
//! Three rules shape every series:
//!
//! - **Unknown is a gap.** A build whose value is Unknown has no value and no delta. Nothing is
//!   interpolated and nothing is drawn as zero.
//! - **A delta needs two exact values.** It is measured from the previous build whose value is known, and
//!   only when both values are exact; a partial total is a floor, and the difference of two floors is not
//!   a change anyone measured.
//! - **Absence is told apart from ignorance.** A build stored before region usage was recorded, a build
//!   with no region table, and a build whose region table lacks this region are three different cells.

use std::collections::BTreeMap;

use crate::domain::memory::RegionKind;

/// One budget total as stored: its bytes when known, and whether they are exact or a floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrendBudget {
    pub bytes: Option<u64>,
    pub exact: bool,
}

impl TrendBudget {
    #[must_use]
    pub const fn exact(bytes: u64) -> Self {
        Self {
            bytes: Some(bytes),
            exact: true,
        }
    }

    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            bytes: None,
            exact: false,
        }
    }
}

/// One stored region of one build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendRegion {
    pub name: String,
    pub kind: RegionKind,
    pub capacity: Option<u64>,
    pub used: Option<u64>,
    /// Whether `used` is exact. A partial usage is a floor: the region holds at least that much, so
    /// `capacity - used` is only an upper bound on the room left and is never reported as headroom.
    pub exact: bool,
}

/// What the store says about a build's regions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrendRegions {
    /// Stored before region usage was recorded (migration 0009). Unknown.
    NotRecorded,
    /// Recorded, and the build had no region table.
    NoRegionTable,
    /// The recorded regions, in the analysis's order.
    Recorded(Vec<TrendRegion>),
}

/// One object's stored bytes in one build. `None` is Unknown: at least one of its rows had no charge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendObject {
    pub object: String,
    pub nonvolatile: Option<u64>,
    pub runtime_ram: Option<u64>,
}

/// Everything the trend needs from one stored build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendBuild {
    pub nonvolatile: TrendBudget,
    pub runtime_ram: TrendBudget,
    pub regions: TrendRegions,
    /// `None`: the build has no object rows (no MAP, or stored before migration 0007).
    pub objects: Option<Vec<TrendObject>>,
}

/// One point of a budget series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetPoint {
    pub bytes: Option<u64>,
    pub exact: bool,
    /// Change from the previous build whose value is known, when both are exact.
    pub delta: Option<i64>,
}

/// One region in one build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionCell {
    NotRecorded,
    NoRegionTable,
    /// The build recorded regions, but not this one.
    Absent,
    Recorded {
        capacity: Option<u64>,
        used: Option<u64>,
        /// `capacity - used`, when both are known, `used` is exact and fits. Never negative: an overflow is
        /// `None` with `exceeded` set.
        remaining: Option<u64>,
        /// `used > capacity`. True for a floor too: at least that much does not fit.
        exceeded: bool,
        /// Whether `used` is exact; a floor has no `remaining`.
        exact: bool,
    },
}

/// The newest recorded headroom of a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Headroom {
    /// Index of the build it was read from.
    pub build_index: usize,
    pub capacity: u64,
    pub used: u64,
    /// `None` when used exceeds capacity or is only a floor.
    pub remaining: Option<u64>,
    /// Remaining share of capacity, in tenths of a percent, rounded down. `None` with `remaining`.
    pub remaining_per_mille: Option<u32>,
    pub exceeded: bool,
    /// Whether `used` is exact. When false, `used` is a floor and no headroom is claimed.
    pub exact: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionSeries {
    pub name: String,
    pub kind: RegionKind,
    pub cells: Vec<RegionCell>,
    pub latest: Option<Headroom>,
}

/// One object's bytes across the builds, for one budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectSeries {
    pub object: String,
    pub values: Vec<Option<u64>>,
    /// The first and last known values, and their difference.
    pub first: Option<u64>,
    pub last: Option<u64>,
    pub delta: Option<i64>,
    /// Largest known value minus smallest: how much it moved.
    pub range: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trend {
    pub nonvolatile: Vec<BudgetPoint>,
    pub runtime_ram: Vec<BudgetPoint>,
    pub regions: Vec<RegionSeries>,
    pub movers_nonvolatile: Vec<ObjectSeries>,
    pub movers_runtime_ram: Vec<ObjectSeries>,
}

/// Which budget an object series reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Nonvolatile,
    RuntimeRam,
}

impl Trend {
    /// The trend over `builds`, oldest first, keeping the `movers` objects that moved most per budget.
    #[must_use]
    pub fn compute(builds: &[TrendBuild], movers: usize) -> Self {
        Self {
            nonvolatile: budget_series(builds.iter().map(|b| b.nonvolatile)),
            runtime_ram: budget_series(builds.iter().map(|b| b.runtime_ram)),
            regions: region_series(builds),
            movers_nonvolatile: object_movers(builds, Side::Nonvolatile, movers),
            movers_runtime_ram: object_movers(builds, Side::RuntimeRam, movers),
        }
    }
}

fn signed_delta(from: u64, to: u64) -> Option<i64> {
    i64::try_from(i128::from(to) - i128::from(from)).ok()
}

fn budget_series(values: impl Iterator<Item = TrendBudget>) -> Vec<BudgetPoint> {
    let mut previous: Option<TrendBudget> = None;
    values
        .map(|value| {
            let delta = match (previous, value.bytes) {
                (Some(prev), Some(now)) if prev.exact && value.exact => {
                    prev.bytes.and_then(|then| signed_delta(then, now))
                }
                _ => None,
            };
            if value.bytes.is_some() {
                previous = Some(value);
            }
            BudgetPoint {
                bytes: value.bytes,
                exact: value.exact && value.bytes.is_some(),
                delta,
            }
        })
        .collect()
}

fn region_series(builds: &[TrendBuild]) -> Vec<RegionSeries> {
    // First-seen order, so a region keeps its place as builds are added.
    let mut order: Vec<(String, RegionKind)> = Vec::new();
    for build in builds {
        if let TrendRegions::Recorded(regions) = &build.regions {
            for region in regions {
                if !order.iter().any(|(name, _)| *name == region.name) {
                    order.push((region.name.clone(), region.kind));
                }
            }
        }
    }

    order
        .into_iter()
        .map(|(name, kind)| {
            let cells: Vec<RegionCell> = builds
                .iter()
                .map(|build| match &build.regions {
                    TrendRegions::NotRecorded => RegionCell::NotRecorded,
                    TrendRegions::NoRegionTable => RegionCell::NoRegionTable,
                    TrendRegions::Recorded(regions) => regions
                        .iter()
                        .find(|region| region.name == name)
                        .map_or(RegionCell::Absent, |region| {
                            let (remaining, exceeded) = match (region.capacity, region.used) {
                                (Some(capacity), Some(used)) => (
                                    capacity.checked_sub(used).filter(|_| region.exact),
                                    used > capacity,
                                ),
                                _ => (None, false),
                            };
                            RegionCell::Recorded {
                                capacity: region.capacity,
                                used: region.used,
                                remaining,
                                exceeded,
                                exact: region.exact,
                            }
                        }),
                })
                .collect();
            let latest = cells
                .iter()
                .enumerate()
                .rev()
                .find_map(|(index, cell)| match *cell {
                    RegionCell::Recorded {
                        capacity: Some(capacity),
                        used: Some(used),
                        remaining,
                        exceeded,
                        exact,
                    } => Some(Headroom {
                        build_index: index,
                        capacity,
                        used,
                        remaining,
                        remaining_per_mille: remaining.and_then(|left| per_mille(left, capacity)),
                        exceeded,
                        exact,
                    }),
                    _ => None,
                });
            RegionSeries {
                name,
                kind,
                cells,
                latest,
            }
        })
        .collect()
}

fn per_mille(part: u64, whole: u64) -> Option<u32> {
    if whole == 0 {
        return None;
    }
    u32::try_from(u128::from(part) * 1000 / u128::from(whole)).ok()
}

fn object_movers(builds: &[TrendBuild], side: Side, keep: usize) -> Vec<ObjectSeries> {
    // Every object any build names, sorted by key so ties break the same way on every read.
    let mut names: BTreeMap<&str, ()> = BTreeMap::new();
    for objects in builds.iter().filter_map(|b| b.objects.as_ref()) {
        for object in objects {
            names.insert(object.object.as_str(), ());
        }
    }

    let mut series: Vec<ObjectSeries> = names
        .keys()
        .map(|name| {
            let values: Vec<Option<u64>> = builds
                .iter()
                .map(|build| {
                    let objects = build.objects.as_ref()?;
                    // The MAP lists every object the linker placed, so one it does not list placed
                    // nothing: 0 there is an observation, not a guess.
                    objects
                        .iter()
                        .find(|o| o.object == *name)
                        .map_or(Some(0), |o| match side {
                            Side::Nonvolatile => o.nonvolatile,
                            Side::RuntimeRam => o.runtime_ram,
                        })
                })
                .collect();
            let known: Vec<u64> = values.iter().flatten().copied().collect();
            let first = known.first().copied();
            let last = known.last().copied();
            let range = match (known.iter().max(), known.iter().min()) {
                (Some(max), Some(min)) => max - min,
                _ => 0,
            };
            ObjectSeries {
                object: (*name).to_owned(),
                delta: match (first, last) {
                    (Some(first), Some(last)) if known.len() >= 2 => signed_delta(first, last),
                    _ => None,
                },
                values,
                first,
                last,
                range,
            }
        })
        .filter(|series| series.range > 0)
        .collect();

    series.sort_by(|a, b| b.range.cmp(&a.range).then_with(|| a.object.cmp(&b.object)));
    series.truncate(keep);
    series
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(nv: TrendBudget, ram: TrendBudget) -> TrendBuild {
        TrendBuild {
            nonvolatile: nv,
            runtime_ram: ram,
            regions: TrendRegions::NotRecorded,
            objects: None,
        }
    }

    fn region(name: &str, capacity: u64, used: u64) -> TrendRegion {
        TrendRegion {
            name: name.to_owned(),
            kind: RegionKind::VolatileRam,
            capacity: Some(capacity),
            used: Some(used),
            exact: true,
        }
    }

    fn object(name: &str, nv: Option<u64>) -> TrendObject {
        TrendObject {
            object: name.to_owned(),
            nonvolatile: nv,
            runtime_ram: Some(0),
        }
    }

    #[test]
    fn an_unknown_build_is_a_gap_and_the_delta_skips_it() {
        let trend = Trend::compute(
            &[
                build(TrendBudget::exact(100), TrendBudget::exact(10)),
                build(TrendBudget::unknown(), TrendBudget::exact(12)),
                build(TrendBudget::exact(130), TrendBudget::exact(9)),
            ],
            5,
        );
        assert_eq!(
            trend.nonvolatile[0].delta, None,
            "the first build has nothing to differ from"
        );
        assert_eq!(trend.nonvolatile[1].bytes, None, "Unknown is never 0");
        assert_eq!(trend.nonvolatile[1].delta, None);
        assert_eq!(
            trend.nonvolatile[2].delta,
            Some(30),
            "measured from the last known build"
        );
        assert_eq!(trend.runtime_ram[2].delta, Some(-3));
    }

    #[test]
    fn a_partial_total_is_a_floor_and_carries_no_delta() {
        let floor = TrendBudget {
            bytes: Some(120),
            exact: false,
        };
        let trend = Trend::compute(
            &[
                build(TrendBudget::exact(100), TrendBudget::unknown()),
                build(floor, TrendBudget::unknown()),
                build(TrendBudget::exact(110), TrendBudget::unknown()),
            ],
            5,
        );
        assert_eq!(trend.nonvolatile[1].bytes, Some(120));
        assert!(!trend.nonvolatile[1].exact);
        assert_eq!(trend.nonvolatile[1].delta, None);
        assert_eq!(
            trend.nonvolatile[2].delta, None,
            "the previous known value was a floor"
        );
    }

    #[test]
    fn region_cells_keep_not_recorded_no_table_and_absent_apart() {
        let mut builds = vec![
            build(TrendBudget::unknown(), TrendBudget::unknown()),
            build(TrendBudget::unknown(), TrendBudget::unknown()),
            build(TrendBudget::unknown(), TrendBudget::unknown()),
            build(TrendBudget::unknown(), TrendBudget::unknown()),
        ];
        builds[1].regions = TrendRegions::NoRegionTable;
        builds[2].regions =
            TrendRegions::Recorded(vec![region("RAM", 1000, 600), region("CCM", 64, 0)]);
        builds[3].regions = TrendRegions::Recorded(vec![region("RAM", 1000, 940)]);
        let trend = Trend::compute(&builds, 5);
        let ram = &trend.regions[0];
        assert_eq!(ram.name, "RAM");
        assert_eq!(ram.cells[0], RegionCell::NotRecorded);
        assert_eq!(ram.cells[1], RegionCell::NoRegionTable);
        let latest = ram.latest.expect("headroom from the newest recorded build");
        assert_eq!(latest.build_index, 3);
        assert_eq!(latest.remaining, Some(60));
        assert_eq!(latest.remaining_per_mille, Some(60));
        let ccm = &trend.regions[1];
        assert_eq!(ccm.cells[3], RegionCell::Absent);
        assert_eq!(ccm.latest.map(|h| h.build_index), Some(2));
    }

    #[test]
    fn an_exceeded_region_has_no_remaining_and_says_so() {
        let mut only = build(TrendBudget::unknown(), TrendBudget::unknown());
        only.regions = TrendRegions::Recorded(vec![region("RAM", 100, 120)]);
        let trend = Trend::compute(&[only], 5);
        assert_eq!(
            trend.regions[0].cells[0],
            RegionCell::Recorded {
                capacity: Some(100),
                used: Some(120),
                remaining: None,
                exceeded: true,
                exact: true,
            }
        );
        let latest = trend.regions[0].latest.expect("headroom");
        assert_eq!((latest.remaining, latest.remaining_per_mille), (None, None));
        assert!(latest.exceeded);
    }

    #[test]
    fn a_partial_region_claims_no_headroom() {
        // 600 bytes accounted for is a floor: the room left is at most 400, never "400 left".
        let mut only = build(TrendBudget::unknown(), TrendBudget::unknown());
        let mut floor = region("RAM", 1000, 600);
        floor.exact = false;
        only.regions = TrendRegions::Recorded(vec![floor]);
        let trend = Trend::compute(&[only], 5);
        let latest = trend.regions[0]
            .latest
            .expect("the floor is still a recorded fact");
        assert_eq!((latest.used, latest.capacity), (600, 1000));
        assert_eq!((latest.remaining, latest.remaining_per_mille), (None, None));
        assert!(!latest.exact && !latest.exceeded);
    }

    #[test]
    fn movers_rank_by_range_and_a_build_without_objects_is_a_gap() {
        let mut builds = vec![
            build(TrendBudget::unknown(), TrendBudget::unknown()),
            build(TrendBudget::unknown(), TrendBudget::unknown()),
            build(TrendBudget::unknown(), TrendBudget::unknown()),
        ];
        builds[0].objects = Some(vec![object("main.o", Some(100)), object("imu.o", Some(10))]);
        // builds[1] has no object rows: a gap, never 0.
        builds[2].objects = Some(vec![object("main.o", Some(104)), object("spi.o", Some(30))]);
        let trend = Trend::compute(&builds, 2);
        let names: Vec<&str> = trend
            .movers_nonvolatile
            .iter()
            .map(|s| s.object.as_str())
            .collect();
        assert_eq!(
            names,
            ["spi.o", "imu.o"],
            "largest range first, top two kept"
        );
        let spi = &trend.movers_nonvolatile[0];
        assert_eq!(
            spi.values,
            vec![Some(0), None, Some(30)],
            "absent from a MAP is 0; no MAP is a gap"
        );
        assert_eq!(spi.delta, Some(30));
        assert!(trend.movers_runtime_ram.is_empty(), "nothing moved in RAM");
    }

    #[test]
    fn an_object_with_an_unknown_charge_is_unknown_in_that_build() {
        let mut builds = vec![
            build(TrendBudget::unknown(), TrendBudget::unknown()),
            build(TrendBudget::unknown(), TrendBudget::unknown()),
        ];
        builds[0].objects = Some(vec![object("a.o", None)]);
        builds[1].objects = Some(vec![object("a.o", Some(5))]);
        let trend = Trend::compute(&builds, 5);
        assert!(
            trend.movers_nonvolatile.is_empty(),
            "one known value is no movement"
        );
    }
}
