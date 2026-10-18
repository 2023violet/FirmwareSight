//! IAR ILINK MAP adapter (ADR-0038).
//!
//! Reads the map that IAR Embedded Workbench for Arm writes with `--map` (ILINK 7.x to 9.x): the
//! `PLACEMENT SUMMARY` with the linker configuration's placement directives and one row per placed section,
//! and the `[n] = ...` index that says which library or object directory each row came from. It produces the
//! same [`MapEvidence`] as the GNU ld and armlink adapters.
//!
//! Facts and their limits:
//! - **Regions** are the address ranges of the `place in [from A to B]` directives, the ranges the `.icf`
//!   file gives the linker. A range holding any `inited`, `zero`, `uninit`, `noinit` or `rw code` row counts
//!   against RAM; a range of only `ro code` and `const` rows counts against Flash; an empty range is of Unknown
//!   kind. The kind is derived from what was placed there, as armlink's is (ADR-0036). Ranges that share any
//!   address are one region over their union, so no capacity is counted twice. `place at` directives fix a
//!   section's address inside a range and are not regions themselves.
//! - **Placements** are not produced. IAR copy-initializes RW data from `Initializer bytes` that sit in the
//!   read-only block, so the ELF's own program headers already describe what Flash stores; there is no load
//!   address to add.
//! - **Object rows** follow ADR-0033: `lib.a(member.o)` for a library member and the bare object name for an
//!   object from one of the project's object directories, never a host path. `Initializer bytes`,
//!   `- Linker created -` rows and `<Block>` headers are the linker's own and are never charged to an object.
//!   IAR names its ELF sections after placement blocks (`P1 ro`, `P2 zi`), which the map does not print, so a
//!   row's output section is resolved from its address against the ELF afterwards.

use crate::error::ArtifactError;
use crate::map::{MapEvidence, MapObjectContribution, RawContribution, normalize_contributions};
use firmwaresight_core::domain::memory::MemoryRegion;

pub const ADAPTER_ID: &str = "iar_ilink";

/// Whether the text is an IAR ILINK map this adapter could read: ILINK's banner and its placement summary.
#[must_use]
pub fn detect(text: &str) -> bool {
    text.contains("IAR ELF Linker") && text.contains("*** PLACEMENT SUMMARY")
}

/// Whether the text names IAR's linker at all, even without the parts this adapter needs.
#[must_use]
pub fn mentions_iar(text: &str) -> bool {
    text.contains("IAR ELF Linker") || text.contains("IAR Linker")
}

/// A row's kind column as ILINK prints it.
const KINDS: [&str; 7] = [
    "ro code", "rw code", "const", "inited", "zero", "uninit", "noinit",
];

fn is_ram_kind(kind: &str) -> bool {
    matches!(kind, "inited" | "zero" | "uninit" | "noinit" | "rw code")
}

/// More ranges than any linker configuration declares: past this the text is not a real map, and the
/// overlap merge below would only cost time.
const MAX_RANGES: usize = 4096;

/// One `place in` address range.
struct Range {
    labels: Vec<String>,
    from: u64,
    to: u64,
    line: usize,
}

/// One placed section row.
struct Row {
    name: String,
    kind: Option<String>,
    address: u64,
    size: u64,
    object: String,
    index: Option<usize>,
}

/// Parse an IAR ILINK map.
///
/// # Errors
///
/// `MapUnsupported` when the text is not an ILINK map for Arm; `MapParseFailed` when a placement range or a
/// row cannot be read, or when no `place in` range is declared.
pub fn parse(text: &str) -> Result<MapEvidence, ArtifactError> {
    if !detect(text) {
        return Err(ArtifactError::MapUnsupported {
            detected: "an IAR file without a placement summary (link with --map)".to_owned(),
        });
    }
    if let Some(banner) = text.lines().find(|l| l.contains("IAR ELF Linker"))
        && !banner.contains(" for ARM")
    {
        let target = banner
            .split(" for ")
            .nth(1)
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap_or("another target");
        return Err(ArtifactError::MapUnsupported {
            detected: format!(
                "an IAR ILINK map for {target}; only IAR Embedded Workbench for Arm is supported"
            ),
        });
    }

    let index = library_index(text);
    let mut ranges: Vec<Range> = Vec::new();
    let mut rows: Vec<(usize, Row)> = Vec::new();

    #[derive(PartialEq)]
    enum Part {
        Before,
        Directives,
        Table,
        After,
    }
    let mut part = Part::Before;
    let mut statement = String::new();
    let mut statement_line = 0;
    let mut saw_table = false;

    for (number, raw) in text.lines().enumerate() {
        let line = number + 1;
        let trimmed = raw.trim();
        match part {
            Part::Before => {
                if trimmed == "*** PLACEMENT SUMMARY" {
                    part = Part::Directives;
                }
            }
            Part::Directives => {
                if is_table_header(trimmed) {
                    // A directive the linker printed without its closing `;` still counts.
                    directive_ranges(&statement, statement_line, &mut ranges)?;
                    statement.clear();
                    part = Part::Table;
                    saw_table = true;
                    continue;
                }
                if trimmed.is_empty() || trimmed.starts_with("***") {
                    continue;
                }
                if statement.is_empty() {
                    statement_line = line;
                }
                statement.push(' ');
                statement.push_str(trimmed);
                if trimmed.ends_with(';') {
                    directive_ranges(&statement, statement_line, &mut ranges)?;
                    statement.clear();
                }
                if ranges.len() > MAX_RANGES {
                    return Err(parse_failed(
                        line,
                        format!(
                            "more than {MAX_RANGES} placement ranges; this is not a linker map"
                        ),
                    ));
                }
            }
            Part::Table => {
                if trimmed.starts_with("Unused ranges") || trimmed.starts_with("***") {
                    part = Part::After;
                    continue;
                }
                if let Some(row) = parse_row(raw) {
                    rows.push((line, row));
                }
            }
            Part::After => break,
        }
    }

    if !saw_table {
        return Err(ArtifactError::MapParseFailed {
            line: 0,
            detail:
                "the IAR map's placement summary has no `Section Kind Address Size Object` table"
                    .to_owned(),
        });
    }
    let ranges = merge_overlapping(ranges);
    if ranges.is_empty() {
        return Err(ArtifactError::MapParseFailed {
            line: 0,
            detail: "the IAR map's placement summary declares no `place in [from .. to ..]` range"
                .to_owned(),
        });
    }

    let regions = ranges
        .iter()
        .map(|range| {
            let inside = |row: &Row| row.address >= range.from && row.address <= range.to;
            let placed: Vec<&Row> = rows
                .iter()
                .map(|(_, r)| r)
                .filter(|r| r.kind.is_some() && inside(r))
                .collect();
            let attributes = if placed
                .iter()
                .any(|r| r.kind.as_deref().is_some_and(is_ram_kind))
            {
                "rw"
            } else if placed.is_empty() {
                ""
            } else {
                "rx"
            };
            MemoryRegion::new(
                range.labels.join("+"),
                range.from,
                (range.to - range.from).saturating_add(1),
                attributes,
                format!("map.line{}", range.line),
            )
        })
        .collect();

    let raw: Vec<RawContribution> = rows
        .iter()
        .filter(|(_, row)| row.kind.is_some() && row.size > 0)
        .filter_map(|(line, row)| {
            let object = object_key(row, &index)?;
            Some(RawContribution {
                input_section: row.name.clone(),
                output_section: None,
                address: row.address,
                size: row.size,
                object,
                line: *line,
            })
        })
        .collect();

    Ok(MapEvidence {
        adapter_id: ADAPTER_ID,
        regions,
        placements: Vec::new(),
        symbols: Vec::new(),
        object_contributions: normalize_contributions(raw),
        linker_bytes: Vec::new(),
        emulation: None,
    })
}

/// Give each row that its adapter could not name an output section the ELF section holding its address
/// (ADR-0038: IAR names ELF sections after placement blocks, which its map does not print). The smallest
/// holding section wins, and it must hold the whole row; a row no allocated section holds keeps `None` and its
/// charge stays Unknown.
pub fn resolve_output_sections(
    rows: &mut [MapObjectContribution],
    sections: &[firmwaresight_core::domain::section::Section],
) {
    for row in rows.iter_mut().filter(|r| r.output_section.is_none()) {
        row.output_section = sections
            .iter()
            .filter(|s| s.flags.alloc)
            .filter_map(|s| {
                let name = s.name.value()?;
                let start = *s.virtual_address.value()?;
                let size = *s.memory_size.value()?;
                let end = start.checked_add(size)?;
                let row_end = row.address.checked_add(row.size)?;
                (size > 0 && row.address >= start && row_end <= end).then_some((size, name))
            })
            .min_by_key(|(size, _)| *size)
            .map(|(_, name)| name.clone());
    }
}

/// Ranges that share any address become one region over their union, named after every directive in it, so
/// no byte of capacity is counted twice (two directives over one range, or a range nested in another). A
/// directive that lists several disjoint ranges names them `P1`, `P1 (2)`, ...
fn merge_overlapping(mut ranges: Vec<Range>) -> Vec<Range> {
    ranges.sort_by_key(|r| (r.from, r.to));
    let mut merged: Vec<Range> = Vec::new();
    for range in ranges {
        match merged.last_mut() {
            Some(last) if range.from <= last.to => {
                last.to = last.to.max(range.to);
                last.line = last.line.min(range.line);
                for label in range.labels {
                    if !last.labels.contains(&label) {
                        last.labels.push(label);
                    }
                }
            }
            _ => merged.push(range),
        }
    }
    let mut seen: Vec<String> = Vec::new();
    for range in &mut merged {
        let name = range.labels.join("+");
        let count = seen.iter().filter(|n| **n == name).count();
        seen.push(name.clone());
        if count > 0 {
            range.labels = vec![format!("{name} ({})", count + 1)];
        }
    }
    merged
}

fn is_table_header(trimmed: &str) -> bool {
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    words == ["Section", "Kind", "Address", "Size", "Object"]
}

/// Every `[from A to B]` (or `[from A size N]`) range of a `"label": place in ...;` statement.
fn directive_ranges(
    statement: &str,
    line: usize,
    ranges: &mut Vec<Range>,
) -> Result<(), ArtifactError> {
    let statement = statement.trim();
    let Some(rest) = statement.strip_prefix('"') else {
        return Ok(());
    };
    let Some((label, body)) = rest.split_once("\":") else {
        return Ok(());
    };
    let body = body.trim_start();
    if !body.starts_with("place in") {
        return Ok(());
    }
    let mut remaining = body;
    while let Some(open) = remaining.find("[from ") {
        let after = &remaining[open + "[from ".len()..];
        let Some(close) = after.find(']') else {
            return Err(parse_failed(
                line,
                format!("an unterminated range in the placement directive \"{label}\""),
            ));
        };
        let inner: Vec<&str> = after[..close].split_whitespace().collect();
        let (from, to) = match inner.as_slice() {
            [from, "to", to] => (hex(from), hex(to)),
            [from, "size", size] => {
                let from = hex(from);
                (
                    from,
                    from.zip(hex(size))
                        .and_then(|(f, s)| f.checked_add(s)?.checked_sub(1)),
                )
            }
            _ => (None, None),
        };
        let (Some(from), Some(to)) = (from, to) else {
            return Err(parse_failed(
                line,
                format!(
                    "the range [from {}] of \"{label}\" cannot be read",
                    &after[..close]
                ),
            ));
        };
        if to < from {
            return Err(parse_failed(
                line,
                format!("the range of \"{label}\" ends before it starts"),
            ));
        }
        ranges.push(Range {
            labels: vec![label.to_owned()],
            from,
            to,
            line,
        });
        remaining = &after[close..];
    }
    Ok(())
}

/// `  name [kind]  0xADDR  0xSIZE  object [n]` on one line. Block headers, range ends and part titles are
/// read too (their kind is `None` or their name is `-`) so that callers can skip them by shape.
fn parse_row(raw: &str) -> Option<Row> {
    if !raw.starts_with([' ', '\t']) {
        return None;
    }
    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let at = (0..tokens.len().saturating_sub(1))
        .find(|&i| hex(tokens[i]).is_some() && hex(tokens[i + 1]).is_some())?;
    let head = &tokens[..at];
    if head.is_empty() || head == ["-"] {
        return None;
    }
    let address = hex(tokens[at])?;
    let size = hex(tokens[at + 1])?;
    let joined = head.join(" ");
    let (name, kind) = KINDS
        .iter()
        .find_map(|kind| {
            joined
                .strip_suffix(kind)
                .filter(|name| name.ends_with(' '))
                .map(|name| (name.trim_end().to_owned(), Some((*kind).to_owned())))
        })
        .unwrap_or((joined.clone(), None));
    let tail = &tokens[at + 2..];
    let (object, index) = match tail {
        [.., last] if last.starts_with('[') && last.ends_with(']') => (
            tail[..tail.len() - 1].join(" "),
            last[1..last.len() - 1].parse().ok(),
        ),
        _ => (tail.join(" "), None),
    };
    Some(Row {
        name,
        kind,
        address,
        size,
        object,
        index,
    })
}

/// The `[n] = ...` lines: `Some(archive)` for a library, `None` for an object directory or the command line.
fn library_index(text: &str) -> Vec<(usize, Option<String>)> {
    text.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix('[')?;
            let (number, value) = rest.split_once("] = ")?;
            let number: usize = number.parse().ok()?;
            let value = value.trim();
            let file = value.rsplit(['\\', '/']).next().unwrap_or(value);
            let lower = file.to_ascii_lowercase();
            let archive =
                (lower.ends_with(".a") || lower.ends_with(".lib")).then(|| file.to_owned());
            Some((number, archive))
        })
        .collect()
}

/// The ADR-0033 key for a row: `lib.a(member.o)` for a library member, the object's name otherwise, and
/// `None` for the linker's own rows (`<Block>`, `<for P2-1>`, `- Linker created -`).
fn object_key(row: &Row, index: &[(usize, Option<String>)]) -> Option<String> {
    let object = row.object.trim();
    if object.is_empty() || object.starts_with('<') || object.starts_with('-') {
        return None;
    }
    let archive = row
        .index
        .and_then(|n| index.iter().find(|(i, _)| *i == n))
        .and_then(|(_, archive)| archive.clone());
    Some(match archive {
        Some(archive) => format!("{archive}({object})"),
        None => object.to_owned(),
    })
}

/// `0x800'0194` or `0x08000194`: ILINK 8 and later group hex digits with `'`.
fn hex(token: &str) -> Option<u64> {
    let digits = token.strip_prefix("0x")?;
    let digits: String = digits.chars().filter(|c| *c != '\'').collect();
    if digits.is_empty() {
        return None;
    }
    u64::from_str_radix(&digits, 16).ok()
}

fn parse_failed(line: usize, detail: String) -> ArtifactError {
    ArtifactError::MapParseFailed { line, detail }
}

#[cfg(test)]
mod tests {
    use super::*;
    use firmwaresight_core::domain::memory::RegionKind;

    /// A trimmed ILINK 9 map in the layout IAR Embedded Workbench for Arm writes: two ranges, an
    /// application object, a library member, linker rows, and a copy-initialized RW block.
    const MAP: &str = "\
###############################################################################
#
# IAR ELF Linker V9.10.1.308/W64 for ARM                  19/Aug/2022  18:34:28
# Copyright 2007-2021 IAR Systems AB.
#
###############################################################################

*******************************************************************************
*** PLACEMENT SUMMARY
***

\"A0\":  place at address 0x800'0000 { ro section .intvec };
\"P1\":  place in [from 0x800'0000 to 0x801'ffff] { ro };
define block CSTACK with size = 1K, alignment = 8 { };
\"P2\":  place in [from 0x2000'0000 to 0x2000'4fff] {
          rw, block CSTACK };
initialize by copy { rw };

  Section            Kind         Address    Size  Object
  -------            ----         -------    ----  ------
\"A0\":                                       0x10c
  .intvec            ro code   0x800'0000   0x10c  startup_stm32f103xb.o [1]
                             - 0x800'010c   0x10c

\"P1\":                                      0x1508
  .text              ro code   0x800'010c   0x6a8  main.o [1]
  .text              ro code   0x800'07b4    0x36  strlen.o [3]
  .rodata            const     0x800'07ea     0x0  zero_init3.o [3]
  .iar.init_table    const     0x800'07ec    0x14  - Linker created -
  Initializer bytes  const     0x800'0800     0x8  <for P2-1>
                             - 0x800'0808   0x6fc

\"P2\", part 1 of 2:                           0x8
  P2-1                        0x2000'0000     0x8  <Init block>
    .data            inited   0x2000'0000     0x8  main.o [1]
                            - 0x2000'0008     0x8

\"P2\", part 2 of 2:                         0x400
  .bss               zero     0x2000'0008    0x20  ws2812b.o [1]
  CSTACK                      0x2000'0028   0x400  <Block>
    CSTACK           uninit   0x2000'0028   0x400  <Block tail>
                            - 0x2000'0428   0x420

Unused ranges:

*******************************************************************************
*** ENTRY LIST
***

[1] = C:\\Work\\WS2812B\\EWARM\\WS2812B\\Obj
[2] = command line
[3] = rt7M_tl.a
";

    #[test]
    fn regions_come_from_the_place_in_ranges() {
        let evidence = parse(MAP).expect("parses");
        assert_eq!(evidence.adapter_id, "iar_ilink");
        let regions: Vec<_> = evidence
            .regions
            .iter()
            .map(|r| (r.name.as_str(), r.origin, r.length, r.kind))
            .collect();
        assert_eq!(
            regions,
            vec![
                ("P1", 0x0800_0000, 0x2_0000, RegionKind::Nonvolatile),
                ("P2", 0x2000_0000, 0x5000, RegionKind::VolatileRam),
            ]
        );
        assert!(
            evidence.placements.is_empty(),
            "the ELF already carries load addresses"
        );
    }

    #[test]
    fn object_rows_name_library_members_and_skip_the_linkers_own_bytes() {
        let evidence = parse(MAP).expect("parses");
        let rows: Vec<_> = evidence
            .object_contributions
            .iter()
            .map(|c| (c.object.as_str(), c.input_section.as_str(), c.size))
            .collect();
        assert_eq!(
            rows,
            vec![
                ("startup_stm32f103xb.o", ".intvec", 0x10c),
                ("main.o", ".text", 0x6a8),
                ("rt7M_tl.a(strlen.o)", ".text", 0x36),
                ("main.o", ".data", 0x8),
                ("ws2812b.o", ".bss", 0x20),
            ],
            "no zero-size row, no initializer bytes, no linker-created table, no block, no host path"
        );
        assert!(
            evidence
                .object_contributions
                .iter()
                .all(|c| c.output_section.is_none())
        );
        let member = &evidence.object_contributions[2];
        assert_eq!(member.archive.as_deref(), Some("rt7M_tl.a"));
        assert_eq!(member.member, "strlen.o");
    }

    #[test]
    fn ilink_7_addresses_without_digit_grouping_read_the_same() {
        let old = MAP
            .replace("0x800'0000 to 0x801'ffff", "0x08000000 to 0x0801ffff")
            .replace("0x2000'0000 to 0x2000'4fff", "0x20000000 to 0x20004fff")
            .replace("0x800'010c   0x6a8", "0x0800010c   0x6a8");
        let evidence = parse(&old).expect("parses");
        assert_eq!(evidence.regions[0].origin, 0x0800_0000);
        assert_eq!(evidence.regions[1].length, 0x5000);
        assert_eq!(evidence.object_contributions[1].address, 0x0800_010c);
    }

    #[test]
    fn two_directives_over_one_range_are_one_region() {
        let shared = MAP.replace(
            "define block CSTACK",
            "\"P3\":  place in [from 0x800'0000 to 0x801'ffff] { section .cfg };\ndefine block CSTACK",
        );
        let evidence = parse(&shared).expect("parses");
        assert_eq!(evidence.regions.len(), 2);
        assert_eq!(evidence.regions[0].name, "P1+P3");
    }

    #[test]
    fn a_size_range_and_an_empty_range_are_read_honestly() {
        let sized = MAP.replace(
            "define block CSTACK",
            "\"P3\":  place in [from 0x1000'0000 size 0x1'0000] { section .ccmram };\ndefine block CSTACK",
        );
        let evidence = parse(&sized).expect("parses");
        let ccm = evidence
            .regions
            .iter()
            .find(|r| r.name == "P3")
            .expect("P3");
        assert_eq!((ccm.origin, ccm.length), (0x1000_0000, 0x1_0000));
        assert_eq!(
            ccm.kind,
            RegionKind::Unknown,
            "nothing placed there proves its kind"
        );
    }

    #[test]
    fn other_iar_targets_and_maps_without_a_placement_summary_are_refused() {
        let stm8 = MAP.replace("for ARM", "for STM8");
        assert!(matches!(
            parse(&stm8),
            Err(ArtifactError::MapUnsupported { detected }) if detected.contains("STM8")
        ));
        let no_summary = MAP.replace("*** PLACEMENT SUMMARY", "*** SOMETHING ELSE");
        assert!(!detect(&no_summary));
        assert!(mentions_iar(&no_summary));
        assert!(matches!(
            parse(&no_summary),
            Err(ArtifactError::MapUnsupported { .. })
        ));
    }

    #[test]
    fn a_map_without_any_range_or_with_a_broken_range_is_a_parse_error() {
        let no_range = MAP
            .replace(
                "\"P1\":  place in [from 0x800'0000 to 0x801'ffff] { ro };",
                "",
            )
            .replace(
                "\"P2\":  place in [from 0x2000'0000 to 0x2000'4fff] {",
                "\"P2\": place at end {",
            );
        assert!(matches!(
            parse(&no_range),
            Err(ArtifactError::MapParseFailed { .. })
        ));
        let backwards = MAP.replace("0x800'0000 to 0x801'ffff", "0x801'ffff to 0x800'0000");
        assert!(matches!(
            parse(&backwards),
            Err(ArtifactError::MapParseFailed { .. })
        ));
        let garbage = MAP.replace("0x800'0000 to 0x801'ffff", "0xzz to 0x801'ffff");
        assert!(matches!(
            parse(&garbage),
            Err(ArtifactError::MapParseFailed { .. })
        ));
    }

    #[test]
    fn rows_take_the_elf_section_that_holds_their_address() {
        use firmwaresight_core::domain::identity::Fact;
        use firmwaresight_core::domain::section::{Section, SectionFlags, SectionRole};
        let section = |name: &str, address: u64, size: u64| Section {
            index: 1,
            name: Fact::known(name.to_owned()),
            role: SectionRole::Code,
            flags: SectionFlags {
                alloc: true,
                ..SectionFlags::default()
            },
            virtual_address: Fact::known(address),
            load_address: Fact::unknown("test"),
            file_offset: Fact::unknown("test"),
            file_size: size,
            memory_size: Fact::known(size),
            region: Fact::unknown("test"),
        };
        let mut evidence = parse(MAP).expect("parses");
        let sections = vec![
            section("A0", 0x0800_0000, 0x10c),
            section("P1 ro", 0x0800_010c, 0x6fc),
            section("P2 rw", 0x2000_0000, 0x8),
            section("P2 zi", 0x2000_0008, 0x20),
        ];
        resolve_output_sections(&mut evidence.object_contributions, &sections);
        let named: Vec<_> = evidence
            .object_contributions
            .iter()
            .map(|c| c.output_section.as_deref())
            .collect();
        assert_eq!(
            named,
            vec![
                Some("A0"),
                Some("P1 ro"),
                Some("P1 ro"),
                Some("P2 rw"),
                Some("P2 zi")
            ]
        );
    }

    #[test]
    fn nested_and_overlapping_ranges_are_one_region_and_disjoint_repeats_are_numbered() {
        let nested = MAP.replace(
            "define block CSTACK",
            "\"P3\":  place in [from 0x801'0000 to 0x801'ffff] { section .cfg };\ndefine block CSTACK",
        );
        let evidence = parse(&nested).expect("parses");
        assert_eq!(
            evidence.regions.len(),
            2,
            "P3 sits inside P1: one region, one capacity"
        );
        assert_eq!(evidence.regions[0].name, "P1+P3");
        assert_eq!(evidence.regions[0].length, 0x2_0000);

        let split = MAP.replace(
            "[from 0x2000'0000 to 0x2000'4fff]",
            "[from 0x2000'0000 to 0x2000'4fff] | [from 0x1000'0000 to 0x1000'ffff]",
        );
        let evidence = parse(&split).expect("parses");
        let names: Vec<_> = evidence.regions.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["P1", "P2", "P2 (2)"]);
    }

    #[test]
    fn a_directive_without_its_semicolon_still_counts_and_a_missing_table_is_an_error() {
        let open = MAP.replace(
            "initialize by copy { rw };",
            "initialize by copy { rw };\n\"P3\":  place in [from 0x1000'0000 to 0x1000'ffff] { section .ccm }",
        );
        let evidence = parse(&open).expect("parses");
        assert!(evidence.regions.iter().any(|r| r.name == "P3"));

        let no_table = MAP.replace(
            "  Section            Kind         Address    Size  Object",
            "",
        );
        assert!(matches!(
            parse(&no_table),
            Err(ArtifactError::MapParseFailed { .. })
        ));
    }

    #[test]
    fn rows_outside_every_elf_section_or_overrunning_one_stay_unnamed() {
        use firmwaresight_core::domain::identity::Fact;
        use firmwaresight_core::domain::section::{Section, SectionFlags, SectionRole};
        let mut evidence = parse(MAP).expect("parses");
        let short = Section {
            index: 1,
            name: Fact::known("P1 ro".to_owned()),
            role: SectionRole::Code,
            flags: SectionFlags {
                alloc: true,
                ..SectionFlags::default()
            },
            virtual_address: Fact::known(0x0800_010c),
            load_address: Fact::unknown("test"),
            file_offset: Fact::unknown("test"),
            file_size: 0x10,
            memory_size: Fact::known(0x10),
            region: Fact::unknown("test"),
        };
        resolve_output_sections(&mut evidence.object_contributions, &[short]);
        assert!(
            evidence
                .object_contributions
                .iter()
                .all(|c| c.output_section.is_none()),
            "main.o's 0x6a8 bytes do not fit in a 0x10-byte section, and nothing else is held"
        );
    }

    #[test]
    fn hostile_text_never_panics() {
        for text in [
            "IAR ELF Linker for ARM\n*** PLACEMENT SUMMARY\n\"P1\": place in [from",
            "IAR ELF Linker for ARM\n*** PLACEMENT SUMMARY\n\"P1\": place in [from 0xffffffffffffffff size 0x2];",
            "IAR ELF Linker for ARM\n*** PLACEMENT SUMMARY\n  Section Kind Address Size Object\n  0x1 0x2\n[x] = y",
            "IAR ELF Linker for ARM\n*** PLACEMENT SUMMARY\n\"P1\": place in [from 0x0 to 0xffffffffffffffff];",
            "IAR ELF Linker for ARM\n*** PLACEMENT SUMMARY\n\"P\u{e9}\": place in [from 0x1 to 0x2] { ro };\n  Section Kind Address Size Object\n  .t\u{eb}xt ro code 0x1 0x1 m\u{e4}in.o [1]\n[1] = \u{f1}.a",
        ] {
            // Each one is refused or read; none panics, and a read never yields an empty region table.
            if let Ok(evidence) = parse(text) {
                assert!(!evidence.regions.is_empty());
            }
        }
    }
}
