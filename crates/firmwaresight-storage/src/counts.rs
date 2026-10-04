//! What the store holds, counted, and how it is journaling — the two questions a support surface asks
//! about a database before it asks anything else.
//!
//! Both are here rather than in the shell because SQL belongs to this crate (`AGENTS.md` 6, and
//! prompt §9's rule that the UI must not send SQL), and because a Diagnostics payload must be an
//! allowlist, not a dump: these are the five object classes prompt §6 names and the one `PRAGMA` that
//! says whether a `-wal` file beside the store is expected.
//!
//! The counts are deliberately not a `Result`. A support surface is most useful on a store that is
//! not well, and `SELECT COUNT(*)` on a damaged table can fail even while four other tables answer —
//! so each count is read on its own and a count that could not be read is reported as absent rather
//! than as zero. Reporting zero would say "this project has no builds", which is a different and
//! untruer claim than "this store would not answer for builds".

use crate::Database;

/// One table's row count: `Some` when SQLite answered, `None` when it would not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreCounts {
    pub projects: Option<i64>,
    pub builds: Option<i64>,
    pub gate_runs: Option<i64>,
    pub accepted_reviews: Option<i64>,
    pub release_records: Option<i64>,
}

/// How the store is journalling. `Wal` is what the connection settings ask for; anything else means
/// this filesystem could not do it, which is worth knowing when a support question mentions a
/// half-written import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalMode {
    Wal,
    Delete,
    Truncate,
    Persist,
    Memory,
    Off,
    /// The engine answered with a name this build does not know, or did not answer.
    Other,
}

impl JournalMode {
    /// The name SQLite uses for it, which is what a Diagnostics payload carries.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wal => "wal",
            Self::Delete => "delete",
            Self::Truncate => "truncate",
            Self::Persist => "persist",
            Self::Memory => "memory",
            Self::Off => "off",
            Self::Other => "other",
        }
    }
}

impl Database {
    /// Count the five object classes a support conversation needs, one table at a time.
    ///
    /// `builds` counts every build row whatever its state, including one left at `IMPORTING` by an
    /// interrupted import: that is the number of rows the store holds, and a count that silently
    /// filtered by state would understate the file a person is being asked about.
    #[must_use]
    pub fn counts(&self) -> StoreCounts {
        StoreCounts {
            projects: count(self, "projects"),
            builds: count(self, "builds"),
            gate_runs: count(self, "gate_runs"),
            accepted_reviews: count(self, "accepted_reviews"),
            release_records: count(self, "release_records"),
        }
    }

    /// The journal mode this connection is running under.
    #[must_use]
    pub fn journal_mode(&self) -> JournalMode {
        self.connection()
            .query_row("PRAGMA journal_mode", [], |row| row.get::<_, String>(0))
            .map(|mode| match mode.to_ascii_lowercase().as_str() {
                "wal" => JournalMode::Wal,
                "delete" => JournalMode::Delete,
                "truncate" => JournalMode::Truncate,
                "persist" => JournalMode::Persist,
                "memory" => JournalMode::Memory,
                "off" => JournalMode::Off,
                _ => JournalMode::Other,
            })
            .unwrap_or(JournalMode::Other)
    }

    /// The schema version the *store* carries, read from its own bookkeeping.
    ///
    /// This is not `SCHEMA_VERSION`, which is the version this binary was built to speak. The two are
    /// equal for any store this application opened, and a Diagnostics payload that reported the build
    /// constant as a fact about the file would be reporting the wrong thing on a store that had been
    /// replaced from a backup.
    #[must_use]
    pub fn schema_version(&self) -> Option<i64> {
        self.connection()
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get::<_, Option<i64>>(0)
            })
            .ok()
            .flatten()
    }

    /// The machine's clock as the engine reads it, in the same UTC shape every timestamp this product
    /// stores carries — `strftime`'s own format, because that is what wrote the rows beside it.
    ///
    /// Diagnostics labels this observation metadata only: nothing hashes it, and no identity claim
    /// reads it. It exists so a person who sends a support file can say when they captured it.
    #[must_use]
    pub fn observed_at(&self) -> Option<String> {
        self.connection()
            .query_row("SELECT strftime('%Y-%m-%dT%H:%M:%SZ','now')", [], |row| {
                row.get(0)
            })
            .ok()
    }
}

/// One `SELECT COUNT(*)`, or `None` if this store would not answer for this table.
///
/// `table` is `&'static str` and every call site below passes a literal, so no caller in the workspace
/// can aim this at a table the allowlist did not name.
fn count(db: &Database, table: &'static str) -> Option<i64> {
    db.connection()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get::<_, i64>(0)
        })
        .ok()
}
