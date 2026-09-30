//! Typed errors for project policy and provenance.
//!
//! Categories follow `05_ENGINEERING/03_ERROR_MODEL.md`: `ConfigInvalid`, `GitUnavailable` and
//! `PolicyInvalid` are named there. Codes continue the families already in use — `ERR-CONFIG-700n`
//! for configuration and `ERR-GIT-800n` for the workspace probe — and each message states what
//! happened and what the release owner can do next, because a config error is read by a human.

use std::path::PathBuf;

/// Every way this crate refuses to continue, and why.
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    /// The config file could not be opened at all.
    #[error("could not read the project configuration at {path}: {source}")]
    ConfigUnreadable {
        path: PathBuf,
        source: std::io::Error,
    },

    /// The bytes are not TOML, or not the shape TOML allows.
    #[error("`{path}` is not valid TOML: {message}")]
    ConfigMalformed { path: PathBuf, message: String },

    /// `schema_version` is missing or from a major this build cannot read. A config from the future
    /// is never interpreted by the rules of the past.
    #[error(
        "unsupported `schema_version` {found}; this FirmwareSight build reads version {expected} only"
    )]
    UnsupportedSchemaVersion { found: i64, expected: i64 },

    /// A value has the wrong type, an unknown enum word, or an out-of-range number.
    #[error("invalid configuration value for `{field}`: {detail}")]
    ConfigValueInvalid { field: String, detail: String },

    /// A configured path is absolute or escapes the project root, so honoring it would read outside
    /// the project the release owner pointed FirmwareSight at.
    #[error("configuration path `{path}` is not project-relative: {detail}")]
    UnsafeConfigPath { path: String, detail: String },

    /// `[version] pattern` does not compile, or has no `version` capture where one is required.
    #[error("`[version] pattern` cannot be used: {detail}")]
    InvalidVersionPattern { detail: String },

    /// A save would drop keys this build does not understand. The config is refused, not rewritten.
    #[error("refusing to save {path}: it holds keys this build does not understand ({keys})")]
    SaveWouldDropKeys { path: PathBuf, keys: String },

    /// The file could not be written, or the temp-and-replace swap failed.
    #[error("could not save the project configuration to {path}: {source}")]
    SaveFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// The workspace probe could not be completed. Facts degrade to Unknown; this variant is for the
    /// caller that asked for Git directly and needs to know why nothing came back.
    #[error("could not read Git workspace facts: {detail}")]
    GitUnavailable { detail: String },

    /// A structural expectation of this crate was violated, which means a bug rather than user input.
    #[error("internal invariant violated: {detail}")]
    InternalInvariant { detail: String },
}

impl ProjectError {
    /// The stable diagnostics code a UI can show and a report can quote.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::ConfigUnreadable { .. } => "ERR-CONFIG-7001",
            Self::ConfigMalformed { .. } => "ERR-CONFIG-7002",
            Self::UnsupportedSchemaVersion { .. } => "ERR-CONFIG-7003",
            Self::ConfigValueInvalid { .. } => "ERR-CONFIG-7004",
            Self::UnsafeConfigPath { .. } => "ERR-CONFIG-7005",
            Self::InvalidVersionPattern { .. } => "ERR-CONFIG-7006",
            Self::SaveWouldDropKeys { .. } => "ERR-CONFIG-7007",
            Self::SaveFailed { .. } => "ERR-CONFIG-7008",
            Self::GitUnavailable { .. } => "ERR-GIT-8001",
            Self::InternalInvariant { .. } => "ERR-INTERNAL-9003",
        }
    }
}
