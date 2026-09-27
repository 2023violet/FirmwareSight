//! The evidence model of `04_TECH/02_DOMAIN_MODEL.md` and `00_GOVERNANCE/02_GLOSSARY.md`.
//!
//! Every durable fact in FirmwareSight carries how it was obtained. `Unknown` is a value,
//! never an absent field, so "we could not determine this" survives serialization and cannot
//! be silently re-read as "there is nothing here".

use crate::domain::identity::Fact;

/// How a fact became known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvidenceClass {
    /// Read directly out of an input file.
    Observed,
    /// Produced from observed facts by a deterministic rule.
    Derived,
    /// Stated by a person or a build-time declaration.
    Declared,
    /// Current evidence cannot support a determination.
    Unknown,
}

impl EvidenceClass {
    /// Confidence ordering used when classes collide for the same field.
    ///
    /// Declared deliberately ranks below Observed: `04_TECH/02_DOMAIN_MODEL.md` requires that
    /// declared data never overwrites observed evidence, and both stay visible.
    #[must_use]
    pub const fn strength(self) -> u8 {
        match self {
            Self::Observed => 3,
            Self::Derived => 2,
            Self::Declared => 1,
            Self::Unknown => 0,
        }
    }
}

/// Where an evidence item came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceType {
    ElfFileHeader,
    ElfSectionHeader,
    ElfProgramHeader,
    ElfSymbolTable,
    ElfNote,
    MapFile,
    MemoryRegionConfig,
    UserDeclaration,
    EmbeddedVersion,
    GitCli,
    FileSystem,
    RuleEngine,
}

/// Confidence qualifier for `Derived` facts. A section-name heuristic is always `Low`
/// and may never be presented as `Observed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// One recorded fact and its provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceItem {
    pub id: String,
    pub classification: EvidenceClass,
    pub source_type: SourceType,
    /// Deterministic locator such as `elf.section_header[3].sh_addr`, so a reader can go
    /// back and re-check the claim.
    pub source_locator: String,
    pub field: String,
    pub raw_value: String,
    /// The parser or accounting rule that produced the value.
    pub rule: String,
    pub confidence: Option<Confidence>,
    pub notes: Option<String>,
}

impl EvidenceItem {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        classification: EvidenceClass,
        source_type: SourceType,
        source_locator: impl Into<String>,
        field: impl Into<String>,
        raw_value: impl Into<String>,
        rule: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            classification,
            source_type,
            source_locator: source_locator.into(),
            field: field.into(),
            raw_value: raw_value.into(),
            rule: rule.into(),
            confidence: None,
            notes: None,
        }
    }

    #[must_use]
    pub fn with_confidence(mut self, confidence: Confidence) -> Self {
        self.confidence = Some(confidence);
        self
    }

    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// A heuristic-derived item, flagged so it cannot be mistaken for an observation.
    #[must_use]
    pub fn heuristic(
        id: impl Into<String>,
        source_type: SourceType,
        source_locator: impl Into<String>,
        field: impl Into<String>,
        raw_value: impl Into<String>,
        rule: impl Into<String>,
    ) -> Self {
        Self::new(
            id,
            EvidenceClass::Derived,
            source_type,
            source_locator,
            field,
            raw_value,
            rule,
        )
        .with_confidence(Confidence::Low)
    }
}

/// Merge two claims about the same field.
///
/// The stronger classification wins, and the loser is reported instead of being dropped:
/// a conflict between what a person declared and what the binary shows is itself a fact.
#[must_use]
pub fn resolve_precedence(existing: &EvidenceItem, proposed: &EvidenceItem) -> MergeResolution {
    let order = proposed
        .classification
        .strength()
        .cmp(&existing.classification.strength());
    match order {
        std::cmp::Ordering::Greater => MergeResolution {
            winner: proposed.clone(),
            superseded: Some(existing.clone()),
        },
        std::cmp::Ordering::Less => MergeResolution {
            winner: existing.clone(),
            superseded: Some(proposed.clone()),
        },
        std::cmp::Ordering::Equal => MergeResolution {
            winner: existing.clone(),
            superseded: Some(proposed.clone()),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeResolution {
    pub winner: EvidenceItem,
    /// `Some` whenever a competing claim exists, including equal-strength conflicts,
    /// which the UI is allowed to surface as a conflict.
    pub superseded: Option<EvidenceItem>,
}

/// Turn an optional field into an explicit domain fact.
///
/// `None` means "this field does not exist"; it must not be used to mean "we do not know".
#[must_use]
pub fn to_fact<T>(value: Option<T>, reason: &str) -> Fact<T> {
    match value {
        Some(v) => Fact::Known(v),
        None => Fact::Unknown {
            reason: reason.to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(class: EvidenceClass, field: &str, value: &str) -> EvidenceItem {
        EvidenceItem::new(
            format!("ev-{field}"),
            class,
            SourceType::ElfSectionHeader,
            "elf.section_header[1]",
            field,
            value,
            "test-rule",
        )
    }

    #[test]
    fn declared_never_overwrites_observed() {
        let observed = item(EvidenceClass::Observed, "runtime_ram_bytes", "4");
        let declared = item(EvidenceClass::Declared, "runtime_ram_bytes", "9999");

        let resolution = resolve_precedence(&observed, &declared);

        assert_eq!(resolution.winner.raw_value, "4");
        assert_eq!(resolution.winner.classification, EvidenceClass::Observed);
        // The rejected declaration stays visible rather than vanishing.
        assert_eq!(
            resolution.superseded.map(|s| s.raw_value),
            Some("9999".into())
        );
    }

    #[test]
    fn equal_strength_conflict_keeps_both_visible() {
        let a = item(EvidenceClass::Observed, "entry", "0x1000");
        let b = item(EvidenceClass::Observed, "entry", "0x2000");

        let resolution = resolve_precedence(&a, &b);

        assert_eq!(resolution.winner.raw_value, "0x1000");
        assert!(resolution.superseded.is_some());
    }

    #[test]
    fn heuristic_can_never_claim_observed() {
        let heuristic = EvidenceItem::heuristic(
            "ev-h",
            SourceType::ElfSectionHeader,
            "elf.section_header[2].sh_name",
            "budget",
            "nonvolatile",
            "section-name-heuristic",
        );

        assert_eq!(heuristic.classification, EvidenceClass::Derived);
        assert_eq!(heuristic.confidence, Some(Confidence::Low));
        assert_ne!(heuristic.classification, EvidenceClass::Observed);
    }

    #[test]
    fn unknown_is_the_weakest_class_so_it_cannot_mask_a_fact() {
        assert!(EvidenceClass::Unknown.strength() < EvidenceClass::Observed.strength());
    }

    #[test]
    fn absent_value_becomes_explicit_unknown_with_a_reason() {
        let fact = to_fact::<u64>(None, "no PT_LOAD segment covered this section");
        assert!(matches!(fact, Fact::Unknown { reason } if reason.contains("PT_LOAD")));
    }
}
