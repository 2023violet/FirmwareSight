//! Persisting the record that a Release Bundle was published.
//!
//! This row is an audit index, never the bundle (prompt §67). It says *which* release id was issued, over
//! *which* stored build, judged by *which* Gate run, carrying *which* project version, with *which*
//! manifest digest — and nothing about where the directory went. There is no destination column, no source
//! path and no blob, so no caller of this API can receive a host path and a moved bundle stays valid
//! (`AGENTS.md` 7, prompt §37).
//!
//! Storage keeps no release semantics. It writes what the bundle builder computed, refuses a second record
//! whose id matches but whose content does not, and hands back what it stored. The shape rules — a release
//! id is `release-` plus a SHA-256, a manifest digest is lowercase hex, a release is never its own
//! baseline, a version is not a path — are CHECK constraints and triggers in migration `0004`, so they
//! hold for a hand-written `INSERT` as well.
//!
//! Three rules from prompt §38 and §54 govern this module:
//!
//! - a release record is written only *after* the bundle was staged, verified and published, which is the
//!   caller's ordering and is why nothing here reads a filesystem;
//! - the same release id with the same semantics dedupes to [`ReleaseRecordWrite::AlreadyStored`] instead
//!   of writing a second row;
//! - the same release id with different semantics is an invariant violation, never an update — a record
//!   that could be edited would let a published release be quietly re-pointed at other bytes.

use rusqlite::{OptionalExtension, params};

use crate::Database;
use crate::db::write_err;
use crate::error::StorageError;
use firmwaresight_core::domain::identity::Sha256;
use firmwaresight_core::domain::release::{ReleaseId, ReleaseVersion};

/// How a release record arrived: written now, or already stored with exactly these facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseRecordWrite {
    Inserted,
    AlreadyStored,
}

/// A release as the caller means to record it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseRecordDraft<'a> {
    /// `release-<sha256>`, the digest of the canonical release input the bundle was built from.
    pub release_id: &'a str,
    /// The persisted build that ships. A record is always tied to a stored build, because that is what
    /// makes it reviewable after the source files are gone.
    pub build_id: &'a str,
    pub baseline_build_id: Option<&'a str>,
    /// The Gate run whose `PASS` disposition qualified this release (§9).
    pub gate_run_id: &'a str,
    pub release_version: &'a str,
    /// SHA-256 of `release-manifest.json` as written, which is the fact that ties this row to a directory
    /// without naming one.
    pub manifest_sha256: &'a str,
}

/// One published release, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredReleaseRecord {
    pub release_id: String,
    pub build_id: String,
    pub baseline_build_id: Option<String>,
    pub gate_run_id: String,
    pub release_version: String,
    pub manifest_sha256: String,
    /// `release_records.created_at`, written by SQLite at insertion. An audit time, and deliberately not
    /// part of the release id (prompt §42).
    pub created_at: String,
}

impl Database {
    /// Record one published release, or recognize that it is already recorded.
    ///
    /// # Errors
    ///
    /// [`StorageError::Write`] when a foreign key the record names is not persisted (an unknown build or
    /// Gate run), or when a constraint the caller's data violated fires; [`StorageError::Invariant`] when
    /// the id, digest or version is not shaped the way a release record requires, or when the same release
    /// id is already stored with different facts — which means two different bundles claim one identity,
    /// and no write happens either way.
    pub fn persist_release_record(
        &mut self,
        draft: &ReleaseRecordDraft<'_>,
    ) -> Result<ReleaseRecordWrite, StorageError> {
        // Checked here as well as in the schema so the failure names the field rather than a constraint
        // index, and so a caller that bypassed this crate still cannot write the row.
        ReleaseId::parse(draft.release_id).map_err(|err| StorageError::Invariant {
            detail: err.to_string(),
        })?;
        Sha256::parse(draft.manifest_sha256).map_err(|err| StorageError::Invariant {
            detail: format!("manifest digest of release `{}`: {err:?}", draft.release_id),
        })?;
        ReleaseVersion::parse(draft.release_version).map_err(|err| StorageError::Invariant {
            detail: format!("release version of `{}`: {err}", draft.release_id),
        })?;

        if let Some(existing) = self.release_record_by_id(draft.release_id)? {
            return if existing.matches(draft) {
                Ok(ReleaseRecordWrite::AlreadyStored)
            } else {
                Err(StorageError::Invariant {
                    detail: format!(
                        "release `{}` is already recorded with different facts; a changed build, Gate run, \
                         version or manifest produces a new release id rather than rewriting that record",
                        draft.release_id
                    ),
                })
            };
        }

        let tx = self.conn.transaction().map_err(write_err)?;
        tx.execute(
            "INSERT INTO release_records
                 (id, build_id, baseline_build_id, gate_run_id, release_version, manifest_sha256)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                draft.release_id,
                draft.build_id,
                draft.baseline_build_id,
                draft.gate_run_id,
                draft.release_version,
                draft.manifest_sha256,
            ],
        )
        .map_err(|source| describe_write(source, draft.release_id))?;
        tx.commit()
            .map_err(|source| describe_write(source, draft.release_id))?;

        Ok(ReleaseRecordWrite::Inserted)
    }

    /// The record for one release id, if this database has one.
    pub fn release_record_by_id(
        &self,
        release_id: &str,
    ) -> Result<Option<StoredReleaseRecord>, StorageError> {
        let found = self
            .conn
            .query_row(
                "SELECT id, build_id, baseline_build_id, gate_run_id, release_version,
                        manifest_sha256, created_at
                   FROM release_records WHERE id = ?1",
                params![release_id],
                |row| {
                    Ok(StoredReleaseRecord {
                        release_id: row.get(0)?,
                        build_id: row.get(1)?,
                        baseline_build_id: row.get(2)?,
                        gate_run_id: row.get(3)?,
                        release_version: row.get(4)?,
                        manifest_sha256: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(write_err)?;
        Ok(found)
    }
}

impl StoredReleaseRecord {
    /// Whether an existing row says exactly what this draft says. Compared field by field, never by
    /// `created_at`: the audit time is SQLite's, and two writes of one release differ in it by design.
    #[must_use]
    fn matches(&self, draft: &ReleaseRecordDraft<'_>) -> bool {
        self.build_id == draft.build_id
            && self.baseline_build_id.as_deref() == draft.baseline_build_id
            && self.gate_run_id == draft.gate_run_id
            && self.release_version == draft.release_version
            && self.manifest_sha256 == draft.manifest_sha256
    }
}

/// A constraint failure, with the release it was about. A bare SQLite message reports which constraint
/// fired; a release owner needs to know which release it fired on.
fn describe_write(source: rusqlite::Error, release_id: &str) -> StorageError {
    StorageError::Write {
        detail: format!("{release_id}: {source}"),
    }
}
