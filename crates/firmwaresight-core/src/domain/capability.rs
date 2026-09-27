//! Capability reporting: what the current inputs actually allow us to analyze.
//!
//! `04_TECH/03_FORMAT_SUPPORT.md` requires capability reporting as a condition of calling a
//! format supported, and `04_TECH/16_ARTIFACT_ANALYSIS_PIPELINE.md` requires that a missing
//! capability degrade the analysis instead of failing the whole snapshot.

use crate::domain::evidence::EvidenceClass;

/// Whether a format can be handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatSupport {
    Supported,
    Partial,
    Unsupported,
    Unknown,
}

/// Whether a piece of information is reachable from the current inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Availability {
    Available,
    Partial,
    Unavailable,
    Unknown,
}

/// Whether an optional companion input was supplied at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provision {
    Provided,
    NotProvided,
    Unknown,
}

/// The capability set reported with a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub elf: FormatSupport,
    pub sections: Availability,
    pub symbols: Availability,
    pub debug_info: Availability,
    pub map: Provision,
    /// Per-object/per-input-file attribution, which needs a MAP or DWARF.
    pub object_attribution: Availability,
    pub git: Availability,
}

impl Capabilities {
    /// ELF present, nothing else offered. This is the STATE A shape from the V0 narrative.
    #[must_use]
    pub fn elf_only() -> Self {
        Self {
            elf: FormatSupport::Supported,
            sections: Availability::Available,
            symbols: Availability::Unknown,
            debug_info: Availability::Unknown,
            map: Provision::NotProvided,
            object_attribution: Availability::Unavailable,
            git: Availability::Unknown,
        }
    }

    #[must_use]
    pub fn with_symbols(mut self, symbols: Availability) -> Self {
        self.symbols = symbols;
        self
    }

    #[must_use]
    pub fn with_debug_info(mut self, debug_info: Availability) -> Self {
        self.debug_info = debug_info;
        self
    }

    #[must_use]
    pub fn with_map(mut self, provision: Provision, attribution: Availability) -> Self {
        self.map = provision;
        self.object_attribution = attribution;
        self
    }

    #[must_use]
    pub fn with_git(mut self, git: Availability) -> Self {
        self.git = git;
        self
    }

    /// A degraded capability must never be reported as a parse failure, but it does cap how
    /// confidently any conclusion can be classified.
    #[must_use]
    pub fn best_supportable_evidence_class(&self) -> EvidenceClass {
        let degraded = self.symbols == Availability::Unavailable
            || self.object_attribution != Availability::Available
            || self.map == Provision::NotProvided;
        if degraded {
            EvidenceClass::Derived
        } else {
            EvidenceClass::Observed
        }
    }

    /// Rows in the stable order the UI renders.
    #[must_use]
    pub fn rows(&self) -> [(&'static str, String); 7] {
        [
            ("ELF", format!("{:?}", self.elf)),
            ("Sections", format!("{:?}", self.sections)),
            ("Symbols", format!("{:?}", self.symbols)),
            ("Debug info", format!("{:?}", self.debug_info)),
            ("MAP", format!("{:?}", self.map)),
            (
                "Object attribution",
                format!("{:?}", self.object_attribution),
            ),
            ("Git", format!("{:?}", self.git)),
        ]
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            elf: FormatSupport::Unknown,
            sections: Availability::Unknown,
            symbols: Availability::Unknown,
            debug_info: Availability::Unknown,
            map: Provision::Unknown,
            object_attribution: Availability::Unknown,
            git: Availability::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_map_degrades_attribution_without_failing_the_artifact() {
        let caps = Capabilities::elf_only();

        assert_eq!(caps.elf, FormatSupport::Supported);
        assert_eq!(caps.map, Provision::NotProvided);
        assert_eq!(caps.object_attribution, Availability::Unavailable);
    }

    #[test]
    fn absent_symbols_is_unknown_and_not_an_error() {
        // `04_TECH/03_FORMAT_SUPPORT.md`: an ELF without a symbol table is still a valid ELF.
        let caps = Capabilities::elf_only().with_symbols(Availability::Unavailable);

        assert_eq!(caps.elf, FormatSupport::Supported);
        assert_eq!(caps.symbols, Availability::Unavailable);
    }

    #[test]
    fn default_reports_unknown_rather_than_assuming_support() {
        let caps = Capabilities::default();

        assert_eq!(caps.elf, FormatSupport::Unknown);
        assert_eq!(caps.git, Availability::Unknown);
    }

    #[test]
    fn degraded_inputs_cap_evidence_at_derived() {
        let caps = Capabilities::default().with_git(Availability::Available);

        assert_eq!(
            caps.best_supportable_evidence_class(),
            EvidenceClass::Derived
        );
    }
}
