//! Identity, provenance and the explicit-unknown value type.
//!
//! The type names here follow `04_TECH/02_DOMAIN_MODEL.md`, which names `Artifact` and
//! `BuildIdentity`. The P0 prompt's "ArtifactIdentity" is therefore implemented as the
//! identity-bearing fields of `Artifact` rather than as a new authority-bearing type name.

use crate::domain::evidence::EvidenceClass;

/// A domain value that distinguishes "absent" from "not determinable".
///
/// `Option::None` means the field does not exist in this shape. `Unknown` means the field
/// exists but current evidence cannot fill it, which is the case the P0 model requires to
/// survive serialization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Fact<T> {
    Known(T),
    Unknown { reason: String },
}

impl<T> Fact<T> {
    #[must_use]
    pub fn known(value: T) -> Self {
        Self::Known(value)
    }

    #[must_use]
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self::Unknown {
            reason: reason.into(),
        }
    }

    #[must_use]
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }

    #[must_use]
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Known(v) => Some(v),
            Self::Unknown { .. } => None,
        }
    }

    #[must_use]
    pub fn reason_if_unknown(&self) -> Option<&str> {
        match self {
            Self::Known(_) => None,
            Self::Unknown { reason } => Some(reason),
        }
    }

    /// Classification implied by whether the value is present.
    #[must_use]
    pub fn implied_class(&self) -> EvidenceClass {
        match self {
            Self::Known(_) => EvidenceClass::Observed,
            Self::Unknown { .. } => EvidenceClass::Unknown,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Fact<U> {
        match self {
            Self::Known(v) => Fact::Known(f(v)),
            Self::Unknown { reason } => Fact::Unknown { reason },
        }
    }
}

/// A fact that has not been established yet defaults to `Unknown`, never to a zero value.
impl<T> Default for Fact<T> {
    fn default() -> Self {
        Self::Unknown {
            reason: "not established".to_owned(),
        }
    }
}

/// Lowercase hex SHA-256 of artifact bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sha256(String);

impl Sha256 {
    pub const HEX_LEN: usize = 64;

    /// Rejects anything that is not exactly 64 lowercase hex characters.
    pub fn parse(input: &str) -> Result<Self, Sha256FormatError> {
        if input.len() != Self::HEX_LEN {
            return Err(Sha256FormatError::WrongLength(input.len()));
        }
        if !input
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Sha256FormatError::InvalidCharacter);
        }
        Ok(Self(input.to_ascii_lowercase()))
    }

    #[must_use]
    pub fn hex(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sha256FormatError {
    WrongLength(usize),
    InvalidCharacter,
}

/// What kind of input file this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    Elf,
    Map,
    Bin,
    IntelHex,
    Unknown,
}

/// Architecture, kept as domain vocabulary so `object::Architecture` never escapes the
/// artifact crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Architecture {
    Arm,
    Thumb,
    X86,
    X86_64,
    AArch64,
    RiscV32,
    RiscV64,
    Mips,
    Other(String),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bitness {
    Bits32,
    Bits64,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Endianness {
    Little,
    Big,
    Unknown,
}

/// Workspace Git context.
///
/// `04_TECH/24_BUILD_IDENTITY_EVIDENCE.md` is explicit that this describes the workspace the
/// analysis was pointed at, and is not proof that the artifact was built from this commit.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitContext {
    pub repo_root: Fact<String>,
    pub head_commit: Fact<String>,
    pub exact_tag: Fact<String>,
    pub dirty: Fact<bool>,
}

impl GitContext {
    #[must_use]
    pub fn unavailable(reason: &str) -> Self {
        Self {
            repo_root: Fact::unknown(reason),
            head_commit: Fact::unknown(reason),
            exact_tag: Fact::unknown("no Git repository was linked"),
            dirty: Fact::unknown(reason),
        }
    }
}

/// Build identity: version, toolchain and Git context, each with its own source.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BuildIdentity {
    pub project_version: Fact<String>,
    /// `Unknown` unless the artifact or a declaration supplies it. Never inferred from mtime.
    pub compiler: Fact<String>,
    pub linker: Fact<String>,
    pub target: Fact<String>,
    pub build_id: Fact<String>,
    pub git: GitContext,
    /// Fields the user declared, retained even where they conflict with observation.
    pub declared_fields: Vec<String>,
}

impl BuildIdentity {
    #[must_use]
    pub fn all_unknown(reason: &str) -> Self {
        Self {
            project_version: Fact::unknown(reason),
            compiler: Fact::unknown(reason),
            linker: Fact::unknown(reason),
            target: Fact::unknown(reason),
            build_id: Fact::unknown(reason),
            git: GitContext::unavailable(reason),
            declared_fields: Vec::new(),
        }
    }
}

/// The three distinct times that must not be conflated.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtifactTimes {
    /// Filesystem observation.
    pub artifact_mtime: Fact<String>,
    /// When FirmwareSight read it.
    pub imported_at: Fact<String>,
    /// Only from artifact or toolchain evidence, or a declaration.
    pub build_time: Fact<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_requires_lowercase_hex_of_exact_length() {
        let good = "a".repeat(64);
        assert!(Sha256::parse(&good).is_ok());

        assert!(Sha256::parse("abc").is_err());
        // Uppercase is not the stable representation.
        assert!(Sha256::parse(&"A".repeat(64)).is_err());
        assert!(Sha256::parse(&format!("{}z", "a".repeat(63))).is_err());
    }

    #[test]
    fn unknown_fact_keeps_its_reason_and_class() {
        let fact: Fact<u64> = Fact::unknown("no symbol table present");

        assert!(!fact.is_known());
        assert_eq!(fact.value(), None);
        assert_eq!(fact.reason_if_unknown(), Some("no symbol table present"));
        assert_eq!(fact.implied_class(), EvidenceClass::Unknown);
    }

    #[test]
    fn missing_git_leaves_every_field_unknown_rather_than_defaulting() {
        let git = GitContext::unavailable("git executable not found");

        assert!(!git.head_commit.is_known());
        assert!(!git.dirty.is_known());
        // Critically: not `false`. An unknown workspace must not read as clean.
        assert_eq!(git.dirty.value(), None);
    }

    #[test]
    fn build_identity_does_not_assume_a_version_from_anything() {
        let identity = BuildIdentity::default();

        assert!(!identity.project_version.is_known());
        assert!(identity.build_id.value().is_none());
    }
}
