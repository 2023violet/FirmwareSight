//! Storage adapter: SQLite via `rusqlite` with the bundled engine.
//!
//! P0's job here is narrow but real: prove the storage boundary holds. That means an explicit
//! migration mechanism, connection settings the baseline requires, and one property that matters
//! more than the rest — a failed import must never be readable as a complete build.
//!
//! P5 added the two things that job always assumed someone else owned: the store can say whether it is
//! well (`health`), and an upgrade is preceded by a verified copy of what the file was (`backup`).
//! `counts` is the third piece of the same argument — a support surface needs bounded facts about the
//! file, and the only place SQL belongs is here.

#![forbid(unsafe_code)]

pub mod backup;
pub mod compare;
pub mod counts;
pub mod db;
pub mod error;
pub mod gate;
pub mod health;
pub mod history;
pub mod query;
pub mod release;

pub use db::{BuildSummary, Database, SCHEMA_VERSION};
pub use error::StorageError;
// A snapshot is taken where the store is opened, and the name a caller would look for is decided
// here rather than guessed at by whoever has to explain the recovery path.
pub use backup::pre_migration_backup_path;
// Health is a storage fact, not a UI idea: the engine answers, this crate bounds the answer.
pub use health::StoreHealth;
// The same argument in counts form: a Diagnostics payload is an allowlist of what this crate can read,
// so the reading stays here and the shell moves numbers across the boundary and nothing else.
pub use counts::{JournalMode, StoreCounts};
// The query rows are defined in terms of these two Core types, so a caller that reads a page has to
// be able to name them. Re-exported rather than made the caller take a second dependency: the
// storage boundary already speaks them in its public signatures.
pub use firmwaresight_core::domain::evidence::EvidenceClass;
// A stored attachment row is typed in these two Core names, so a caller that reads one run's rows has
// to be able to name them; the same reason `Fact` is re-exported above.
pub use firmwaresight_core::domain::gate::{GateAttachmentFact, KindBasis};
pub use firmwaresight_core::domain::identity::{ArtifactKind, Fact};
// Compare reads persisted facts only; the diff itself is decided in `firmwaresight-core`.
pub use compare::{
    CandidateQuery, CompareCandidate, DEFAULT_CANDIDATE_LIMIT, MAX_CANDIDATE_LIMIT, StoredBudget,
};
pub use gate::{
    AcceptReviewError, AcceptedReview, GateArtifactRow, GateEvidenceGaps, GateFootprintRow,
    GateRunDraft, GateRunWrite, GateSnapshotFacts, StoredGateAttachment, StoredGateFinding,
    StoredGateRun,
};
// History reads persisted facts only; it adds no table and decides no verdict.
pub use history::{
    DEFAULT_HISTORY_LIMIT, HistoryGateRun, HistoryQuery, HistoryReleaseRecord, MAX_HISTORY_LIMIT,
};
pub use query::{
    DEFAULT_QUERY_LIMIT, EvidenceQuery, EvidenceRow, EvidenceSort, MAX_QUERY_LIMIT, Page,
    SectionQuery, SectionRow, SectionSort, SortDir, SymbolQuery, SymbolRow, SymbolSort,
};
pub use release::{ReleaseRecordDraft, ReleaseRecordWrite, StoredReleaseRecord};
