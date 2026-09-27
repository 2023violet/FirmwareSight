//! Normalized symbol facts.
//!
//! Deliberately bounded: `04_TECH/13_FRONTEND_ARCHITECTURE.md` forbids copying a 100k symbol
//! table anywhere in one payload, so the domain carries per-symbol facts and the application
//! layer pages them.

use crate::domain::identity::Fact;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    NotType,
    Object,
    Function,
    Section,
    File,
    Common,
    Other(u8),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
    Other(u8),
    Unknown,
}

/// Linkage target: which section header index the symbol is relative to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolSectionRef {
    Undefined,
    Absolute,
    Index(usize),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: Fact<String>,
    pub address: Fact<u64>,
    pub size: Fact<u64>,
    pub kind: SymbolKind,
    pub binding: SymbolBinding,
    pub section: SymbolSectionRef,
}

impl Symbol {
    /// Names the UI is expected to be able to page over without a second parse.
    #[must_use]
    pub fn is_callable(&self) -> bool {
        self.kind == SymbolKind::Function
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_without_debug_info_still_carries_observable_fields() {
        let symbol = Symbol {
            name: Fact::known("mqtt_task".to_owned()),
            address: Fact::known(0x0800_0010),
            size: Fact::known(0x2c),
            kind: SymbolKind::Function,
            binding: SymbolBinding::Global,
            section: SymbolSectionRef::Index(1),
        };

        assert!(symbol.is_callable());
        assert_eq!(symbol.name.value(), Some(&"mqtt_task".to_owned()));
    }

    #[test]
    fn missing_size_is_unknown_and_not_zero() {
        let symbol = Symbol {
            name: Fact::known("g_zero".to_owned()),
            address: Fact::known(0x2000_0000),
            size: Fact::unknown("STT_OBJECT entry carried sh_size 0"),
            kind: SymbolKind::Object,
            binding: SymbolBinding::Global,
            section: SymbolSectionRef::Index(4),
        };

        assert_eq!(symbol.size.value(), None);
        assert!(symbol.size.reason_if_unknown().is_some());
    }
}
