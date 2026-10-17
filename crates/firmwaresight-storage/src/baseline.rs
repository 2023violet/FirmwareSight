//! The pinned baseline (R5 / D3): one stored build a project keeps as Compare's measurement reference.
//!
//! A pin is a measurement concept, not a bookmark (`03_DESIGN/08_POST_MVP_PRESENTATION_RULES.md` §8). Three
//! rules decide this module:
//!
//! 1. **One per project, at any moment.** The current pin is the newest event for the project when that
//!    event is a `pin`; anything else means there is none.
//! 2. **Nothing is overwritten.** Pinning and unpinning append to `baseline_pin_events`, whose rows the
//!    schema refuses to edit or delete (migration 0008), so an unpin is itself a record.
//! 3. **Only a complete build of the project can be pinned.** A build still importing, a failed one, or a
//!    build of another project is refused as not found, never pinned "for later".
//!
//! Like Compare, nothing here opens an artifact or returns a path: a pin is read back as the build's ids,
//! file name and digest.

use rusqlite::{OptionalExtension, params};

use crate::Database;
use crate::db::write_err;
use crate::error::StorageError;
use crate::query::read_err;
use firmwaresight_core::domain::diff::display_file_name;

/// The build a project has pinned, and when it was pinned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselinePin {
    pub build_id: String,
    pub snapshot_id: String,
    /// The artifact's file name, never its directory (`AGENTS.md` 7).
    pub file_name: String,
    pub sha256: String,
    /// When the pin event that is current now was recorded (UTC, `YYYY-MM-DDTHH:MM:SSZ`).
    pub pinned_at: String,
}

impl Database {
    /// The project's current pin, or `None` when it has never pinned one or last unpinned it.
    ///
    /// # Errors
    ///
    /// [`StorageError::Write`] when the read fails (the crate's read errors share that variant).
    pub fn current_baseline_pin(
        &self,
        project_id: &str,
    ) -> Result<Option<BaselinePin>, StorageError> {
        let newest = self
            .connection()
            .query_row(
                "SELECT action, build_id, recorded_at FROM baseline_pin_events
                  WHERE project_id = ?1 ORDER BY seq DESC LIMIT 1",
                params![project_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(read_err)?;
        let Some((action, Some(build_id), pinned_at)) = newest else {
            return Ok(None);
        };
        if action != "pin" {
            return Ok(None);
        }
        self.connection()
            .query_row(
                "SELECT b.snapshot_id, a.path, a.sha256 FROM builds b
                   JOIN artifacts a ON a.id = b.id || '#0'
                  WHERE b.id = ?1",
                params![build_id],
                |row| {
                    Ok(BaselinePin {
                        build_id: build_id.clone(),
                        snapshot_id: row.get(0)?,
                        file_name: display_file_name(&row.get::<_, String>(1)?),
                        sha256: row.get(2)?,
                        pinned_at: pinned_at.clone(),
                    })
                },
            )
            .optional()
            .map_err(read_err)
    }

    /// Pin the project's complete build that carries `snapshot_id` and return the pin now current.
    ///
    /// Pinning the build that is already pinned writes nothing and returns the existing pin, so a second
    /// press does not move the recorded time.
    ///
    /// # Errors
    ///
    /// - [`StorageError::NotFound`] when no complete build of this project carries the snapshot id;
    /// - [`StorageError::Write`] when the read or the append fails.
    pub fn pin_baseline(
        &self,
        project_id: &str,
        snapshot_id: &str,
    ) -> Result<BaselinePin, StorageError> {
        self.immediate(|| self.pin_baseline_unlocked(project_id, snapshot_id))
    }

    fn pin_baseline_unlocked(
        &self,
        project_id: &str,
        snapshot_id: &str,
    ) -> Result<BaselinePin, StorageError> {
        let build_id = self
            .connection()
            .query_row(
                "SELECT id FROM builds
                  WHERE project_id = ?1 AND snapshot_id = ?2 AND state = 'COMPLETE'
                  ORDER BY created_at DESC, id ASC LIMIT 1",
                params![project_id, snapshot_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(read_err)?
            .ok_or_else(|| StorageError::NotFound {
                id: snapshot_id.to_owned(),
            })?;
        if let Some(current) = self.current_baseline_pin(project_id)?
            && current.build_id == build_id
        {
            return Ok(current);
        }
        self.connection()
            .execute(
                "INSERT INTO baseline_pin_events (project_id, action, build_id) VALUES (?1, 'pin', ?2)",
                params![project_id, build_id],
            )
            .map_err(write_err)?;
        self.current_baseline_pin(project_id)?
            .ok_or_else(|| StorageError::Invariant {
                detail: "a pin that was just appended is not the current pin".to_owned(),
            })
    }

    /// Unpin the project's baseline. Returns `false`, and writes nothing, when nothing was pinned.
    ///
    /// # Errors
    ///
    /// [`StorageError::Write`] when the read or the append fails.
    pub fn unpin_baseline(&self, project_id: &str) -> Result<bool, StorageError> {
        self.immediate(|| {
            if self.current_baseline_pin(project_id)?.is_none() {
                return Ok(false);
            }
            self.connection()
                .execute(
                    "INSERT INTO baseline_pin_events (project_id, action, build_id) VALUES (?1, 'unpin', NULL)",
                    params![project_id],
                )
                .map_err(write_err)?;
            Ok(true)
        })
    }

    /// Run a read-then-append as one `BEGIN IMMEDIATE` transaction, so a second writer on the same file
    /// (the CLI beside the desktop) cannot interleave between "what is pinned now" and the new row (R5
    /// review S3). A failure rolls the append back.
    fn immediate<T>(
        &self,
        work: impl FnOnce() -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        self.connection()
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(write_err)?;
        match work() {
            Ok(value) => {
                self.connection()
                    .execute_batch("COMMIT")
                    .map_err(write_err)?;
                Ok(value)
            }
            Err(err) => {
                // The original error is the one worth reporting; a failed rollback leaves SQLite to undo
                // the open transaction when the connection closes.
                let _ = self.connection().execute_batch("ROLLBACK");
                Err(err)
            }
        }
    }
}
