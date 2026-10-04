//! Typed storage errors.
//!
//! `StorageFailure` is the frozen taxonomy name from `05_ENGINEERING/03_ERROR_MODEL.md`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("could not open the database at {path}")]
    Open {
        path: String,
        #[source]
        source: rusqlite::Error,
    },

    #[error("database configuration failed: {detail}")]
    Configure { detail: String },

    #[error("migration {version} failed")]
    Migration {
        version: i64,
        #[source]
        source: rusqlite::Error,
    },

    #[error(
        "the database is at schema version {found}, which this build does not understand \
         (it supports up to {supported})"
    )]
    UnsupportedSchemaVersion { found: i64, supported: i64 },

    #[error("no build was found with id {id}")]
    NotFound { id: String },

    #[error("database write failed: {detail}")]
    Write { detail: String },

    /// The snapshot a pending upgrade owed the store could not be written or verified. `detail` names
    /// the snapshot by file name and never by directory, because this text crosses into a WebView.
    #[error("the snapshot before the upgrade failed: {detail}")]
    Backup { detail: String },

    /// The health question itself could not be asked. Damage the engine *can* name arrives as
    /// `StoreHealth::Unhealthy`, not as an error, so this variant means "we do not know".
    #[error("the database health check could not be completed: {detail}")]
    IntegrityCheck { detail: String },

    #[error("a constraint that should be structurally impossible was violated: {detail}")]
    Invariant { detail: String },
}

impl StorageError {
    /// Stable code for the diagnostic envelope, kept in the same registry as the artifact codes.
    #[must_use]
    pub fn stable_code(&self) -> &'static str {
        match self {
            Self::Open { .. } => "ERR-STORAGE-4001",
            Self::Configure { .. } => "ERR-STORAGE-4002",
            Self::Migration { .. } => "ERR-STORAGE-4003",
            Self::UnsupportedSchemaVersion { .. } => "ERR-STORAGE-4004",
            Self::NotFound { .. } => "ERR-STORAGE-4005",
            Self::Write { .. } => "ERR-STORAGE-4006",
            Self::Backup { .. } => "ERR-STORAGE-4011",
            Self::IntegrityCheck { .. } => "ERR-STORAGE-4012",
            Self::Invariant { .. } => "ERR-INTERNAL-9002",
        }
    }

    #[must_use]
    pub fn remediation(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion { .. } => {
                "Restore a backup and run the version of FirmwareSight that wrote this database. \
                 FirmwareSight will never silently rebuild a database it does not understand."
            }
            Self::Open { .. } => "Check that the project directory exists and is writable.",
            Self::NotFound { .. } => "Reload the project, then pick the build again.",
            // Named explicitly because the catch-all below says "retry the import", and an import is
            // not what this is: nothing was changed, and the store is the reason it was not.
            Self::Backup { .. } => {
                "FirmwareSight stopped before it changed the database, so the store still holds the \
                 schema and rows it had. Make the folder that holds it writable and with space to \
                 spare, then start the application again."
            }
            Self::IntegrityCheck { .. } => {
                "The store could not answer a health question. Leave the database file where it is, \
                 export Diagnostics if the screen allows it, and report the operation id; no \
                 firmware file is needed."
            }
            Self::Invariant { .. } => "Report the operation id; no artifact data is needed.",
            _ => "Retry the import. If it persists, inspect the operation id in the log.",
        }
    }
}
