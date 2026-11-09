//! D7 over IPC (ADR-0039): the composition window of a held build and the growth waterfall of a
//! computed comparison, through the same `Session` methods the commands call.
//!
//! - a held build opens at the root with its children, bounded and conserved, and a focus opens one
//!   level down with the path back;
//! - a build this session does not hold is unavailable with a reason, never an empty tree;
//! - the waterfall of the M2 pair starts at the base total, ends at the target total, and its steps
//!   add up when every step is known.

use std::path::{Path, PathBuf};

use firmwaresight_desktop::composition::MAX_CHILDREN;
use firmwaresight_desktop::ipc::{
    BudgetSideDto, CompareRequestDto, CompositionKindDto, CompositionNodeDto,
    CompositionRequestDto, WaterfallRequestDto, WaterfallStepKindDto,
};
use firmwaresight_desktop::{Session, service::FixtureCatalog};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("<root>/apps/desktop/src-tauri")
        .to_path_buf()
}

struct TempDb(PathBuf);

impl TempDb {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "fwsight-d7-{label}-{}-{:x}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_file(&path);
        Self(path)
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let mut candidate = self.0.as_os_str().to_os_string();
            candidate.push(suffix);
            let _ = std::fs::remove_file(Path::new(&candidate));
        }
    }
}

fn store(session: &Session, side: &str) -> String {
    let dir = repo_root().join(format!("fixtures/elf/m2-objects/{side}"));
    let selection = session
        .stage_artifact(dir.join("firmware.elf"), "op-stage")
        .expect("stages");
    session
        .stage_map(&selection.selection_id, dir.join("firmware.map"), "op-map")
        .expect("map stages");
    session
        .analyze_selection(&selection.selection_id, "op-analyze")
        .expect("analyzes")
        .identity
        .snapshot_id
}

fn session(label: &str) -> (Session, TempDb) {
    let db = TempDb::new(label);
    let session =
        Session::open(FixtureCatalog::from_environment(), &db.0).expect("a session opens");
    (session, db)
}

fn conserved(node: &CompositionNodeDto) {
    if let Some(parent) = node.bytes
        && !node.children.is_empty()
        && node.children.iter().all(|c| c.bytes.is_some())
        && node.folded == 0
    {
        let sum: u64 = node.children.iter().filter_map(|c| c.bytes).sum();
        assert!(
            sum == parent || node.reason.is_some(),
            "{}: {sum} vs {parent}",
            node.id
        );
    }
    assert!(node.children.len() <= MAX_CHILDREN + 1);
    node.children.iter().for_each(conserved);
}

#[test]
fn a_held_build_opens_its_composition_and_drills_down_with_a_path_back() {
    let (session, _db) = session("open");
    let snapshot_id = store(&session, "target");
    let root = session
        .query_composition(
            &CompositionRequestDto {
                snapshot_id: snapshot_id.clone(),
                side: BudgetSideDto::Flash,
                focus: None,
            },
            "op-c1",
        )
        .expect("answers");
    assert!(root.available, "{:?}", root.reason);
    let node = root.node.expect("a root window");
    assert_eq!(node.kind, CompositionKindDto::Root);
    assert!(node.bytes.is_some_and(|b| b > 0));
    conserved(&node);
    assert_eq!(root.path.len(), 1);

    let child = node
        .children
        .iter()
        .find(|c| !c.children.is_empty())
        .expect("a child with children");
    let down = session
        .query_composition(
            &CompositionRequestDto {
                snapshot_id,
                side: BudgetSideDto::Flash,
                focus: Some(child.id.clone()),
            },
            "op-c2",
        )
        .expect("answers");
    assert_eq!(down.path.len(), 2);
    assert_eq!(down.node.expect("a window").id, child.id);
}

#[test]
fn a_build_this_session_does_not_hold_is_unavailable_with_a_reason() {
    let (session, _db) = session("absent");
    let answer = session
        .query_composition(
            &CompositionRequestDto {
                snapshot_id: "snap-not-held".into(),
                side: BudgetSideDto::Ram,
                focus: None,
            },
            "op-c3",
        )
        .expect("answers");
    assert!(!answer.available);
    assert!(
        answer
            .reason
            .is_some_and(|r| r.contains("analyze it again"))
    );
    assert!(answer.node.is_none());
}

#[test]
fn the_waterfall_of_the_m2_pair_runs_from_base_to_target() {
    let (session, _db) = session("waterfall");
    let base = store(&session, "base");
    let target = store(&session, "target");
    let summary = session
        .compare_snapshots(
            &CompareRequestDto {
                base_snapshot_id: base,
                target_snapshot_id: target,
            },
            "op-cmp",
        )
        .expect("the pair compares");
    let fall = session
        .query_growth_waterfall(
            &WaterfallRequestDto {
                diff_id: summary.diff_id.clone(),
                side: BudgetSideDto::Flash,
                libraries: false,
                top_n: Some(3),
            },
            "op-w",
        )
        .expect("answers");
    assert!(fall.available, "{:?}", fall.reason);
    let first = fall.steps.first().expect("a start");
    let last = fall.steps.last().expect("an end");
    assert_eq!(
        (first.kind, last.kind),
        (WaterfallStepKindDto::Start, WaterfallStepKindDto::End)
    );
    let moved: Option<i64> = fall
        .steps
        .iter()
        .filter(|s| {
            !matches!(
                s.kind,
                WaterfallStepKindDto::Start | WaterfallStepKindDto::End
            )
        })
        .map(|s| s.delta)
        .sum();
    if let (Some(moved), Some(b), Some(t)) = (moved, first.total, last.total) {
        assert_eq!(
            i128::from(b) + i128::from(moved),
            i128::from(t),
            "the steps reconcile the totals"
        );
    }
    assert!(
        fall.steps
            .iter()
            .filter(|s| s.kind == WaterfallStepKindDto::Item)
            .count()
            <= 3
    );
    assert!(
        fall.steps.iter().any(|s| s.label.contains("imu.o")),
        "{:?}",
        fall.steps
    );
}
