//! Deterministic serialization of core facts.
//!
//! Three separate things are kept separate here, as `04_TECH/26_PORTABLE_SCHEMA_POLICY.md` and
//! the P0 prompt both require:
//!
//! - the Core domain (`firmwaresight-core`) — the facts themselves;
//! - this crate's CLI/DTO projection — a bounded, stable-shaped view of those facts;
//! - the portable public schemas under `schemas/` — Release Bundle contracts.
//!
//! The shape emitted by `analyze` is marked `p0-internal`. It is deliberately *not* published as
//! `release-manifest v1` or any other versioned public contract, because P0 has not earned that
//! promise yet. The P2 diff export is a different case: it is declared, versioned and contract-
//! tested under `urn:firmwaresight:schema:diff:1` (`schemas/diff.schema.json`), and a Desktop IPC
//! DTO is never a public schema.

#![forbid(unsafe_code)]

pub mod diff;
pub mod diff_render;
pub mod dto;
pub mod render;

pub use diff::DiffResultDto;
pub use dto::AnalyzeResultDto;
pub use render::{to_json, to_json_pretty};

/// The identity of this output shape. Bumping the trailing number is a breaking change.
pub const ANALYZE_SCHEMA_ID: &str = "firmwaresight.analyze/p0-internal-1";

/// Explicit marker that this contract is internal to P0 and carries no compatibility promise.
pub const SCHEMA_STABILITY: &str = "p0-internal";
