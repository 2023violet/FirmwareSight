//! D7 (ADR-0039): the Analyze composition treemap and the Compare growth waterfall.
//!
//! Both are windows over facts Core decides: [`composition::build`] over a build this session holds,
//! and [`waterfall::growth_waterfall`] over a comparison it computed. This module picks the window
//! (a focus node, two levels, at most [`MAX_CHILDREN`] children) and maps it to DTOs; it never
//! computes a byte (`AGENTS.md` 3).

use std::sync::Arc;

use firmwaresight_core::domain::composition::{self, CompositionNode, NodeKind, Side};
use firmwaresight_core::domain::evidence::EvidenceClass;
use firmwaresight_core::domain::waterfall::{self, StepKind};
use tauri::State;

use crate::ipc::{
    BudgetSideDto, CompositionCrumbDto, CompositionDto, CompositionKindDto, CompositionNodeDto,
    CompositionRequestDto, ErrorEnvelopeDto, EvidenceClassDto, WaterfallDto, WaterfallRequestDto,
    WaterfallStepDto, WaterfallStepKindDto,
};
use crate::{Session, next_operation_id, task_join_error};

/// Levels sent below the focus node.
const WINDOW_DEPTH: usize = 2;
/// Children per node before the rest fold into one "N more" node (ADR-0039 D6).
pub const MAX_CHILDREN: usize = 200;
/// Waterfall steps by default, and at most.
const DEFAULT_TOP_N: usize = 12;
const MAX_TOP_N: usize = 50;

fn side(dto: BudgetSideDto) -> Side {
    match dto {
        BudgetSideDto::Flash => Side::Nonvolatile,
        BudgetSideDto::Ram => Side::Runtime,
    }
}

fn kind(kind: NodeKind) -> CompositionKindDto {
    match kind {
        NodeKind::Root => CompositionKindDto::Root,
        NodeKind::Region => CompositionKindDto::Region,
        NodeKind::NoRegion => CompositionKindDto::NoRegion,
        NodeKind::Section => CompositionKindDto::Section,
        NodeKind::Object => CompositionKindDto::Object,
        NodeKind::Symbol => CompositionKindDto::Symbol,
        NodeKind::LinkerBytes => CompositionKindDto::LinkerBytes,
        NodeKind::NoSymbol => CompositionKindDto::NoSymbol,
        NodeKind::More => CompositionKindDto::More,
    }
}

fn class(class: EvidenceClass) -> EvidenceClassDto {
    match class {
        EvidenceClass::Observed => EvidenceClassDto::Observed,
        EvidenceClass::Derived => EvidenceClassDto::Derived,
        EvidenceClass::Declared => EvidenceClassDto::Declared,
        EvidenceClass::Unknown => EvidenceClassDto::Unknown,
    }
}

fn node_dto(node: &CompositionNode) -> CompositionNodeDto {
    CompositionNodeDto {
        id: node.id.clone(),
        kind: kind(node.kind),
        name: node.name.clone(),
        bytes: node.bytes,
        classification: class(node.class),
        reason: node.reason.clone(),
        locator: node.locator.clone(),
        capacity: node.capacity,
        folded: node.folded,
        children: node.children.iter().map(node_dto).collect(),
    }
}

/// The nodes from the root down to (and including) `id`.
fn path_to<'a>(node: &'a CompositionNode, id: &str, out: &mut Vec<&'a CompositionNode>) -> bool {
    out.push(node);
    if node.id == id || node.children.iter().any(|c| path_to(c, id, out)) {
        return true;
    }
    out.pop();
    false
}

fn unavailable(side: BudgetSideDto, reason: &str) -> CompositionDto {
    CompositionDto {
        available: false,
        reason: Some(reason.to_owned()),
        side,
        path: Vec::new(),
        node: None,
    }
}

impl Session {
    /// One window of a held build's composition tree.
    ///
    /// # Errors
    ///
    /// An internal envelope for a poisoned lock. A build that is not held, or a focus that names no
    /// node, is an unavailable answer with its reason, not an error.
    pub fn query_composition(
        &self,
        request: &CompositionRequestDto,
        operation_id: &str,
    ) -> Result<CompositionDto, ErrorEnvelopeDto> {
        let Some(snapshot) = self.held_snapshot(&request.snapshot_id, operation_id)? else {
            return Ok(unavailable(
                request.side,
                "this build is not open in this session: analyze it again to explore its composition",
            ));
        };
        let Some(memory) = snapshot.memory() else {
            return Ok(unavailable(
                request.side,
                "this build carries no memory footprint",
            ));
        };
        let tree = composition::build(
            side(request.side),
            memory,
            snapshot.objects(),
            snapshot.symbols(),
        );
        let focus = request.focus.as_deref().unwrap_or(&tree.id);
        let mut path = Vec::new();
        if !path_to(&tree, focus, &mut path) {
            return Ok(unavailable(
                request.side,
                "that part of the tree no longer exists for this build; start again from the top",
            ));
        }
        let node = path.last().copied().unwrap_or(&tree);
        Ok(CompositionDto {
            available: true,
            reason: None,
            side: request.side,
            path: path
                .iter()
                .map(|n| CompositionCrumbDto {
                    id: n.id.clone(),
                    name: n.name.clone(),
                })
                .collect(),
            node: Some(node_dto(&node.window(WINDOW_DEPTH, MAX_CHILDREN))),
        })
    }

    /// The growth waterfall of a comparison this session computed.
    ///
    /// # Errors
    ///
    /// `ERR-DIFF-5002` when the diff id was never issued or has been evicted.
    pub fn query_growth_waterfall(
        &self,
        request: &WaterfallRequestDto,
        operation_id: &str,
    ) -> Result<WaterfallDto, ErrorEnvelopeDto> {
        let top_n = request.top_n.unwrap_or(DEFAULT_TOP_N).clamp(1, MAX_TOP_N);
        let fall = self.with_diff(&request.diff_id, operation_id, |diff| {
            waterfall::growth_waterfall(diff, side(request.side), request.libraries, top_n)
        })?;
        Ok(WaterfallDto {
            available: fall.available,
            reason: fall.reason,
            side: request.side,
            libraries: request.libraries,
            steps: fall
                .steps
                .into_iter()
                .map(|s| WaterfallStepDto {
                    kind: match s.kind {
                        StepKind::Start => WaterfallStepKindDto::Start,
                        StepKind::Item => WaterfallStepKindDto::Item,
                        StepKind::Rest => WaterfallStepKindDto::Rest,
                        StepKind::Linker => WaterfallStepKindDto::Linker,
                        StepKind::End => WaterfallStepKindDto::End,
                    },
                    label: s.label,
                    change: s.change.map(|c| c.label().to_ascii_lowercase()),
                    delta: s.delta,
                    total: s.total,
                    folded: s.folded,
                    reason: s.reason,
                })
                .collect(),
        })
    }
}

/// One window of a held build's composition tree.
#[tauri::command]
pub(crate) async fn query_composition(
    request: CompositionRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<CompositionDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || session.query_composition(&request, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// The growth waterfall of a computed comparison.
#[tauri::command]
pub(crate) async fn query_growth_waterfall(
    request: WaterfallRequestDto,
    state: State<'_, Arc<Session>>,
) -> Result<WaterfallDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || {
        session.query_growth_waterfall(&request, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}
