//! Storage adapter: SQLite via `rusqlite` with the bundled engine.
//!
//! P0's job here is narrow but real: prove the storage boundary holds. That means an explicit
//! migration mechanism, connection settings the baseline requires, and one property that matters
//! more than the rest — a failed import must never be readable as a complete build.

#![forbid(unsafe_code)]

pub mod db;
pub mod error;

pub use db::{BuildSummary, Database, SCHEMA_VERSION};
pub use error::StorageError;
