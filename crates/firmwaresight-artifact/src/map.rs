//! GNU ld MAP adapter.
//!
//! This exists because the section header table cannot express where the *loader* stored a
//! section, and `04_TECH/23_MEMORY_ACCOUNTING_MODEL.md` ranks supported MAP region evidence
//! above anything derivable from names. GNU ld writes both facts into its MAP: a
//! `Memory Configuration` table, and a `load address 0x...` suffix on output sections whose
//! load address differs from their runtime address.
//!
//! Adapters are split per toolchain (`04_TECH/03_FORMAT_SUPPORT.md`). An ArmClang or IAR MAP is
//! reported as unsupported; it is never fed to this parser.

use crate::error::ArtifactError;
use firmwaresight_core::domain::memory::{MemoryLayout, MemoryRegion};

pub const ADAPTER_ID: &str = "gnu_ld";

/// One output-section placement line from the memory map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapPlacement {
    pub section: String,
    /// Runtime address.
    pub address: u64,
    pub size: u64,
    /// Where the bytes were stored, when the line says so explicitly.
    pub load_address: Option<u64>,
    pub line: usize,
}

/// One symbol contribution record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapSymbol {
    pub address: u64,
    pub name: String,
    pub line: usize,
}

/// One input-section-to-object contribution record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapObjectContribution {
    pub input_section: String,
    pub address: u64,
    pub size: u64,
    pub object: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEvidence {
    pub adapter_id: &'static str,
    pub regions: Vec<MemoryRegion>,
    pub placements: Vec<MapPlacement>,
    pub symbols: Vec<MapSymbol>,
    pub object_contributions: Vec<MapObjectContribution>,
    /// The `OUTPUT(...)` line records the emulation the linker used.
    pub emulation: Option<String>,
}

impl MapEvidence {
    /// Region table as the memory model wants it. Attributed to the MAP so the accounting
    /// basis is level 2, not a guess.
    #[must_use]
    pub fn layout(&self) -> MemoryLayout {
        MemoryLayout {
            regions: self.regions.clone(),
            source: firmwaresight_core::domain::memory::LayoutSource::MapMemoryConfiguration,
        }
    }

    #[must_use]
    pub fn placement_for(&self, section_name: &str) -> Option<&MapPlacement> {
        self.placements.iter().find(|p| p.section == section_name)
    }
}

/// Decide whether this text is a MAP this adapter may parse.
///
/// Returns `Ok(true)` for GNU ld, `Err(MapUnsupported)` for a recognized foreign toolchain, and
/// `Ok(false)` when nothing identifies it at all. A negative answer is never a silent partial
/// parse.
pub fn detect(text: &str) -> Result<bool, ArtifactError> {
    if is_gnu_ld(text) {
        return Ok(true);
    }
    let foreign = [
        "ARM Linker map information",
        "*** MAP FILE",
        "IAR Linker",
        "RN32/RN42",
        "Linker Map File",
    ];
    for marker in foreign {
        if text.contains(marker) {
            return Err(ArtifactError::MapUnsupported {
                detected: format!("recognized a non-GNU linker MAP banner: {marker}"),
            });
        }
    }
    Ok(false)
}

/// Whether this text carries a GNU ld banner.
///
/// The whole text is searched, not a head window. GNU ld prints its banners behind a preamble whose
/// length the project does not control — an `Archive member included to satisfy reference by file
/// (symbol)` block for every symbol pulled from a static library, and a `Discarded input sections`
/// block under `--gc-sections` — so a fixed window refuses ordinary vendor-HAL builds. This is the
/// same ground [`detect`] already searches for the foreign-toolchain banners.
fn is_gnu_ld(text: &str) -> bool {
    text.contains("Linker script and memory map") || text.contains("Memory Configuration")
}

/// Parse a GNU ld MAP.
pub fn parse(text: &str) -> Result<MapEvidence, ArtifactError> {
    if !detect(text)? {
        return Err(ArtifactError::MapUnsupported {
            detected: "no GNU ld memory-map banner was found".to_owned(),
        });
    }

    let mut regions = Vec::new();
    let mut placements = Vec::new();
    let mut symbols = Vec::new();
    let mut object_contributions = Vec::new();
    let mut emulation = None;

    let mut in_region_table = false;

    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;

        if raw.trim() == "Memory Configuration" {
            in_region_table = true;
            continue;
        }
        if in_region_table {
            // GNU ld prints a blank line between the banner and the column headers, so blanks
            // are skipped rather than treated as the end of the table.
            if raw.trim().is_empty() {
                continue;
            }
            if raw.trim_start().starts_with("Name") {
                continue;
            }
            if let Some(region) = parse_region_line(raw, line)? {
                regions.push(region);
                continue;
            }
            // First non-blank line that is not a row ends the table; fall through to it.
            in_region_table = false;
        }

        if let Some(rest) = raw.strip_prefix("OUTPUT(") {
            emulation = Some(rest.trim_end_matches(')').trim().to_owned());
            continue;
        }

        // Column-zero output section: `.name 0xADDR 0xSIZE [load address 0xADDR]`.
        if let Some(placement) = parse_placement_line(raw, line)? {
            placements.push(placement);
            continue;
        }
        // Indented input section with an object file: ` .name 0xADDR 0xSIZE obj.o`.
        if let Some(contribution) = parse_object_contribution_line(raw, line) {
            object_contributions.push(contribution);
            continue;
        }
        // Address-only continuation line naming a symbol.
        if let Some(symbol) = parse_symbol_line(raw, line) {
            symbols.push(symbol);
        }
    }

    if regions.is_empty() {
        return Err(ArtifactError::MapParseFailed {
            line: 0,
            detail: "the MAP declared no memory regions, so no region evidence exists".to_owned(),
        });
    }

    Ok(MapEvidence {
        adapter_id: ADAPTER_ID,
        regions,
        placements,
        symbols,
        object_contributions,
        emulation,
    })
}

fn parse_hex(token: &str) -> Option<u64> {
    let token = token.strip_prefix("0x").or(token.strip_prefix("0X"))?;
    if token.is_empty() || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    u64::from_str_radix(token, 16).ok()
}

fn parse_region_line(raw: &str, line: usize) -> Result<Option<MemoryRegion>, ArtifactError> {
    let fields: Vec<&str> = raw.split_whitespace().collect();
    if fields.len() < 3 {
        return Ok(None);
    }
    let (name, origin, length) = (fields[0], fields[1], fields[2]);
    let Some(origin) = parse_hex(origin) else {
        // Not a region row; let the caller fall through to other line kinds.
        return Ok(None);
    };
    let Some(length) = parse_hex(length) else {
        return Ok(None);
    };
    if name == "*default*" {
        return Ok(Some(MemoryRegion::new(
            name,
            origin,
            length,
            "",
            format!("map.line{line}"),
        )));
    }
    let attributes = fields.get(3).copied().unwrap_or("");
    Ok(Some(MemoryRegion::new(
        name,
        origin,
        length,
        attributes,
        format!("map.line{line}"),
    )))
}

fn parse_placement_line(raw: &str, line: usize) -> Result<Option<MapPlacement>, ArtifactError> {
    // Output sections start in column zero; input sections are indented.
    if raw.is_empty() || raw.starts_with([' ', '\t']) || !raw.starts_with('.') {
        return Ok(None);
    }
    let head = raw.split_whitespace().next().unwrap_or_default();
    if !head.starts_with('.') {
        return Ok(None);
    }
    let fields: Vec<&str> = raw.split_whitespace().collect();
    if fields.len() < 3 {
        return Ok(None);
    }
    let (Some(address), Some(size)) = (parse_hex(fields[1]), parse_hex(fields[2])) else {
        return Ok(None);
    };

    // `... load address 0x...` is the explicit statement of where the bytes were stored.
    let load_address = match fields.iter().position(|f| *f == "load") {
        Some(position) => match fields.get(position + 2) {
            Some(token) => match parse_hex(token) {
                Some(value) => Some(value),
                None => {
                    return Err(ArtifactError::MapParseFailed {
                        line,
                        detail: "a 'load address' record did not carry a hex value".to_owned(),
                    });
                }
            },
            None => {
                return Err(ArtifactError::MapParseFailed {
                    line,
                    detail: "a 'load address' record was truncated".to_owned(),
                });
            }
        },
        None => None,
    };

    Ok(Some(MapPlacement {
        section: head.to_owned(),
        address,
        size,
        load_address,
        line,
    }))
}

fn parse_object_contribution_line(raw: &str, line: usize) -> Option<MapObjectContribution> {
    if !raw.starts_with([' ', '\t']) {
        return None;
    }
    let fields: Vec<&str> = raw.split_whitespace().collect();
    if fields.len() < 4 {
        return None;
    }
    if !fields[0].starts_with('.') {
        return None;
    }
    let address = parse_hex(fields[1])?;
    let size = parse_hex(fields[2])?;
    let object = fields[3..].join(" ");
    // A committed fixture must not carry a private build-machine path.
    if object.contains(':') || object.starts_with('/') {
        return None;
    }
    Some(MapObjectContribution {
        input_section: fields[0].to_owned(),
        address,
        size,
        object,
        line,
    })
}

fn parse_symbol_line(raw: &str, line: usize) -> Option<MapSymbol> {
    if !raw.starts_with([' ', '\t']) {
        return None;
    }
    let fields: Vec<&str> = raw.split_whitespace().collect();
    if fields.len() != 2 {
        return None;
    }
    let address = parse_hex(fields[0])?;
    let name = fields[1];
    if name.starts_with('.') || name.starts_with("0x") {
        return None;
    }
    Some(MapSymbol {
        address,
        name: name.to_owned(),
        line,
    })
}
