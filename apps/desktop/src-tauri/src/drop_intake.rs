//! Drag-and-drop intake: a file dropped on the window becomes a selection exactly as if it had been chosen
//! in the native dialog, and its path never leaves Rust.
//!
//! The window's `DragDrop` event hands Rust the dropped paths. They are classified here, held in the
//! session under an opaque token, and the WebView is told only the token, the file names and what each file
//! is taken to be. The UI then calls [`accept_dropped_files`] with that token and, for a MAP on its own, the
//! selection it is showing; Rust stages the held paths through the same `stage_artifact` / `stage_map` the
//! dialog uses. A token is single-use and a newer drop replaces an older one, so a late accept can never
//! stage files the reader has since dropped something else over.
//!
//! What is refused, before anything is staged: a directory, more than two files, and two files that are not
//! one MAP plus one other file. A dropped file that is not an ELF is staged like a dialog choice would be,
//! and Analyze reports the parser's typed refusal, so there is one place that decides what an ELF is.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use ts_rs::TS;

use crate::ipc::{ErrorEnvelopeDto, SelectionDto};
use crate::service::display_name;
use crate::{Session, next_operation_id};

/// The event name the window emits for every drag-and-drop phase.
pub const DROP_EVENT: &str = "fs-intake-drop";

/// What a drop was taken to contain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropPlan {
    Artifact(PathBuf),
    Map(PathBuf),
    ArtifactAndMap { artifact: PathBuf, map: PathBuf },
}

/// Why a drop was refused. The word is what the UI translates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropRefusal {
    Directory,
    TooMany,
    Ambiguous,
}

impl DropRefusal {
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::TooMany => "too_many",
            Self::Ambiguous => "ambiguous",
        }
    }
}

fn is_map(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("map"))
}

/// Classify dropped paths. `is_dir` is injected so the rule is testable without a filesystem.
pub fn classify(
    paths: &[PathBuf],
    is_dir: impl Fn(&Path) -> bool,
) -> Result<Option<DropPlan>, DropRefusal> {
    if paths.is_empty() {
        return Ok(None);
    }
    if paths.iter().any(|path| is_dir(path)) {
        return Err(DropRefusal::Directory);
    }
    match paths {
        [only] if is_map(only) => Ok(Some(DropPlan::Map(only.clone()))),
        [only] => Ok(Some(DropPlan::Artifact(only.clone()))),
        [first, second] => match (is_map(first), is_map(second)) {
            (true, false) => Ok(Some(DropPlan::ArtifactAndMap {
                artifact: second.clone(),
                map: first.clone(),
            })),
            (false, true) => Ok(Some(DropPlan::ArtifactAndMap {
                artifact: first.clone(),
                map: second.clone(),
            })),
            _ => Err(DropRefusal::Ambiguous),
        },
        _ => Err(DropRefusal::TooMany),
    }
}

/// One drag-and-drop phase as the WebView sees it: names and roles, never a path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DropEventDto {
    /// `hover`, `leave`, `dropped` or `refused`.
    pub phase: String,
    /// Present only for `dropped`; single-use.
    pub token: Option<String>,
    pub artifact_name: Option<String>,
    pub map_name: Option<String>,
    /// Present only for `refused`: `directory`, `too_many` or `ambiguous`.
    pub refusal: Option<String>,
}

impl DropEventDto {
    fn phase(phase: &str) -> Self {
        Self {
            phase: phase.to_owned(),
            token: None,
            artifact_name: None,
            map_name: None,
            refusal: None,
        }
    }

    #[must_use]
    pub fn hover() -> Self {
        Self::phase("hover")
    }

    #[must_use]
    pub fn leave() -> Self {
        Self::phase("leave")
    }
}

/// The drop held for the UI to accept.
#[derive(Debug, Default)]
pub struct PendingDrop {
    held: Option<(String, DropPlan)>,
    issued: u64,
}

impl Session {
    /// Classify and hold a drop, replacing any earlier one, and describe it for the WebView.
    pub fn hold_drop(&self, paths: &[PathBuf]) -> Option<DropEventDto> {
        let plan = match classify(paths, Path::is_dir) {
            Ok(None) => return None,
            Ok(Some(plan)) => plan,
            Err(refusal) => {
                if let Ok(mut pending) = self.drops.lock() {
                    pending.held = None;
                }
                let mut event = DropEventDto::phase("refused");
                event.refusal = Some(refusal.word().to_owned());
                return Some(event);
            }
        };
        let mut event = DropEventDto::phase("dropped");
        match &plan {
            DropPlan::Artifact(artifact) => event.artifact_name = Some(display_name(artifact)),
            DropPlan::Map(map) => event.map_name = Some(display_name(map)),
            DropPlan::ArtifactAndMap { artifact, map } => {
                event.artifact_name = Some(display_name(artifact));
                event.map_name = Some(display_name(map));
            }
        }
        let mut pending = self.drops.lock().ok()?;
        pending.issued += 1;
        let token = format!("drop-{}", pending.issued);
        pending.held = Some((token.clone(), plan));
        event.token = Some(token);
        Some(event)
    }

    /// Stage the held drop. A MAP on its own attaches to `selection_id`.
    ///
    /// # Errors
    ///
    /// `ERR-INPUT-0001` when the token is not the one held (it was used, or a newer drop replaced it),
    /// when a MAP arrives with no selection to attach to, or when staging itself refuses the file.
    pub fn accept_drop(
        &self,
        token: &str,
        selection_id: Option<&str>,
        operation_id: &str,
    ) -> Result<SelectionDto, ErrorEnvelopeDto> {
        let plan = {
            let mut pending = self
                .drops
                .lock()
                .map_err(|_| crate::internal_error(operation_id, "the drop lock was poisoned"))?;
            match pending.held.take() {
                Some((held, plan)) if held == token => plan,
                other => {
                    pending.held = other;
                    return Err(drop_stale(operation_id));
                }
            }
        };
        match plan {
            DropPlan::Artifact(artifact) => self.stage_artifact(artifact, operation_id),
            DropPlan::ArtifactAndMap { artifact, map } => {
                let staged = self.stage_artifact(artifact, operation_id)?;
                self.stage_map(&staged.selection_id, map, operation_id)
            }
            DropPlan::Map(map) => match selection_id {
                Some(selection_id) => self.stage_map(selection_id, map, operation_id),
                None => Err(map_without_artifact(operation_id)),
            },
        }
    }
}

fn drop_stale(operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-INPUT-0001".to_owned(),
        message: "That drop is no longer available.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some("a drop is single-use and a newer drop replaces it".to_owned()),
        remediation: Some("Drop the file again.".to_owned()),
    }
}

fn map_without_artifact(operation_id: &str) -> ErrorEnvelopeDto {
    ErrorEnvelopeDto {
        code: "ERR-INPUT-0001".to_owned(),
        message: "A MAP needs a firmware file to attach to.".to_owned(),
        operation_id: operation_id.to_owned(),
        details: Some("a MAP was dropped while no firmware file was selected".to_owned()),
        remediation: Some("Drop the ELF first, or drop the ELF and its MAP together.".to_owned()),
    }
}

/// Stage the files the window last received by drag and drop.
#[tauri::command]
pub(crate) fn accept_dropped_files(
    token: String,
    selection_id: Option<String>,
    state: State<'_, Arc<Session>>,
) -> Result<SelectionDto, ErrorEnvelopeDto> {
    state.accept_drop(&token, selection_id.as_deref(), &next_operation_id())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    fn no_dirs(_: &Path) -> bool {
        false
    }

    #[test]
    fn one_file_is_the_artifact_and_a_map_is_recognised_by_extension() {
        assert_eq!(
            classify(&[p("a.elf")], no_dirs),
            Ok(Some(DropPlan::Artifact(p("a.elf"))))
        );
        assert_eq!(
            classify(&[p("a.axf")], no_dirs),
            Ok(Some(DropPlan::Artifact(p("a.axf"))))
        );
        assert_eq!(
            classify(&[p("FW.MAP")], no_dirs),
            Ok(Some(DropPlan::Map(p("FW.MAP"))))
        );
    }

    #[test]
    fn an_elf_and_its_map_pair_in_either_order() {
        let pair = DropPlan::ArtifactAndMap {
            artifact: p("a.elf"),
            map: p("a.map"),
        };
        assert_eq!(
            classify(&[p("a.elf"), p("a.map")], no_dirs),
            Ok(Some(pair.clone()))
        );
        assert_eq!(classify(&[p("a.map"), p("a.elf")], no_dirs), Ok(Some(pair)));
    }

    #[test]
    fn unclear_drops_are_refused_before_anything_is_staged() {
        assert_eq!(
            classify(&[p("a.elf"), p("b.elf")], no_dirs),
            Err(DropRefusal::Ambiguous)
        );
        assert_eq!(
            classify(&[p("a.map"), p("b.map")], no_dirs),
            Err(DropRefusal::Ambiguous)
        );
        assert_eq!(
            classify(&[p("a"), p("b"), p("c.map")], no_dirs),
            Err(DropRefusal::TooMany)
        );
        assert_eq!(
            classify(&[p("dir"), p("a.map")], |path| path == Path::new("dir")),
            Err(DropRefusal::Directory)
        );
        assert_eq!(classify(&[], no_dirs), Ok(None));
    }
}
