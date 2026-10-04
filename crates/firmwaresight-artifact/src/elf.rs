//! ELF parsing through the `object` crate.
//!
//! Everything returned here is already normalized into core domain types; `object`'s own types
//! never leave this module.
//!
//! Section attributes are derived from `object`'s format-agnostic `SectionKind`, which the
//! parser itself computes from the ELF section type and flags. That keeps this module off
//! format-specific flag constants while preserving the same distinctions the memory model needs:
//! code, read-only data, initialized data and zero-initialized data.

use object::read::{Object, ObjectSection, ObjectSymbol};
use object::{Architecture as ObjectArchitecture, SectionKind, SymbolKind as ObjectSymbolKind};

use crate::error::ArtifactError;
use firmwaresight_core::domain::capability::{Availability, FormatSupport};
use firmwaresight_core::domain::identity::{
    Architecture, ArtifactKind, Bitness, Endianness, Fact, Sha256,
};
use firmwaresight_core::domain::section::{Section, SectionFlags, SectionRole};
use firmwaresight_core::domain::symbol::{Symbol, SymbolBinding, SymbolKind, SymbolSectionRef};

/// Parser identity recorded on every artifact.
pub const PARSER_ID: &str = "object-elf/0.40";

#[derive(Debug, Clone)]
pub struct ElfFacts {
    pub architecture: Architecture,
    pub bitness: Bitness,
    pub endianness: Endianness,
    pub entry_point: Fact<u64>,
    pub build_id: Fact<String>,
    pub kind: ArtifactKind,
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    pub symbol_table_present: bool,
    pub debug_sections_present: bool,
    pub elf_support: FormatSupport,
    pub debug_support: Availability,
}

fn map_architecture(value: ObjectArchitecture) -> Architecture {
    match value {
        ObjectArchitecture::Arm => Architecture::Arm,
        ObjectArchitecture::Aarch64 | ObjectArchitecture::Aarch64_Ilp32 => Architecture::AArch64,
        ObjectArchitecture::X86_64 | ObjectArchitecture::X86_64_X32 => Architecture::X86_64,
        ObjectArchitecture::Mips => Architecture::Mips,
        ObjectArchitecture::Mips64 | ObjectArchitecture::Mips64_N32 => Architecture::Mips,
        other => Architecture::Other(format!("{other:?}")),
    }
}

/// Names that identify host/toolchain metadata regardless of how the parser classified them.
///
/// `object` reports `.debug_*` and `.debug_frame` as `SectionKind::Other`, so relying on kind
/// alone would charge DWARF bytes to the device budget — the exact mistake
/// `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` forbids.
fn classify_by_name(name: &str) -> Option<SectionRole> {
    if name.starts_with(".debug") || name.starts_with(".zdebug") {
        return Some(SectionRole::Debug);
    }
    if name.starts_with(".ARM.attributes") || name == ".ARM.exidx" {
        return Some(SectionRole::ArmAttribute);
    }
    match name {
        ".symtab" | ".dynsym" => Some(SectionRole::SymbolTable),
        ".strtab" | ".dynstr" | ".shstrtab" => Some(SectionRole::StringTable),
        ".comment" | ".note.GNU-stack" => Some(SectionRole::Note),
        _ => None,
    }
}

/// Role plus the alloc/write/execute attributes, from the parser's section classification.
///
/// `alloc` is the ELF's own `SHF_ALLOC` bit, not an inference from `SectionKind`. The inference used
/// to read "kind I do not recognise" as "not allocated", which silently dropped a section the header
/// said occupies the image: clang emits `.ARM.exidx.text.main` with `SHF_ALLOC|SHF_LINK_ORDER` inside
/// a declared FLASH region, and the dual-budget model charged it to neither side while reporting the
/// total as `Exact` with nothing listed as unattributed. `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` §3
/// says a custom section is settled by segment and region evidence and never by guessing from its
/// name, so honouring the bit is what the model already required.
///
/// The metadata-name net still overrides it: bytes the product knows to be host tooling stay out of
/// the device budget whatever their flags say, which is the mistake this function was written to
/// avoid. `write` and `execute` stay as they were — the finding was about allocation, and changing
/// those would move classifications no evidence here asked to move.
fn map_section_kind(
    kind: SectionKind,
    name: Option<&str>,
    is_alloc: bool,
) -> (SectionRole, SectionFlags) {
    let named_role = name.and_then(classify_by_name);

    let alloc = is_alloc && named_role.is_none();
    let write = matches!(
        kind,
        SectionKind::Data
            | SectionKind::ReadOnlyDataWithRel
            | SectionKind::UninitializedData
            | SectionKind::Tls
            | SectionKind::UninitializedTls
            | SectionKind::TlsVariables
    );
    let execute = matches!(kind, SectionKind::Text);

    let role = match kind {
        SectionKind::Text => SectionRole::Code,
        SectionKind::Data => SectionRole::InitializedData,
        SectionKind::ReadOnlyData
        | SectionKind::ReadOnlyDataWithRel
        | SectionKind::ReadOnlyString => SectionRole::ReadOnlyData,
        SectionKind::UninitializedData
        | SectionKind::UninitializedTls
        | SectionKind::Tls
        | SectionKind::TlsVariables => SectionRole::UninitializedData,
        SectionKind::Debug | SectionKind::DebugString => SectionRole::Debug,
        SectionKind::Note => SectionRole::Note,
        SectionKind::OtherString => SectionRole::StringTable,
        SectionKind::Linker => SectionRole::Other,
        SectionKind::Metadata => SectionRole::Other,
        SectionKind::Unknown | SectionKind::Other => match name {
            Some(n) if n.starts_with(".ARM.attributes") => SectionRole::ArmAttribute,
            Some(n) if n == ".symtab" || n == ".dynsym" => SectionRole::SymbolTable,
            Some(n) if n.starts_with(".strtab") || n.starts_with(".dynstr") => {
                SectionRole::StringTable
            }
            // Unrecognized stays unrecognized: it is neither budgeted nor assumed to be
            // host metadata.
            _ => SectionRole::Unknown,
        },
        _ => SectionRole::Unknown,
    };
    // A recognized metadata name outranks whatever generic classification the parser offered.
    let role = named_role.unwrap_or(role);

    (
        role,
        SectionFlags {
            alloc,
            execute,
            write,
            merge: false,
            strings: matches!(kind, SectionKind::ReadOnlyString | SectionKind::OtherString),
            tls: matches!(
                kind,
                SectionKind::Tls | SectionKind::UninitializedTls | SectionKind::TlsVariables
            ),
            compressed: false,
        },
    )
}

fn map_symbol_kind(kind: ObjectSymbolKind) -> SymbolKind {
    match kind {
        // `object` reports a function symbol as `Text`; there is no separate Function variant.
        ObjectSymbolKind::Text => SymbolKind::Function,
        ObjectSymbolKind::Data => SymbolKind::Object,
        ObjectSymbolKind::Section => SymbolKind::Section,
        ObjectSymbolKind::File => SymbolKind::File,
        ObjectSymbolKind::Label => SymbolKind::NotType,
        ObjectSymbolKind::Tls => SymbolKind::Common,
        ObjectSymbolKind::Unknown => SymbolKind::Unknown,
        other => SymbolKind::Other(u8::try_from(other as i64).unwrap_or(u8::MAX)),
    }
}

/// Binding comes from the parser's scope predicates rather than from a raw `st_info` byte.
#[must_use]
pub fn classify_binding(
    is_weak: bool,
    is_global: bool,
    is_local: bool,
    is_common: bool,
) -> SymbolBinding {
    if is_weak {
        SymbolBinding::Weak
    } else if is_global {
        SymbolBinding::Global
    } else if is_local {
        SymbolBinding::Local
    } else if is_common {
        SymbolBinding::Global
    } else {
        SymbolBinding::Unknown
    }
}

fn map_section_index(value: Option<object::SectionIndex>) -> SymbolSectionRef {
    match value {
        Some(index) => SymbolSectionRef::Index(index.0),
        None => SymbolSectionRef::Unknown,
    }
}

/// Parse ELF bytes that were already size-guarded and hashed.
pub fn parse(bytes: &[u8], _sha256: &Sha256) -> Result<ElfFacts, ArtifactError> {
    let file = object::File::parse(bytes).map_err(|err| ArtifactError::InvalidElf {
        detail: format!("object could not parse the ELF container: {err}"),
    })?;

    let bitness = if file.is_64() {
        Bitness::Bits64
    } else {
        Bitness::Bits32
    };
    let endianness = if file.is_little_endian() {
        Endianness::Little
    } else {
        Endianness::Big
    };
    let entry_point = Fact::known(file.entry());

    let mut sections = Vec::new();
    let mut debug_sections_present = false;

    for section in file.sections() {
        // The parser's own index, so a locator like `elf.section_header[7]` can be checked
        // against readelf output rather than against an iteration counter.
        let index = section.index().0;
        let name: Option<&str> = section.name().ok().filter(|value| !value.is_empty());
        let kind = section.kind();
        // `SHF_ALLOC` straight from the section header. `SectionKind::Unknown` means the parser did
        // not recognise the section, which is not the same fact as the file not being loaded.
        let allocated = matches!(
            section.flags(),
            object::SectionFlags::Elf { sh_flags, .. } if sh_flags.contains(object::elf::SHF_ALLOC)
        );
        let (role, flags) = map_section_kind(kind, name, allocated);

        // `data()` yields the bytes actually stored in the file. A NOBITS section such as `.bss`
        // reports an empty slice, which is the distinction the memory model depends on.
        let payload_len = section.data().map(|data| data.len() as u64).unwrap_or(0);
        let memory_size = section.size();

        if matches!(role, SectionRole::Debug) {
            debug_sections_present = true;
        }

        sections.push(Section {
            index,
            name: match name {
                Some(value) => Fact::known(value.to_owned()),
                None => Fact::unknown("the section header table carries no name for this section"),
            },
            role,
            flags,
            virtual_address: if section.address() == 0 && role.is_device_metadata() {
                Fact::unknown("this section is host metadata and is not loaded at runtime")
            } else {
                Fact::known(section.address())
            },
            // The section header table has no physical-address field. The GNU ld MAP supplies
            // it as an explicit `load address` record; see `map.rs`.
            load_address: Fact::unknown("not present in the section header table"),
            file_offset: match section.file_range() {
                Some((start, _)) => Fact::known(start),
                None => {
                    Fact::unknown("the section has no file range; it occupies no bytes on disk")
                }
            },
            file_size: payload_len,
            memory_size: if memory_size == 0 && payload_len == 0 {
                Fact::unknown("the section declares no size and carries no payload")
            } else {
                Fact::known(memory_size)
            },
            region: Fact::unknown("no memory-region evidence was supplied"),
        });
    }

    let mut symbols = Vec::new();
    for symbol in file.symbols() {
        let name: Option<&str> = symbol.name().ok().filter(|value| !value.is_empty());
        symbols.push(Symbol {
            name: match name {
                Some(value) => Fact::known(value.to_owned()),
                None => Fact::unknown("the symbol entry has an empty name"),
            },
            address: Fact::known(symbol.address()),
            size: if symbol.size() == 0 {
                Fact::unknown("the symbol entry records size 0")
            } else {
                Fact::known(symbol.size())
            },
            kind: map_symbol_kind(symbol.kind()),
            binding: classify_binding(
                symbol.is_weak(),
                symbol.is_global(),
                symbol.is_local(),
                symbol.is_common(),
            ),
            section: map_section_index(symbol.section_index()),
        });
    }

    // Reported from what was actually read rather than from a container query: a stripped
    // object and an object with an empty table both mean no symbol evidence is available.
    let symbol_table_present = !symbols.is_empty();

    let build_id = extract_build_id(&file);

    Ok(ElfFacts {
        architecture: map_architecture(file.architecture()),
        bitness,
        endianness,
        entry_point,
        kind: ArtifactKind::Elf,
        build_id,
        sections,
        symbols,
        symbol_table_present,
        debug_sections_present,
        elf_support: FormatSupport::Supported,
        debug_support: if debug_sections_present {
            Availability::Available
        } else {
            Availability::Unavailable
        },
    })
}

/// Build-id from `.note.gnu.build-id`, with explicit bounds checks on every length read from
/// the file. Absent is normal for linked firmware, so it is Unknown rather than a failure.
fn extract_build_id(file: &object::File<'_>) -> Fact<String> {
    let Some(section) = file.section_by_name(".note.gnu.build-id") else {
        return Fact::unknown("no .note.gnu.build-id section is present");
    };
    let Ok(data) = section.data() else {
        return Fact::unknown("the build-id note section could not be read");
    };

    // Elf32_Nhdr: namesz(4) descsz(4) type(4), then the name padded up to a 4-byte boundary,
    // then the descriptor.
    if data.len() < 12 {
        return Fact::unknown("the build-id note header is truncated");
    }
    let namesz = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let descsz = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;

    let Some(name_end) = 12usize.checked_add(namesz) else {
        return Fact::unknown("the build-id note name length overflowed");
    };
    if name_end > data.len() {
        return Fact::unknown("the build-id note name runs past the end of the section");
    }
    let desc_start = name_end.div_ceil(4) * 4;
    let Some(desc_end) = desc_start.checked_add(descsz) else {
        return Fact::unknown("the build-id note descriptor length overflowed");
    };
    if desc_end > data.len() {
        return Fact::unknown("the build-id note descriptor runs past the end of the section");
    }

    let hex = crate::intake::lowercase_hex(&data[desc_start..desc_end]);
    if hex.is_empty() {
        Fact::unknown("the build-id note carried an empty descriptor")
    } else {
        Fact::known(hex)
    }
}
