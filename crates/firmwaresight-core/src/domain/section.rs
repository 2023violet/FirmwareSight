//! Normalized section facts.
//!
//! Names follow `04_TECH/02_DOMAIN_MODEL.md` (`sections` is a first-class part of a
//! `BuildSnapshot`). `virtual_address` and `load_address` carry the ELF `sh_addr` / `p_paddr`
//! distinction that makes `.data` dual-accounting provable rather than assumed.

use crate::domain::identity::Fact;

/// Coarse section role, decided from flags and evidence rather than from the name alone.
///
/// An unrecognized role is `Unknown` and contributes to neither budget until evidence says
/// otherwise. The raw name stays on `Section::name`, so the role enum carries no payload and
/// stays `Copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionRole {
    Code,
    ReadOnlyData,
    InitializedData,
    UninitializedData,
    Debug,
    Note,
    SymbolTable,
    StringTable,
    ArmAttribute,
    /// A recognized section that is neither code, data nor host metadata.
    Other,
    /// Role could not be decided from available evidence.
    Unknown,
}

impl SectionRole {
    /// Default treatment in the two memory budgets.
    ///
    /// This is the rule table of `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md`, and is only
    /// *applied* to a section once the accounting basis has been resolved.
    #[must_use]
    pub const fn contributes_nonvolatile_by_default(self) -> bool {
        match self {
            Self::Code | Self::ReadOnlyData | Self::InitializedData => true,
            Self::UninitializedData
            | Self::Debug
            | Self::Note
            | Self::SymbolTable
            | Self::StringTable
            | Self::ArmAttribute
            | Self::Other
            | Self::Unknown => false,
        }
    }

    #[must_use]
    pub const fn contributes_runtime_ram_by_default(self) -> bool {
        match self {
            Self::InitializedData | Self::UninitializedData => true,
            Self::Code
            | Self::ReadOnlyData
            | Self::Debug
            | Self::Note
            | Self::SymbolTable
            | Self::StringTable
            | Self::ArmAttribute
            | Self::Other
            | Self::Unknown => false,
        }
    }

    /// Host/toolchain metadata that is not part of the device budget.
    #[must_use]
    pub const fn is_device_metadata(self) -> bool {
        matches!(
            self,
            Self::Debug | Self::SymbolTable | Self::StringTable | Self::Note | Self::ArmAttribute
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SectionFlags {
    pub alloc: bool,
    pub execute: bool,
    pub write: bool,
    pub merge: bool,
    pub strings: bool,
    pub tls: bool,
    pub compressed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// Section header index, the stable locator inside one ELF file.
    pub index: usize,
    pub name: Fact<String>,
    pub role: SectionRole,
    pub flags: SectionFlags,
    /// Runtime address (`sh_addr`).
    pub virtual_address: Fact<u64>,
    /// Address the bytes are loaded from (`p_paddr` of the covering PT_LOAD), which for
    /// embedded images usually sits in ROM.
    pub load_address: Fact<u64>,
    pub file_offset: Fact<u64>,
    /// Bytes occupied in the file. Zero for `SHT_NOBITS`.
    pub file_size: u64,
    /// Bytes occupied at runtime.
    pub memory_size: Fact<u64>,
    /// Named memory region from a linker/MAP source, when one exists.
    pub region: Fact<String>,
}

impl Section {
    /// Whether the file actually stores payload for this section.
    #[must_use]
    pub fn has_file_payload(&self) -> bool {
        self.file_size > 0
    }

    /// A `NOBITS` section is the classic `.bss` shape: it costs RAM but no image bytes.
    #[must_use]
    pub fn is_nobits_shaped(&self) -> bool {
        self.file_size == 0 && self.memory_size.value().copied().unwrap_or(0) > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(role: SectionRole, file_size: u64, mem_size: Fact<u64>) -> Section {
        Section {
            index: 1,
            name: Fact::unknown("stripped"),
            role,
            flags: SectionFlags::default(),
            virtual_address: Fact::unknown("test"),
            load_address: Fact::unknown("test"),
            file_offset: Fact::unknown("test"),
            file_size,
            memory_size: mem_size,
            region: Fact::unknown("test"),
        }
    }

    #[test]
    fn bss_shaped_section_costs_ram_but_no_image_bytes() {
        let bss = section(SectionRole::UninitializedData, 0, Fact::known(4096));

        assert!(bss.is_nobits_shaped());
        assert!(!bss.has_file_payload());
        assert!(!bss.role.contributes_nonvolatile_by_default());
        assert!(bss.role.contributes_runtime_ram_by_default());
    }

    #[test]
    fn debug_sections_are_device_metadata() {
        assert!(SectionRole::Debug.is_device_metadata());
        assert!(!SectionRole::Debug.contributes_nonvolatile_by_default());
        assert!(!SectionRole::Debug.contributes_runtime_ram_by_default());
    }

    #[test]
    fn unknown_role_is_never_treated_as_allocatable() {
        let role = SectionRole::Unknown;

        assert!(!role.contributes_nonvolatile_by_default());
        assert!(!role.contributes_runtime_ram_by_default());
        assert!(!role.is_device_metadata());
    }

    #[test]
    fn an_undetermined_role_defaults_to_neither_budget() {
        // A section we cannot classify must not be charged just because it has a name.
        let custom = SectionRole::Unknown;

        assert!(!custom.contributes_nonvolatile_by_default());
        assert!(!custom.contributes_runtime_ram_by_default());
        assert!(!custom.is_device_metadata());
    }
}
