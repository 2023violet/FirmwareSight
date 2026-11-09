//! Where a comparison's growth came from (ADR-0039 D4): base total → one step per object or library →
//! the folded rest → linker and unattributed → target total.
//!
//! Every figure is one the diff already decided ([`ObjectChange`], [`MemoryDiff`]); this only orders and
//! reconciles them. A step whose delta is Unknown breaks the arithmetic, so the reconciling step turns
//! Unknown with the reason instead of being forced to balance.

use super::composition::Side;
use super::diff::{ByteChange, ChangeKind, DiffResult, ObjectAttribution, ObjectChange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    /// The base total.
    Start,
    /// One object or library.
    Item,
    /// The objects past `top_n`, folded.
    Rest,
    /// Total delta minus every object delta: fill, stubs, sections without rows.
    Linker,
    /// The target total.
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaterfallStep {
    pub kind: StepKind,
    pub label: String,
    /// `Item` only.
    pub change: Option<ChangeKind>,
    /// The step's delta (`Item`, `Rest`, `Linker`); `None` is Unknown.
    pub delta: Option<i64>,
    /// The running total after this step (`Start`, `End`, and every step while it is known).
    pub total: Option<u64>,
    /// How many objects a `Rest` step folds.
    pub folded: usize,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waterfall {
    pub available: bool,
    pub reason: Option<String>,
    pub steps: Vec<WaterfallStep>,
}

fn side_change(change: &ObjectChange, side: Side) -> &ByteChange {
    match side {
        Side::Nonvolatile => &change.nonvolatile,
        Side::Runtime => &change.runtime_ram,
    }
}

/// A row's contribution to the total's change. An added object brought its whole target size and a
/// removed one took its whole base size away: the step is that size, while the table still shows the
/// absent side as "—" (absent is not zero as a value; it is as a share of a total).
fn step_delta(change: &ObjectChange, side: Side) -> Option<i64> {
    let c = side_change(change, side);
    match change.change {
        ChangeKind::Added => c.target.and_then(|t| i64::try_from(t).ok()),
        ChangeKind::Removed => c.base.and_then(|b| i64::try_from(b).ok()).map(|b| -b),
        ChangeKind::Changed => c.delta,
    }
}

/// The waterfall of one budget, by objects (`libraries == false`) or by libraries.
#[must_use]
pub fn growth_waterfall(diff: &DiffResult, side: Side, libraries: bool, top_n: usize) -> Waterfall {
    let total = match side {
        Side::Nonvolatile => diff.memory.nonvolatile.clone(),
        Side::Runtime => diff.memory.runtime_ram.clone(),
    };
    growth_waterfall_of(total, &diff.attribution, side, libraries, top_n)
}

fn growth_waterfall_of(
    total: ByteChange,
    attribution: &ObjectAttribution,
    side: Side,
    libraries: bool,
    top_n: usize,
) -> Waterfall {
    let total = &total;
    if !attribution.available {
        return Waterfall {
            available: false,
            reason: Some(attribution.reason.clone()),
            steps: Vec::new(),
        };
    }
    let rows = if libraries {
        &attribution.libraries
    } else {
        &attribution.objects
    };

    // moved rows only, largest |delta| first; Unknown deltas after the known ones; then by key
    let mut moved: Vec<&ObjectChange> = rows
        .iter()
        .filter(|r| step_delta(r, side) != Some(0))
        .collect();
    moved.sort_by(|a, b| {
        let (da, db) = (step_delta(a, side), step_delta(b, side));
        match (da, db) {
            (Some(x), Some(y)) => y
                .unsigned_abs()
                .cmp(&x.unsigned_abs())
                .then_with(|| a.key.cmp(&b.key)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.key.cmp(&b.key),
        }
    });

    let mut steps = Vec::new();
    let mut running = total.base;
    steps.push(WaterfallStep {
        kind: StepKind::Start,
        label: "base".to_owned(),
        change: None,
        delta: None,
        total: total.base,
        folded: 0,
        reason: None,
    });

    let advance = |running: Option<u64>, delta: Option<i64>| -> Option<u64> {
        let (r, d) = (running?, delta?);
        u64::try_from(i128::from(r) + i128::from(d)).ok()
    };
    let mut items_sum: Option<i64> = Some(0);
    for row in moved.iter().take(top_n) {
        let change = side_change(row, side);
        let delta = step_delta(row, side);
        running = advance(running, delta);
        items_sum = items_sum.zip(delta).and_then(|(a, b)| a.checked_add(b));
        steps.push(WaterfallStep {
            kind: StepKind::Item,
            label: row.key.clone(),
            change: Some(row.change),
            delta,
            total: running,
            folded: 0,
            reason: change.reason.clone(),
        });
    }
    let rest = &moved[moved.len().min(top_n)..];
    if !rest.is_empty() {
        let delta = rest.iter().try_fold(0_i64, |acc, r| {
            step_delta(r, side).and_then(|d| acc.checked_add(d))
        });
        running = advance(running, delta);
        items_sum = items_sum.zip(delta).and_then(|(a, b)| a.checked_add(b));
        steps.push(WaterfallStep {
            kind: StepKind::Rest,
            label: format!("{} more", rest.len()),
            change: None,
            delta,
            total: running,
            folded: rest.len(),
            reason: delta
                .is_none()
                .then(|| "at least one folded delta is Unknown".to_owned()),
        });
    }
    let linker = total
        .delta
        .zip(items_sum)
        .and_then(|(t, s)| t.checked_sub(s));
    if linker != Some(0) {
        running = if linker.is_some() { total.target } else { None };
        steps.push(WaterfallStep {
            kind: StepKind::Linker,
            label: "linker and unattributed".to_owned(),
            change: None,
            delta: linker,
            total: running,
            folded: 0,
            reason: Some(if linker.is_some() {
                "fill, linker stubs and sections no object row names".to_owned()
            } else {
                total.reason.clone().unwrap_or_else(|| "a total or an object delta is Unknown, so the remainder cannot be reconciled".to_owned())
            }),
        });
    }
    steps.push(WaterfallStep {
        kind: StepKind::End,
        label: "target".to_owned(),
        change: None,
        delta: None,
        total: total.target,
        folded: 0,
        reason: None,
    });
    Waterfall {
        available: true,
        reason: None,
        steps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::diff::{Comparability, ObjectAttribution};

    fn change(key: &str, kind: ChangeKind, base: Option<u64>, target: Option<u64>) -> ObjectChange {
        ObjectChange {
            change: kind,
            key: key.to_owned(),
            archive: None,
            nonvolatile: ByteChange::between(base, target, Comparability::Exact),
            runtime_ram: ByteChange::between(Some(0), Some(0), Comparability::Exact),
            ambiguous: false,
        }
    }

    fn waterfall(objects: Vec<ObjectChange>, base: u64, target: u64, top_n: usize) -> Waterfall {
        let attribution = ObjectAttribution {
            available: true,
            reason: String::new(),
            objects,
            libraries: Vec::new(),
            unchanged_objects: 0,
            unchanged_libraries: 0,
        };
        let total = ByteChange::between(Some(base), Some(target), Comparability::Exact);
        growth_waterfall_of(total, &attribution, Side::Nonvolatile, false, top_n)
    }

    fn summary(w: &Waterfall) -> Vec<(StepKind, String, Option<i64>, Option<u64>)> {
        w.steps
            .iter()
            .map(|s| (s.kind, s.label.clone(), s.delta, s.total))
            .collect()
    }

    #[test]
    fn the_armlink_pair_reconciles_to_the_linker() {
        // 108 → 192: imu.o +54, main.o +24, a removed file −2, the rest is the linker's
        let w = waterfall(
            vec![
                change("main.o", ChangeKind::Changed, Some(38), Some(62)),
                change("imu.o", ChangeKind::Changed, Some(38), Some(92)),
                change("old.o", ChangeKind::Removed, Some(2), None),
            ],
            108,
            192,
            10,
        );
        assert_eq!(
            summary(&w),
            vec![
                (StepKind::Start, "base".into(), None, Some(108)),
                (StepKind::Item, "imu.o".into(), Some(54), Some(162)),
                (StepKind::Item, "main.o".into(), Some(24), Some(186)),
                (StepKind::Item, "old.o".into(), Some(-2), Some(184)),
                (
                    StepKind::Linker,
                    "linker and unattributed".into(),
                    Some(8),
                    Some(192)
                ),
                (StepKind::End, "target".into(), None, Some(192)),
            ],
            "a removed object takes its whole base size out of the total"
        );
    }

    #[test]
    fn the_tail_folds_and_an_unknown_delta_makes_the_remainder_unknown() {
        let w = waterfall(
            vec![
                change("a.o", ChangeKind::Changed, Some(10), Some(30)),
                change("b.o", ChangeKind::Changed, Some(10), Some(15)),
                change("c.o", ChangeKind::Changed, Some(10), Some(12)),
            ],
            100,
            127,
            1,
        );
        assert_eq!(
            summary(&w)[2],
            (StepKind::Rest, "2 more".into(), Some(7), Some(127))
        );
        assert_eq!(
            w.steps.len(),
            4,
            "27 = 20 + 7: nothing left for the linker step"
        );

        let unknown = waterfall(
            vec![change("a.o", ChangeKind::Changed, None, Some(30))],
            100,
            130,
            5,
        );
        let linker = unknown
            .steps
            .iter()
            .find(|s| s.kind == StepKind::Linker)
            .expect("a remainder step");
        assert_eq!((linker.delta, linker.total), (None, None));
        assert_eq!(
            unknown.steps.last().map(|s| s.total),
            Some(Some(130)),
            "the target total still shows"
        );
    }
}
