//! The snapshot a store is owed before an upgrade changes its shape.
//!
//! `04_TECH/15_STORAGE_DATABASE_BASELINE.md` §8 has required "backup/recovery plan before destructive
//! operation" since the baseline was written, and `error.rs` has told a user since P3 to "restore a
//! backup" the product could not make. This module is the capability that message was pointing at.
//!
//! Two rules decide its shape:
//!
//! - **Consistent, not copied.** The store runs in WAL mode where the filesystem allows it, so the
//!   committed state is the `.sqlite` file *plus* whatever the write-ahead log still holds. A
//!   `std::fs::copy` of the file alone can miss those frames, so the snapshot is taken through
//!   SQLite's own online-backup API — the `backup` feature of the `rusqlite` dependency this crate
//!   already carries, which adds no package to the graph and is the reason `AGENTS.md` 6's frozen
//!   `rusqlite + bundled` choice still holds. The alternative that needs no feature at all,
//!   `VACUUM INTO`, binds its filename as SQL `TEXT`, so a store path that is not valid UTF-8 either
//!   fails or silently names a different file; the backup API takes `AsRef<Path>` and cannot.
//! - **Verify, then replace.** The snapshot is written under a staging name, opened, checked and only
//!   then renamed into place, because §14 forbids destroying the one good copy by writing a bad new
//!   one over it. `std::fs::rename` replaces on both platforms, so a repeat of the same transition
//!   leaves exactly one file.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::Database;
use crate::error::StorageError;

/// The name a `from` → `to` upgrade of `store` owes it: the store's own file stem, the transition,
/// and nothing else. No username, no project name, no artifact name — the stem is the product's
/// constant, so the name is the same on every machine that runs the same upgrade.
#[must_use]
pub fn pre_migration_backup_path(store: &Path, from: i64, to: i64) -> PathBuf {
    let stem = store
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "firmwaresight-store".to_owned());
    store.with_file_name(format!("{stem}.pre-migration-v{from}-to-v{to}.sqlite"))
}

/// Which snapshot, if any, a pending upgrade owes — the whole rule, in one pure function.
///
/// `None` store is an in-memory database: there is no file to protect and nothing to put beside it.
/// A current version of `0` is a fresh store with no user schema in it, so a "pre-migration backup"
/// of it would be an empty file with a misleading name. A store already at the target has nothing
/// pending, and §11 forbids a backup on every startup.
#[must_use]
pub(crate) fn backup_plan(store: Option<&Path>, current: i64, target: i64) -> Option<PathBuf> {
    let path = store?;
    if current == 0 || current >= target {
        return None;
    }
    Some(pre_migration_backup_path(path, current, target))
}

impl Database {
    /// Copy the live store to `destination` through the engine, verify the copy, and only then put it
    /// in place.
    ///
    /// # Errors
    ///
    /// `StorageError::Backup` naming what failed and which file it failed on — never a directory. The
    /// caller must not proceed with the migration on this path: the store stays at the schema and the
    /// rows it already had.
    pub(crate) fn snapshot_to(&self, destination: &Path, from: i64) -> Result<(), StorageError> {
        let name = destination
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "the pre-migration snapshot".to_owned());
        let staging = staging_next_to(destination);
        // A leftover staging file would make the engine refuse the destination it is handed, and the
        // refusal would look like a new failure rather than the debris of an older one.
        let _ = std::fs::remove_file(&staging);

        self.connection()
            .backup("main", &staging, None)
            .map_err(|_| StorageError::Backup {
                detail: format!("{name} could not be written"),
            })?;

        verify(&staging, from).map_err(|reason| {
            let _ = std::fs::remove_file(&staging);
            StorageError::Backup {
                detail: format!("{name} was written but {reason}, so the upgrade did not start"),
            }
        })?;

        std::fs::rename(&staging, destination).map_err(|_| {
            let _ = std::fs::remove_file(&staging);
            StorageError::Backup {
                detail: format!("{name} could not be put in place"),
            }
        })
    }
}

/// The copy is a database, it is well, and it carries the schema the live store is about to leave.
fn verify(snapshot: &Path, from: i64) -> Result<(), String> {
    let conn = Connection::open(snapshot).map_err(|_| "did not open as a database".to_owned())?;
    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .map_err(|_| "recorded no schema version".to_owned())?;
    if version != from {
        return Err(format!(
            "it records schema version {version} instead of {from}"
        ));
    }
    let answer: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|_| "could not be checked".to_owned())?;
    if !answer.eq_ignore_ascii_case("ok") {
        return Err("it is not a sound copy".to_owned());
    }
    drop(conn);
    Ok(())
}

fn staging_next_to(destination: &Path) -> PathBuf {
    let name = destination
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "snapshot".to_owned());
    destination.with_file_name(format!("{name}.staging-{:x}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::backup_plan;
    use std::path::Path;

    /// Which transitions owe a snapshot, in one pure function. Nothing-pending is §11's rule that a
    /// backup is not a startup chore; the in-memory case is the one the integration tests cannot reach,
    /// because an in-memory store has no path to assert on.
    #[test]
    fn only_a_file_backed_store_with_a_pending_upgrade_owes_a_snapshot() {
        let store = Path::new("/var/lib/example/firmwaresight-p0.sqlite");

        assert!(
            backup_plan(None, 4, 5).is_none(),
            "an in-memory store has no file to protect"
        );
        assert!(
            backup_plan(Some(store), 0, 5).is_none(),
            "a fresh store would be snapshot as an empty file with a misleading name"
        );
        assert!(
            backup_plan(Some(store), 5, 5).is_none(),
            "a store already at the target has nothing pending"
        );
        assert!(
            backup_plan(Some(store), 6, 5).is_none(),
            "a newer store is refused by the version check, not backed up by this path"
        );

        let owed = backup_plan(Some(store), 4, 5).expect("a 4 → 5 upgrade owes a snapshot");
        assert_eq!(
            owed.file_name().and_then(|name| name.to_str()),
            Some("firmwaresight-p0.pre-migration-v4-to-v5.sqlite"),
            "the name states the transition and adds nothing to the store's own file name"
        );
        assert_eq!(
            owed.parent(),
            store.parent(),
            "the copy sits beside the store"
        );
    }
}
