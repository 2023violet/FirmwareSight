//! The intake edge of the desktop shell: the only place a FirmwareSight process asks the
//! operating system for a file the user chose.
//!
//! Two rules hold here, and they are the reason this module exists separately.
//!
//! 1. **The dialog is opened from Rust.** `tauri-plugin-dialog` is registered so the shell can call
//!    it, not so the WebView can. The plugin's JS-facing commands are never granted by the
//!    capability, which stays `core:default` (`ADR-0025`, `AGENTS.md` 7): the front end gains no
//!    filesystem, shell or network surface, and there is still no command that takes a path.
//! 2. **Cancel is not an error.** A dialog that returns nothing means the user decided not to pick
//!    a file. That is a normal outcome, so it travels as `Ok(None)` rather than as an envelope the
//!    UI would have to render as a failure.
//!
//! `on_artifact_picked` and `on_map_picked` are pure: they take the path the dialog produced and
//! hand it to the session. The tests exercise those two directly, which is how the cancel and
//! typed-error behaviour is proven without opening a window.

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::ipc::{AnalysisSummaryDto, ErrorEnvelopeDto, SelectionDto};
use crate::{Session, next_operation_id, task_join_error};

/// Open the native file dialog and report what came back.
///
/// `rfd` runs a modal dialog, so it never executes on the WebView event loop: every command hops
/// through `spawn_blocking` before reaching this function.
fn pick_path(app: &AppHandle, title: &str) -> Option<PathBuf> {
    app.dialog()
        .file()
        .set_title(title)
        .blocking_pick_file()
        .and_then(|picked| picked.into_path().ok())
}

/// Turn "the user chose a file" into a staged selection.
///
/// `None` is the Cancel path: nothing is staged, no error is raised, and whatever the screen was
/// already showing stays on screen.
///
/// # Errors
///
/// Propagates the session's typed envelope when the chosen path cannot be staged.
pub fn on_artifact_picked(
    session: &Arc<Session>,
    picked: Option<PathBuf>,
    operation_id: &str,
) -> Result<Option<SelectionDto>, ErrorEnvelopeDto> {
    let Some(path) = picked else {
        return Ok(None);
    };
    Ok(Some(session.stage_artifact(path, operation_id)?))
}

/// Turn "the user chose a MAP" into an attachment on an existing selection.
///
/// # Errors
///
/// Propagates the session's typed envelope when the selection is unknown or the MAP cannot be read.
pub fn on_map_picked(
    session: &Session,
    selection_id: &str,
    picked: Option<PathBuf>,
    operation_id: &str,
) -> Result<Option<SelectionDto>, ErrorEnvelopeDto> {
    let Some(path) = picked else {
        return Ok(None);
    };
    Ok(Some(session.stage_map(selection_id, path, operation_id)?))
}

/// Choose the firmware artifact. Resolves to `None` when the user cancels.
#[tauri::command]
pub(crate) async fn select_artifact(
    app: AppHandle,
    state: State<'_, Arc<Session>>,
) -> Result<Option<SelectionDto>, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    let picked = tauri::async_runtime::spawn_blocking(move || {
        pick_path(&app, "Choose the firmware artifact to analyze")
    })
    .await
    .map_err(task_join_error)?;

    on_artifact_picked(&session, picked, &operation_id)
}

/// Attach a GNU ld MAP to the current selection. Resolves to `None` when the user cancels.
#[tauri::command]
pub(crate) async fn attach_map(
    app: AppHandle,
    selection_id: String,
    state: State<'_, Arc<Session>>,
) -> Result<Option<SelectionDto>, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    let picked = tauri::async_runtime::spawn_blocking(move || {
        pick_path(&app, "Choose the linker MAP for this artifact")
    })
    .await
    .map_err(task_join_error)?;

    on_map_picked(&session, &selection_id, picked, &operation_id)
}

/// Detach the MAP. The artifact selection itself stays staged.
#[tauri::command]
pub(crate) fn clear_map(
    selection_id: String,
    state: State<'_, Arc<Session>>,
) -> Result<SelectionDto, ErrorEnvelopeDto> {
    state.clear_map(&selection_id, &next_operation_id())
}

/// Analyze the selected file with the pipeline P0 already validated.
#[tauri::command]
pub(crate) async fn analyze_selection(
    selection_id: String,
    state: State<'_, Arc<Session>>,
) -> Result<AnalysisSummaryDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();

    tauri::async_runtime::spawn_blocking(move || {
        session.analyze_selection(&selection_id, &operation_id)
    })
    .await
    .map_err(task_join_error)?
}
