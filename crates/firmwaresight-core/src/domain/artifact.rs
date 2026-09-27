//! The `Artifact` entity of `04_TECH/02_DOMAIN_MODEL.md`.
//!
//! Identity is observed-only here: version strings and Git context belong to
//! `BuildIdentity`, because an artifact's own bytes are the only build evidence that cannot
//! drift away from it.

use crate::domain::evidence::EvidenceItem;
use crate::domain::identity::{
    Architecture, ArtifactKind, ArtifactTimes, Bitness, BuildIdentity, Endianness, Fact, Sha256,
};

/// The parser/adapter that produced this artifact's facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserId(pub String);

impl ParserId {
    #[must_use]
    pub fn elf_object(id: &str) -> Self {
        Self(format!("object-elf/{id}"))
    }

    #[must_use]
    pub fn gnu_ld_map() -> Self {
        Self("map-adapter/gnu_ld".to_owned())
    }
}

/// One analyzed input file and the facts observed from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub id: String,
    /// Project/fixture-relative where possible; never rewritten into an absolute host path
    /// inside a deterministic payload.
    pub path: String,
    pub kind: ArtifactKind,
    pub sha256: Sha256,
    pub byte_size: u64,
    pub parser_id: ParserId,
    pub architecture: Architecture,
    pub bitness: Bitness,
    pub endianness: Endianness,
    pub entry_point: Fact<u64>,
    pub build_id: Fact<String>,
    pub times: ArtifactTimes,
    pub identity: BuildIdentity,
    pub evidence: Vec<EvidenceItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_ids_are_namespaced_by_adapter_family() {
        assert_eq!(ParserId::elf_object("0.36").0, "object-elf/0.36");
        assert_eq!(ParserId::gnu_ld_map().0, "map-adapter/gnu_ld");
    }

    #[test]
    fn entry_point_can_be_unknown_without_invalidating_the_artifact() {
        let artifact = Artifact {
            id: "art-1".to_owned(),
            path: "fixtures/elf/p0-basic/firmware.elf".to_owned(),
            kind: ArtifactKind::Elf,
            sha256: Sha256::parse(&"b".repeat(64)).expect("64 lowercase hex chars"),
            byte_size: 260,
            parser_id: ParserId::elf_object("test"),
            architecture: Architecture::Thumb,
            bitness: Bitness::Bits32,
            endianness: Endianness::Little,
            entry_point: Fact::unknown("ET_REL objects have no entry point"),
            build_id: Fact::unknown("no .note.gnu.build-id section"),
            times: ArtifactTimes::default(),
            identity: BuildIdentity::default(),
            evidence: Vec::new(),
        };

        assert_eq!(artifact.kind, ArtifactKind::Elf);
        assert_eq!(artifact.entry_point.value(), None);
        assert!(artifact.entry_point.reason_if_unknown().is_some());
    }
}
