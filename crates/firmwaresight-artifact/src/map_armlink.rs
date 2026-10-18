//! Arm linker (armlink) MAP adapter (ADR-0036).
//!
//! Reads the `Memory Map of the image` that `armlink --map` writes (Arm Compiler 5 and 6, Keil MDK):
//! load regions, execution regions and one row per input section with the object or library member it
//! came from. It produces the same [`MapEvidence`] as the GNU ld adapter, so memory accounting, region
//! usage and object attribution read one shape whichever linker built the image.
//!
//! Facts and their limits:
//! - **Regions** are the scatter file's execution regions: `Exec base` (or `Base`) and `Max`. A region that
//!   holds any RW or ZI row counts against RAM; a region of only RO/XO rows counts against Flash.
//! - **Load addresses** come only from what armlink printed: `Load base` in the region header, which
//!   `--load_addr_map_info` adds (Keil uVision passes it by default). Without it, a region that executes
//!   inside its load region loads where it runs; one that executes elsewhere has an **unknown** load
//!   address, said with its reason, never guessed.
//! - **Compressed RW data** (`COMPRESSED[...]`) is stored in Flash at a size that is not the section's,
//!   so that region's load address is reported unknown rather than charged at the wrong size.
//! - **Object rows** follow ADR-0033: `libx.lib(y.o)` for a library member; `PAD` and veneer rows are the
//!   linker's own bytes and are never charged to an object.

use crate::error::ArtifactError;
use crate::map::{
    LinkerBytesKind, MapEvidence, MapLinkerBytes, MapPlacement, RawContribution,
    normalize_contributions,
};
use firmwaresight_core::domain::memory::MemoryRegion;

pub const ADAPTER_ID: &str = "armlink";

/// Whether this text is an armlink image map this adapter can read: armlink's memory-map banner and at
/// least one execution region. A file with armlink's banner but no memory map is refused by the caller.
#[must_use]
pub fn detect(text: &str) -> bool {
    text.contains("Memory Map of the image") && text.contains("Execution Region ")
}

/// Whether the text names armlink at all (banner, tool line or the image-map heading), even without the
/// parts this adapter needs.
#[must_use]
pub fn mentions_armlink(text: &str) -> bool {
    text.contains("Tool: armlink")
        || text.contains("ARM Linker")
        || text.contains("Memory Map of the image")
        || text.contains("Image component sizes")
}

/// One region header as armlink printed it.
struct RegionHeader {
    name: String,
    exec_base: u64,
    load_base: Option<u64>,
    size: u64,
    max: Option<u64>,
    compressed: bool,
    line: usize,
}

/// The load region an execution region sits in.
struct LoadRegion {
    base: u64,
    max: Option<u64>,
}

/// Parse an armlink image map.
///
/// # Errors
///
/// `MapUnsupported` when the text is not an armlink image map; `MapParseFailed` when a region header or a
/// row cannot be read, or when no execution region carries a usable `Max`.
pub fn parse(text: &str) -> Result<MapEvidence, ArtifactError> {
    if !detect(text) {
        return Err(ArtifactError::MapUnsupported {
            detected: "an armlink file without an image memory map (link with --map)".to_owned(),
        });
    }

    let mut regions = Vec::new();
    let mut placements = Vec::new();
    let mut raw_contributions = Vec::new();
    let mut linker_bytes = Vec::new();

    let mut in_map = false;
    let mut load_region: Option<LoadRegion> = None;
    let mut current: Option<RegionHeader> = None;
    let mut current_has_rw = false;
    let mut has_load_column = false;

    // A region is finished when the next one starts or the map ends; it becomes a region and a placement.
    let finish = |header: Option<RegionHeader>,
                  has_rw: bool,
                  load: &Option<LoadRegion>,
                  regions: &mut Vec<MemoryRegion>,
                  placements: &mut Vec<MapPlacement>|
     -> Result<(), ArtifactError> {
        let Some(header) = header else { return Ok(()) };
        let Some(max) = header.max else {
            return Err(ArtifactError::MapParseFailed {
                line: header.line,
                detail: format!(
                    "execution region {} has no Max, so its capacity is unknown",
                    header.name
                ),
            });
        };
        // An empty region (Size 0, no rows: a RAM region the scatter file reserves) proves nothing about its
        // kind, so it is Unknown rather than counted as Flash capacity.
        let attributes = if has_rw {
            "rw"
        } else if header.size == 0 {
            ""
        } else {
            "rx"
        };
        regions.push(MemoryRegion::new(
            header.name.clone(),
            header.exec_base,
            max,
            attributes,
            format!("map.line{}", header.line),
        ));
        // Without the load region's Max the containment cannot be shown, so nothing is assumed.
        let inside_load_region = load.as_ref().is_some_and(|lr| {
            lr.max
                .is_some_and(|max| header.exec_base >= lr.base && header.exec_base - lr.base < max)
        });
        let (load_address, load_unknown) = if header.compressed {
            (
                None,
                Some(
                    "armlink compressed this region's initialization data (COMPRESSED); its stored size is not the section's size",
                ),
            )
        } else if let Some(load) = header.load_base {
            (Some(load), None)
        } else if inside_load_region {
            (Some(header.exec_base), None)
        } else {
            (
                None,
                Some(
                    "the armlink map gives no Load base for a region that runs outside its load region; link with --load_addr_map_info",
                ),
            )
        };
        placements.push(MapPlacement {
            section: header.name,
            address: header.exec_base,
            size: header.size,
            load_address,
            load_unknown,
            line: header.line,
        });
        Ok(())
    };

    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();

        if trimmed == "Memory Map of the image" {
            in_map = true;
            continue;
        }
        if !in_map {
            continue;
        }
        // The map ends at the next report section (`Image component sizes`, the `===` rule before it).
        if trimmed.starts_with("Image component sizes") || trimmed.starts_with("=====") {
            finish(
                current.take(),
                current_has_rw,
                &load_region,
                &mut regions,
                &mut placements,
            )?;
            in_map = false;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("Load Region ") {
            finish(
                current.take(),
                current_has_rw,
                &load_region,
                &mut regions,
                &mut placements,
            )?;
            let (_, fields) = header_fields(rest, line)?;
            load_region = Some(LoadRegion {
                base: field(&fields, &["Base", "Exec base"], line)?,
                max: field_opt(&fields, "Max"),
            });
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("Execution Region ") {
            finish(
                current.take(),
                current_has_rw,
                &load_region,
                &mut regions,
                &mut placements,
            )?;
            let (name, fields) = header_fields(rest, line)?;
            current = Some(RegionHeader {
                name,
                exec_base: field(&fields, &["Exec base", "Base"], line)?,
                load_base: field_opt(&fields, "Load base"),
                size: field(&fields, &["Size"], line)?,
                max: field_opt(&fields, "Max"),
                compressed: fields.iter().any(|(key, _)| key.starts_with("COMPRESSED")),
                line,
            });
            current_has_rw = false;
            continue;
        }
        if trimmed.starts_with("Exec Addr") {
            has_load_column = true;
            continue;
        }
        if trimmed.starts_with("Base Addr") {
            has_load_column = false;
            continue;
        }
        let Some(header) = current.as_ref() else {
            continue;
        };
        let Some(row) = parse_row(trimmed, has_load_column) else {
            continue;
        };
        let output = Some(header.name.clone());
        match row.kind {
            RowKind::Pad => linker_bytes.push(MapLinkerBytes {
                output_section: output,
                address: row.address,
                size: row.size,
                kind: LinkerBytesKind::Fill,
                line,
            }),
            RowKind::Veneer => linker_bytes.push(MapLinkerBytes {
                output_section: output,
                address: row.address,
                size: row.size,
                kind: LinkerBytesKind::LinkerStubs,
                line,
            }),
            RowKind::Section {
                rw,
                ref name,
                ref object,
            } => {
                current_has_rw |= rw;
                raw_contributions.push(RawContribution {
                    input_section: name.clone(),
                    output_section: output,
                    address: row.address,
                    size: row.size,
                    object: object.clone(),
                    line,
                });
            }
        }
    }
    finish(
        current.take(),
        current_has_rw,
        &load_region,
        &mut regions,
        &mut placements,
    )?;

    if regions.is_empty() {
        return Err(ArtifactError::MapParseFailed {
            line: 0,
            detail: "the armlink map declared no execution region".to_owned(),
        });
    }

    Ok(MapEvidence {
        adapter_id: ADAPTER_ID,
        regions,
        placements,
        symbols: Vec::new(),
        object_contributions: normalize_contributions(raw_contributions),
        linker_bytes,
        emulation: None,
    })
}

/// `NAME (Key: 0x.., Key: 0x.., FLAG, ...)` → the name and its key/value fields.
fn header_fields(
    rest: &str,
    line: usize,
) -> Result<(String, Vec<(String, String)>), ArtifactError> {
    let (name, inner) = rest
        .split_once('(')
        .and_then(|(name, tail)| tail.rsplit_once(')').map(|(inner, _)| (name.trim(), inner)))
        .ok_or_else(|| ArtifactError::MapParseFailed {
            line,
            detail: "a region header without its (Base: ..., Size: ...) list".to_owned(),
        })?;
    let fields = inner
        .split(',')
        .map(|part| match part.split_once(':') {
            Some((key, value)) => (key.trim().to_owned(), value.trim().to_owned()),
            None => (part.trim().to_owned(), String::new()),
        })
        .collect();
    Ok((name.to_owned(), fields))
}

fn field(fields: &[(String, String)], keys: &[&str], line: usize) -> Result<u64, ArtifactError> {
    keys.iter()
        .find_map(|key| field_opt(fields, key))
        .ok_or_else(|| ArtifactError::MapParseFailed {
            line,
            detail: format!("a region header without {}", keys[0]),
        })
}

fn field_opt(fields: &[(String, String)], key: &str) -> Option<u64> {
    fields
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, value)| parse_hex(value))
}

fn parse_hex(token: &str) -> Option<u64> {
    let digits = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))?;
    u64::from_str_radix(digits, 16).ok()
}

enum RowKind {
    Pad,
    Veneer,
    Section {
        rw: bool,
        name: String,
        object: String,
    },
}

struct Row {
    address: u64,
    size: u64,
    kind: RowKind,
}

/// One row of an execution region:
/// `Exec Addr [Load Addr|-] Size Type Attr Idx [*] Section-Name Object`, or `Addr [Load] Size PAD`.
fn parse_row(trimmed: &str, has_load_column: bool) -> Option<Row> {
    let fields: Vec<&str> = trimmed.split_whitespace().collect();
    let address = parse_hex(fields.first()?)?;
    let size_at = if has_load_column { 2 } else { 1 };
    let size = parse_hex(fields.get(size_at)?)?;
    let kind = *fields.get(size_at + 1)?;
    if kind == "PAD" {
        return Some(Row {
            address,
            size,
            kind: RowKind::Pad,
        });
    }
    if kind.starts_with("Ven") {
        return Some(Row {
            address,
            size,
            kind: RowKind::Veneer,
        });
    }
    if !matches!(kind, "Code" | "Data" | "Zero") {
        return None;
    }
    let attr = *fields.get(size_at + 2)?;
    // Idx, then an optional `*` (the entry section), then the section name and the object.
    let mut rest = fields.get(size_at + 4..)?.to_vec();
    if rest.first() == Some(&"*") {
        rest.remove(0);
    }
    if rest.len() < 2 {
        return None;
    }
    let object = rest.pop()?.to_owned();
    let name = rest.join(" ");
    Some(Row {
        address,
        size,
        kind: RowKind::Section {
            rw: attr == "RW" || kind == "Zero",
            name,
            object,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const WITH_LOAD: &str = "\
Memory Map of the image

  Load Region LR_IROM1 (Base: 0x08000000, Size: 0x000000c0, Max: 0x00010000, ABSOLUTE)

    Execution Region ER_IROM1 (Exec base: 0x08000000, Load base: 0x08000000, Size: 0x000000bc, Max: 0x00010000, ABSOLUTE)

    Exec Addr    Load Addr    Size         Type   Attr      Idx    E Section Name        Object

    0x08000000   0x08000000   0x0000003a   Code   RO            2  * .text.Reset_Handler  main.o
    0x08000066   0x08000066   0x00000002   PAD
    0x08000068   0x08000068   0x00000018   Code   RO           21    .text.spi_flush     libdrv.lib(spi.o)

    Execution Region RW_IRAM1 (Exec base: 0x20000000, Load base: 0x080000bc, Size: 0x0000002c, Max: 0x00004000, ABSOLUTE)

    Exec Addr    Load Addr    Size         Type   Attr      Idx    E Section Name        Object

    0x20000000   0x080000bc   0x00000004   Data   RW            6    .data.g_threshold   main.o
    0x20000004        -       0x00000020   Zero   RW           23    .bss.spi_buffer     libdrv.lib(spi.o)

==============================================================================
";

    #[test]
    fn regions_placements_and_rows_come_from_the_image_map() {
        let evidence = parse(WITH_LOAD).expect("parses");
        assert_eq!(evidence.adapter_id, "armlink");
        let names: Vec<_> = evidence
            .regions
            .iter()
            .map(|r| (r.name.as_str(), r.origin, r.length))
            .collect();
        assert_eq!(
            names,
            vec![
                ("ER_IROM1", 0x0800_0000, 0x1_0000),
                ("RW_IRAM1", 0x2000_0000, 0x4000)
            ]
        );
        assert_eq!(
            evidence.regions[0].kind,
            firmwaresight_core::domain::memory::RegionKind::Nonvolatile
        );
        assert_eq!(
            evidence.regions[1].kind,
            firmwaresight_core::domain::memory::RegionKind::VolatileRam
        );
        assert_eq!(evidence.placements[1].load_address, Some(0x0800_00bc));
        assert_eq!(evidence.placements[1].load_unknown, None);
        let keys: Vec<_> = evidence
            .object_contributions
            .iter()
            .map(|r| r.object.as_str())
            .collect();
        assert_eq!(
            keys,
            vec!["main.o", "libdrv.lib(spi.o)", "main.o", "libdrv.lib(spi.o)"]
        );
        assert_eq!(
            evidence.object_contributions[1].archive.as_deref(),
            Some("libdrv.lib")
        );
        assert_eq!(evidence.object_contributions[1].member, "spi.o");
        assert_eq!(
            evidence.linker_bytes.len(),
            1,
            "PAD is the linker's, never an object's"
        );
    }

    #[test]
    fn a_region_running_outside_its_load_region_without_a_load_base_is_unknown_not_guessed() {
        let old_style = WITH_LOAD
            .replace(
                "Exec base: 0x08000000, Load base: 0x08000000,",
                "Base: 0x08000000,",
            )
            .replace(
                "Exec base: 0x20000000, Load base: 0x080000bc,",
                "Base: 0x20000000,",
            );
        let evidence = parse(&old_style).expect("parses");
        assert_eq!(
            evidence.placements[0].load_address,
            Some(0x0800_0000),
            "runs inside its load region"
        );
        assert_eq!(evidence.placements[1].load_address, None);
        assert!(
            evidence.placements[1]
                .load_unknown
                .is_some_and(|why| why.contains("--load_addr_map_info"))
        );
    }

    #[test]
    fn compressed_initialization_data_is_never_charged_at_the_section_size() {
        let compressed = WITH_LOAD.replace(
            "Max: 0x00004000, ABSOLUTE)",
            "Max: 0x00004000, ABSOLUTE, COMPRESSED[0x00000002])",
        );
        let evidence = parse(&compressed).expect("parses");
        assert_eq!(evidence.placements[1].load_address, None);
        assert!(
            evidence.placements[1]
                .load_unknown
                .is_some_and(|why| why.contains("COMPRESSED"))
        );
    }

    #[test]
    fn an_empty_region_is_unknown_and_a_load_region_without_max_proves_no_containment() {
        let empty = WITH_LOAD.replace(
            "Execution Region RW_IRAM1 (Exec base: 0x20000000, Load base: 0x080000bc, Size: 0x0000002c,",
            "Execution Region RW_IRAM1 (Exec base: 0x20000000, Load base: 0x080000bc, Size: 0x00000000,",
        );
        let rows: Vec<&str> = empty
            .lines()
            .filter(|l| !l.trim_start().starts_with("0x2000000"))
            .collect();
        let evidence = parse(&rows.join("\n")).expect("parses");
        assert_eq!(
            evidence.regions[1].kind,
            firmwaresight_core::domain::memory::RegionKind::Unknown
        );

        let no_lr_max = WITH_LOAD
            .replace(
                "Size: 0x000000c0, Max: 0x00010000, ABSOLUTE)",
                "Size: 0x000000c0, ABSOLUTE)",
            )
            .replace(
                "Exec base: 0x08000000, Load base: 0x08000000,",
                "Base: 0x08000000,",
            );
        let evidence = parse(&no_lr_max).expect("parses");
        assert_eq!(
            evidence.placements[0].load_address, None,
            "containment not shown, so not assumed"
        );
    }

    #[test]
    fn text_without_an_image_map_is_refused_not_half_read() {
        let banner_only = "Component: Arm Compiler for Embedded 6.24 Tool: armlink [5f371500]\n";
        assert!(mentions_armlink(banner_only));
        assert!(matches!(
            parse(banner_only),
            Err(ArtifactError::MapUnsupported { .. })
        ));
    }

    #[test]
    fn a_region_without_max_is_a_parse_error_with_its_line() {
        let no_max = WITH_LOAD.replace(", Max: 0x00004000", "");
        match parse(&no_max) {
            Err(ArtifactError::MapParseFailed { line, detail }) => {
                assert!(line > 0 && detail.contains("RW_IRAM1"), "{line} {detail}");
            }
            other => panic!("expected a parse error, got {other:?}"),
        }
    }
}
