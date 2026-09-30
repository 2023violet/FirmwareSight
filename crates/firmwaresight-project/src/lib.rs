//! FirmwareSight project boundary: `firmwaresight.toml`, workspace provenance and the deterministic
//! fingerprints a Gate run is identified by.
//!
//! ADR-0027 authorizes this crate and fixes its edges. It reads the outside world — a config file, a
//! release-notes file, a read-only `git` process — and hands `firmwaresight-core` plain facts. It never
//! decides what a fact means: no Gate rule, no state, no severity and no aggregate is computed here,
//! because the CLI and the desktop must get the same answer from the same input.
//!
//! ```text
//! firmwaresight.toml ─┐
//! project files      ─┤ firmwaresight-project      firmwaresight-core
//! system git (ro)    ─┘ facts + fingerprints  ──▶  rule evaluation  ──▶  GateRun
//! ```
//!
//! What this crate must not do, stated because each one has a real temptation behind it: depend on
//! Tauri, SQLite or Tokio; parse a firmware artifact; implement a Gate rule; build a UI; reach the
//! network; or surface the project root, which is a host path (`AGENTS.md` 3, 6, 7).

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

pub mod config;
pub mod error;
pub mod evidence;
pub mod fingerprint;
pub mod git;
pub mod version;

pub use config::{
    CONFIG_FILE_NAME, LoadedProject, ProjectConfig, REQUIRED_ARTIFACT_VOCABULARY,
    SUPPORTED_SCHEMA_VERSION, to_gate_policy,
};
pub use error::ProjectError;
pub use evidence::{
    FOOTPRINT_EVIDENCE_FIELDS, GateRunRequest, SnapshotFacts, build_context, footprint_evidence_id,
    observe_release_notes,
};
pub use fingerprint::{policy_sha256, run_id};
pub use git::{GitObservation, GitProbe};

#[cfg(test)]
mod fixtures;
