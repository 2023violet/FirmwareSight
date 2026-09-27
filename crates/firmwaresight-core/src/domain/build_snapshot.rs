//! `BuildSnapshot`: the immutable, normalized record everything downstream reads.
//!
//! `04_TECH/02_DOMAIN_MODEL.md` requires that a snapshot is immutable after creation and that
//! diff and gate only ever read it. That is enforced structurally here: the only way to obtain
//! a `BuildSnapshot` is to `seal()` a `SnapshotBuilder`, and `BuildSnapshot` exposes no
//! mutator.

use crate::domain::artifact::Artifact;
use crate::domain::capability::Capabilities;
use crate::domain::evidence::EvidenceItem;
use crate::domain::identity::{BuildIdentity, Sha256};
use crate::domain::memory::MemoryFootprint;
use crate::domain::section::Section;
use crate::domain::symbol::Symbol;
use crate::error::CoreError;

/// Version of the normalization rules that produced a snapshot.
///
/// Bumping this is what makes an old snapshot distinguishable from a re-analyzed one, so a
/// stable id never silently means "same bytes".
pub const NORMALIZATION_VERSION: &str = "p0-normalize-1";

/// Content-addressed snapshot identifier.
///
/// The baseline domain model requires stable ids but does not define the derivation, so this
/// is the P0 decision: concatenate the already-cryptographic artifact identities plus the
/// normalization version. It is stable across runs, changes when any input byte changes, and
/// avoids introducing a second hash implementation into a dependency-free core crate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SnapshotId(pub String);

impl SnapshotId {
    #[must_use]
    pub fn compose(artifact_sha256: &Sha256, map_sha256: Option<&Sha256>) -> Self {
        let mut id = format!("snap-{}-{NORMALIZATION_VERSION}", artifact_sha256.hex());
        if let Some(map) = map_sha256 {
            id.push('-');
            id.push_str(map.hex());
        }
        Self(id)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Accumulating half of the snapshot API.
#[derive(Debug, Clone, Default)]
pub struct SnapshotBuilder {
    project_id: Option<String>,
    artifacts: Vec<Artifact>,
    identity: BuildIdentity,
    memory: Option<MemoryFootprint>,
    sections: Vec<Section>,
    symbols: Vec<Symbol>,
    evidence: Vec<EvidenceItem>,
    capabilities: Capabilities,
    created_by_fwsight_version: String,
}

impl SnapshotBuilder {
    #[must_use]
    pub fn new(fwsight_version: impl Into<String>) -> Self {
        Self {
            capabilities: Capabilities::default(),
            created_by_fwsight_version: fwsight_version.into(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn project_id(mut self, id: impl Into<String>) -> Self {
        self.project_id = Some(id.into());
        self
    }

    #[must_use]
    pub fn artifact(mut self, artifact: Artifact) -> Self {
        self.artifacts.push(artifact);
        self
    }

    #[must_use]
    pub fn identity(mut self, identity: BuildIdentity) -> Self {
        self.identity = identity;
        self
    }

    #[must_use]
    pub fn memory(mut self, memory: MemoryFootprint) -> Self {
        self.memory = Some(memory);
        self
    }

    #[must_use]
    pub fn sections(mut self, sections: Vec<Section>) -> Self {
        self.sections = sections;
        self
    }

    #[must_use]
    pub fn symbols(mut self, symbols: Vec<Symbol>) -> Self {
        self.symbols = symbols;
        self
    }

    #[must_use]
    pub fn evidence(mut self, evidence: Vec<EvidenceItem>) -> Self {
        self.evidence = evidence;
        self
    }

    #[must_use]
    pub fn capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Freeze the facts. A snapshot with no artifact is not a description of anything, so it
    /// is rejected rather than emitted empty.
    pub fn seal(self) -> Result<BuildSnapshot, CoreError> {
        let Some(primary) = self.artifacts.first() else {
            return Err(CoreError::InternalInvariant {
                detail: "snapshot sealed with no artifact".to_owned(),
            });
        };

        let map_sha = self
            .artifacts
            .iter()
            .find(|a| a.kind == crate::domain::identity::ArtifactKind::Map)
            .map(|a| &a.sha256);

        let id = SnapshotId::compose(&primary.sha256, map_sha);

        Ok(BuildSnapshot {
            id,
            project_id: self.project_id.unwrap_or_else(|| "unnamed".to_owned()),
            artifacts: self.artifacts,
            identity: self.identity,
            memory: self.memory,
            sections: self.sections,
            symbols: self.symbols,
            evidence: self.evidence,
            capabilities: self.capabilities,
            created_by_fwsight_version: self.created_by_fwsight_version,
        })
    }
}

/// A sealed, read-only snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSnapshot {
    id: SnapshotId,
    project_id: String,
    artifacts: Vec<Artifact>,
    identity: BuildIdentity,
    memory: Option<MemoryFootprint>,
    sections: Vec<Section>,
    symbols: Vec<Symbol>,
    evidence: Vec<EvidenceItem>,
    capabilities: Capabilities,
    created_by_fwsight_version: String,
}

impl BuildSnapshot {
    #[must_use]
    pub fn id(&self) -> &SnapshotId {
        &self.id
    }

    #[must_use]
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    #[must_use]
    pub fn artifacts(&self) -> &[Artifact] {
        &self.artifacts
    }

    /// The ELF (or other primary) artifact the snapshot was addressed by.
    #[must_use]
    pub fn primary_artifact(&self) -> Option<&Artifact> {
        self.artifacts.first()
    }

    #[must_use]
    pub fn identity(&self) -> &BuildIdentity {
        &self.identity
    }

    #[must_use]
    pub fn memory(&self) -> Option<&MemoryFootprint> {
        self.memory.as_ref()
    }

    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    #[must_use]
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    #[must_use]
    pub fn evidence(&self) -> &[EvidenceItem] {
        &self.evidence
    }

    #[must_use]
    pub fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    #[must_use]
    pub fn created_by_fwsight_version(&self) -> &str {
        &self.created_by_fwsight_version
    }

    #[must_use]
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    #[must_use]
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::artifact::ParserId;
    use crate::domain::identity::Architecture;
    use crate::domain::identity::ArtifactKind;
    use crate::domain::identity::{ArtifactTimes, Bitness, Endianness, Fact};

    fn artifact(hex_digit: char) -> Artifact {
        Artifact {
            id: "art-1".to_owned(),
            path: "fixtures/elf/p0-basic/firmware.elf".to_owned(),
            kind: ArtifactKind::Elf,
            sha256: Sha256::parse(&hex_digit.to_string().repeat(64)).expect("hex"),
            byte_size: 260,
            parser_id: ParserId::elf_object("test"),
            architecture: Architecture::Thumb,
            bitness: Bitness::Bits32,
            endianness: Endianness::Little,
            entry_point: Fact::known(0x0800_0039),
            build_id: Fact::unknown("absent"),
            times: ArtifactTimes::default(),
            identity: BuildIdentity::default(),
            evidence: Vec::new(),
        }
    }

    #[test]
    fn snapshot_id_is_stable_for_the_same_input_bytes() {
        let a = SnapshotId::compose(&Sha256::parse(&"a".repeat(64)).expect("hex"), None);
        let b = SnapshotId::compose(&Sha256::parse(&"a".repeat(64)).expect("hex"), None);

        assert_eq!(a, b);
    }

    #[test]
    fn snapshot_id_changes_when_any_input_changes() {
        let without_map = SnapshotId::compose(&Sha256::parse(&"a".repeat(64)).expect("hex"), None);
        let with_map = SnapshotId::compose(
            &Sha256::parse(&"a".repeat(64)).expect("hex"),
            Some(&Sha256::parse(&"f".repeat(64)).expect("hex")),
        );

        assert_ne!(without_map, with_map);
        assert!(without_map.as_str().contains(NORMALIZATION_VERSION));
    }

    #[test]
    fn sealing_without_an_artifact_is_rejected_not_emitted_empty() {
        let result = SnapshotBuilder::new("0.1.0").seal();

        assert!(matches!(result, Err(CoreError::InternalInvariant { .. })));
    }

    #[test]
    fn sealed_snapshot_exposes_no_mutator() {
        // Structural immutability: these are the only associated functions available, and
        // none of them take &mut self.
        let snapshot = SnapshotBuilder::new("0.1.0")
            .artifact(artifact('a'))
            .seal()
            .expect("one artifact");

        assert_eq!(snapshot.artifacts().len(), 1);
        assert_eq!(snapshot.primary_artifact().map(|a| a.byte_size), Some(260));
    }
}
