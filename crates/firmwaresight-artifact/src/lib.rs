//! Artifact intake and parsing.
//!
//! `object` crate types stop here: nothing from it is exposed to core, the CLI DTO or the
//! desktop DTO (`04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md`).

#![forbid(unsafe_code)]

pub mod elf;
pub mod error;
pub mod intake;
pub mod map;
pub mod pipeline;

pub use error::ArtifactError;
pub use intake::{DetectedFormat, GuardConfig, GuardedInput};
pub use pipeline::{Analysis, AnalysisRequest, analyze};
