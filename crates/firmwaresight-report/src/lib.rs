//! Deterministic serialization of core facts.
//!
//! Three separate things are kept separate here, as `04_TECH/26_PORTABLE_SCHEMA_POLICY.md` and
//! the P0 prompt both require:
//!
//! - the Core domain (`firmwaresight-core`) — the facts themselves;
//! - this crate's CLI/DTO projection — a bounded, stable-shaped view of those facts;
//! - the portable public schemas under `schemas/` — Release Bundle contracts.
//!
//! The five portable contracts are compiled into this crate by [`schemas`], so the build that writes a
//! bundle can verify one anywhere: prompt §44 asks the verifier to prove each document is valid against its
//! schema, and §45 asks it to do that on a machine holding nothing but the bundle. The authored copies stay
//! in `schemas/`, and a test compares the two so they cannot diverge.
//!
//! The shape emitted by `analyze` is marked `p0-internal`. It is deliberately *not* published as
//! `release-manifest v1` or any other versioned public contract, because P0 has not earned that
//! promise yet. The P2 diff export is a different case: it is declared, versioned and contract-
//! tested under `urn:firmwaresight:schema:diff:1` (`schemas/diff.schema.json`), and a Desktop IPC
//! DTO is never a public schema.

#![forbid(unsafe_code)]

pub mod analysis;
pub mod diff;
pub mod diff_render;
pub mod dto;
pub mod gate;
pub mod gate_render;
pub mod release;
pub mod release_render;
pub mod render;
pub mod schema_check;
pub mod schemas;

pub use analysis::{
    ANALYSIS_SCHEMA_ID, ANALYSIS_SCHEMA_VERSION, AnalysisDocumentDto, PortableArtifactDto,
    SectionRowDto, SnapshotDto, SymbolRowDto,
};
pub use diff::DiffResultDto;
pub use dto::AnalyzeResultDto;
pub use gate::{
    ACCEPTANCE_ORIGINAL_STATE, ACCEPTED_REVIEWS_SCHEMA_ID, ACCEPTED_REVIEWS_SCHEMA_VERSION,
    AcceptanceDto, AcceptedReviewsDto, GATE_SCHEMA_ID, GATE_SCHEMA_VERSION, GateFindingDto,
    GateResultsDto,
};
pub use release::{
    BuildFactsDto, GeneratedByDto, IntegrityModelDto, ManifestError, ManifestFileDto,
    RELEASE_MANIFEST_SCHEMA_ID, RELEASE_MANIFEST_SCHEMA_VERSION, ReleaseManifestDto,
};
pub use render::{to_json, to_json_pretty};

/// The identity of this output shape. Bumping the trailing number is a breaking change.
pub const ANALYZE_SCHEMA_ID: &str = "firmwaresight.analyze/p0-internal-1";

/// Explicit marker that this contract is internal to P0 and carries no compatibility promise.
pub const SCHEMA_STABILITY: &str = "p0-internal";
