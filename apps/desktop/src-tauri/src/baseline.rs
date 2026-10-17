//! The pinned baseline (R5 / D3): three use-case commands over one append-only log.
//!
//! A pin is the build this project measures against, and Compare offers it as its default base. The rules
//! follow the History layer's, with one addition:
//!
//! 1. **Rust fixes the scope.** The WebView never names a project: every command reads and writes the one
//!    project this application files user builds under (`AGENTS.md` 7).
//! 2. **The WebView names a build only by the snapshot id it already holds.** Storage resolves it to a
//!    complete build of that project or refuses it as not found; nothing here takes a path.
//! 3. **Nothing is erased.** Pin and unpin append to `baseline_pin_events`, which refuses an edit or a
//!    delete (migration 0008), so moving the reference leaves a record of where it was.
//!
//! These are the only write commands next to History's reads, and they live in their own module so
//! `history.rs` can keep saying it has none.

use std::sync::Arc;

use firmwaresight_storage::BaselinePin;
use tauri::State;

use crate::ipc::{BaselinePinDto, ErrorEnvelopeDto};
use crate::{LOCAL_PROJECT_ID, Session, envelope_from_storage, next_operation_id, task_join_error};

impl Session {
    /// The project's current pin, or `None`.
    ///
    /// # Errors
    ///
    /// The storage envelope for a failed read, or an internal envelope for a poisoned lock.
    pub fn baseline_pin(
        &self,
        operation_id: &str,
    ) -> Result<Option<BaselinePinDto>, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        db.current_baseline_pin(LOCAL_PROJECT_ID)
            .map(|pin| pin.map(pin_dto))
            .map_err(|err| envelope_from_storage(&err, operation_id))
    }

    /// Pin the stored build that carries `snapshot_id`.
    ///
    /// # Errors
    ///
    /// The storage envelope when no complete build of this project carries the snapshot id
    /// (`ERR-STORAGE-4005`) or the append fails.
    pub fn pin_baseline(
        &self,
        snapshot_id: &str,
        operation_id: &str,
    ) -> Result<BaselinePinDto, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        db.pin_baseline(LOCAL_PROJECT_ID, snapshot_id)
            .map(pin_dto)
            .map_err(|err| envelope_from_storage(&err, operation_id))
    }

    /// Unpin the project's baseline. `false` when nothing was pinned, in which case nothing is written.
    ///
    /// # Errors
    ///
    /// The storage envelope when the append fails.
    pub fn unpin_baseline(&self, operation_id: &str) -> Result<bool, ErrorEnvelopeDto> {
        let db = self.lock_db(operation_id)?;
        db.unpin_baseline(LOCAL_PROJECT_ID)
            .map_err(|err| envelope_from_storage(&err, operation_id))
    }
}

fn pin_dto(pin: BaselinePin) -> BaselinePinDto {
    BaselinePinDto {
        build_id: pin.build_id,
        snapshot_id: pin.snapshot_id,
        file_name: pin.file_name,
        sha256: pin.sha256,
        pinned_at: pin.pinned_at,
    }
}

/// Which stored build this project measures against, if any.
#[tauri::command]
pub(crate) async fn get_baseline_pin(
    state: State<'_, Arc<Session>>,
) -> Result<Option<BaselinePinDto>, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || session.baseline_pin(&operation_id))
        .await
        .map_err(task_join_error)?
}

/// Make one stored build, named by its snapshot id, the project's pinned baseline.
#[tauri::command]
pub(crate) async fn pin_baseline(
    snapshot_id: String,
    state: State<'_, Arc<Session>>,
) -> Result<BaselinePinDto, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || session.pin_baseline(&snapshot_id, &operation_id))
        .await
        .map_err(task_join_error)?
}

/// Unpin the project's baseline; the unpin is itself recorded.
#[tauri::command]
pub(crate) async fn unpin_baseline(
    state: State<'_, Arc<Session>>,
) -> Result<bool, ErrorEnvelopeDto> {
    let session = Arc::clone(&state);
    let operation_id = next_operation_id();
    tauri::async_runtime::spawn_blocking(move || session.unpin_baseline(&operation_id))
        .await
        .map_err(task_join_error)?
}
