//! Storage adapter: SQLite via `rusqlite` with the bundled engine.
//!
//! P0's job here is narrow but real: prove the storage boundary holds. That means an explicit
//! migration mechanism, connection settings the baseline requires, and one property that matters
//! more than the rest — a failed import must never be readable as a complete build.

#![forbid(unsafe_code)]

pub mod compare;
pub mod db;
pub mod error;
pub mod query;

pub use db::{BuildSummary, Database, SCHEMA_VERSION};
pub use error::StorageError;
// The query rows are defined in terms of these two Core types, so a caller that reads a page has to
// be able to name them. Re-exported rather than made the caller take a second dependency: the
// storage boundary already speaks them in its public signatures.
pub use firmwaresight_core::domain::evidence::EvidenceClass;
pub use firmwaresight_core::domain::identity::Fact;
// Compare reads persisted facts only; the diff itself is decided in `firmwaresight-core`.
pub use compare::{
    CandidateQuery, CompareCandidate, DEFAULT_CANDIDATE_LIMIT, MAX_CANDIDATE_LIMIT, StoredBudget,
};
pub use query::{
    DEFAULT_QUERY_LIMIT, EvidenceQuery, EvidenceRow, EvidenceSort, MAX_QUERY_LIMIT, Page,
    SectionQuery, SectionRow, SectionSort, SortDir, SymbolQuery, SymbolRow, SymbolSort,
};
