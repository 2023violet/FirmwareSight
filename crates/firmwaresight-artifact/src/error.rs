//! Typed errors for artifact intake and parsing.
//!
//! Variant names follow the frozen taxonomy in `05_ENGINEERING/03_ERROR_MODEL.md`.
//! `ArtifactTooLarge` is required by `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md`, which mandates
//! a typed size-guard error before any full-buffer allocation.
//!
//! A missing optional input (no MAP, no symbol table, no Git) is deliberately *not* an error
//! here: those degrade capabilities, they do not fail the analysis.

use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("artifact not found: {path}")]
    InputNotFound { path: String },

    #[error("artifact could not be read: {path}")]
    InputUnreadable {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Raised from the stat result, before the file is read into memory.
    #[error("artifact is {byte_size} bytes, above the {limit_bytes} byte full-buffer limit")]
    ArtifactTooLarge {
        path: String,
        byte_size: u64,
        limit_bytes: u64,
    },

    #[error("unsupported format: {detected}")]
    UnsupportedFormat { detected: String },

    #[error("malformed artifact: {detail}")]
    MalformedArtifact { detail: String },

    #[error("artifact parsing failed: {detail}")]
    InvalidElf { detail: String },

    #[error("MAP format is not supported by any adapter: {detected}")]
    MapUnsupported { detected: String },

    #[error("MAP parsing failed at line {line}: {detail}")]
    MapParseFailed { line: usize, detail: String },

    /// Only for a violated internal invariant, never for user input.
    #[error("internal error: {detail}")]
    InternalBug { detail: String },
}

impl ArtifactError {
    /// Stable machine code for the `ErrorEnvelope` of `04_TECH/21_OBSERVABILITY_DIAGNOSTICS.md`.
    ///
    /// The `ERR-<AREA>-<NNNN>` shape itself is proposed by P0; no earlier baseline document
    /// defined a code registry, so this table is the registry and is asserted in tests.
    #[must_use]
    pub fn stable_code(&self) -> &'static str {
        match self {
            Self::InputNotFound { .. } => "ERR-INPUT-0001",
            Self::InputUnreadable { .. } => "ERR-INPUT-0002",
            Self::ArtifactTooLarge { .. } => "ERR-GUARD-0001",
            Self::UnsupportedFormat { .. } => "ERR-FORMAT-0001",
            Self::MalformedArtifact { .. } => "ERR-PARSE-2001",
            Self::InvalidElf { .. } => "ERR-PARSE-2002",
            Self::MapUnsupported { .. } => "ERR-MAP-3001",
            Self::MapParseFailed { .. } => "ERR-MAP-3002",
            Self::InternalBug { .. } => "ERR-INTERNAL-9001",
        }
    }

    /// What a user can actually do next, per the four-part message structure required by
    /// `05_ENGINEERING/03_ERROR_MODEL.md`.
    #[must_use]
    pub fn remediation(&self) -> &'static str {
        match self {
            Self::InputNotFound { .. } => {
                "Check the path, or select the artifact in a file dialog."
            }
            Self::InputUnreadable { .. } => "Close other programs holding the file, then retry.",
            Self::ArtifactTooLarge { .. } => {
                "Raise max_full_buffer_bytes in firmwaresight.toml only if this size is expected."
            }
            Self::UnsupportedFormat { .. } => "Provide an ELF linker output, or a BIN/HEX image.",
            Self::MalformedArtifact { .. } | Self::InvalidElf { .. } => {
                "Choose the linker ELF output rather than a stripped or truncated copy."
            }
            Self::MapUnsupported { .. } => {
                "Import the ELF without a MAP, or supply a GNU ld MAP; other toolchain MAPs are \
                 not supported yet."
            }
            Self::MapParseFailed { .. } => {
                "Regenerate the MAP with the same linker invocation; do not hand-edit it."
            }
            Self::InternalBug { .. } => "Report the operation id; no artifact data is needed.",
        }
    }

    /// Exit-code class per `04_TECH/07_CLI_SPEC.md`: parse/import problems are 3.
    #[must_use]
    pub fn is_parse_or_import(&self) -> bool {
        !matches!(self, Self::InternalBug { .. })
    }
}

/// Path shown to users and logged: never rewritten, but never widened.
#[must_use]
pub fn display_path(path: &Path) -> String {
    path.display().to_string()
}

/// Convenience for building an error from a path without leaking parent directories into
/// diagnostics more than the user supplied.
#[must_use]
pub fn not_found(path: &Path) -> ArtifactError {
    ArtifactError::InputNotFound {
        path: display_path(path),
    }
}

#[must_use]
pub fn unreadable(path: &Path, source: std::io::Error) -> ArtifactError {
    ArtifactError::InputUnreadable {
        path: display_path(&PathBuf::from(path)),
        source,
    }
}
