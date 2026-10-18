//! Object and library attribution (ADR-0033): which object file or library member the image's bytes
//! came from, as the linker MAP states it.
//!
//! A row is never charged by guesswork. It is charged exactly like the output section it sits under in
//! [`MemoryFootprint`] (ADR-0021), so the per-object figures and the two budgets cannot disagree about
//! what counts: a row in `.data` costs Flash and RAM, a row in `.bss` costs RAM only, a row in a section
//! the footprint does not charge (debug, comment) costs neither, and a row whose section charge is
//! Unknown is Unknown, never zero (ADR-0033 D3).

use super::memory::MemoryFootprint;

/// One input-section row attributed to an object (ADR-0033 D1, D2). `object` is the normalized key,
/// never a host path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectContribution {
    pub object: String,
    pub archive: Option<String>,
    pub member: String,
    pub input_section: String,
    pub output_section: Option<String>,
    pub address: u64,
    pub size: u64,
    /// The MAP line the row was read from (evidence locator).
    pub map_line: u64,
}

/// What one row costs each budget. `None` is Unknown, and is kept apart from `Some(0)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Charge {
    pub nonvolatile: Option<u64>,
    pub runtime_ram: Option<u64>,
}

/// One object's (or one library's) total, in the order the summary ranks them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectUsage {
    /// The object key, or the archive file name for a library row.
    pub key: String,
    /// The library an object came from; `None` for an application object and for library rows.
    pub archive: Option<String>,
    pub nonvolatile: Option<u64>,
    pub runtime_ram: Option<u64>,
    /// How many MAP rows the total is built from.
    pub rows: usize,
}

/// The per-object and per-library view of one build, with what the rows do not cover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributionSummary {
    pub objects: Vec<ObjectUsage>,
    /// Archives only; application objects are not grouped under a made-up library.
    pub libraries: Vec<ObjectUsage>,
    /// Footprint bytes no object row accounts for (padding, linker stubs, sections with no rows).
    /// `None` when either side of the subtraction is not exact.
    pub unattributed_nonvolatile: Option<u64>,
    pub unattributed_runtime_ram: Option<u64>,
}

fn side(section_charge: Option<Option<u64>>, size: u64) -> Option<u64> {
    match section_charge {
        // The section is not charged at all (non-allocatable, debug): the row costs nothing.
        None => Some(0),
        Some(None) => None,
        Some(Some(0)) => Some(0),
        Some(Some(_)) => Some(size),
    }
}

/// What `row` costs, read from the charge of its output section in `footprint`.
#[must_use]
pub fn charge(row: &ObjectContribution, footprint: &MemoryFootprint) -> Charge {
    let Some(output) = row.output_section.as_deref() else {
        return Charge {
            nonvolatile: None,
            runtime_ram: None,
        };
    };
    // One name is one section for GNU ld. armlink can give two sections one name (the RW data and ZI parts
    // of a region, ADR-0036): the row's address then chooses, and a row no candidate holds is Unknown.
    let named: Vec<_> = footprint
        .contributions
        .iter()
        .filter(|c| c.section_name.value().map(String::as_str) == Some(output))
        .collect();
    let section = if named.len() > 1 {
        let holding = named.iter().copied().find(|c| {
            c.runtime_span
                .is_some_and(|(start, size)| row.address >= start && row.address - start < size)
        });
        let Some(holding) = holding else {
            return Charge {
                nonvolatile: None,
                runtime_ram: None,
            };
        };
        Some(holding)
    } else {
        named.first().copied()
    };
    let nonvolatile = section.map(|c| c.nonvolatile_bytes.value().copied());
    let runtime_ram = section.map(|c| c.runtime_ram_bytes.value().copied());
    // A section charged to both budgets with two different sizes stores something other than its rows in
    // Flash: armlink's compressed RW data (ADR-0036 A1). Each row's stored share is unknowable, so its Flash
    // charge is Unknown; its RAM charge is still its size.
    let stored_differently = matches!(
        (nonvolatile, runtime_ram),
        (Some(Some(stored)), Some(Some(runtime))) if stored > 0 && runtime > 0 && stored != runtime
    );
    Charge {
        nonvolatile: if stored_differently {
            None
        } else {
            side(nonvolatile, row.size)
        },
        runtime_ram: side(runtime_ram, row.size),
    }
}

fn add(total: Option<u64>, value: Option<u64>) -> Option<u64> {
    total.zip(value).map(|(a, b)| a.saturating_add(b))
}

fn rank(usages: &mut [ObjectUsage]) {
    // Largest Flash first, Unknown after every known figure, then by key so the order is total.
    usages.sort_by(|a, b| {
        let key = |u: &ObjectUsage| {
            (
                u.nonvolatile.is_none(),
                std::cmp::Reverse(u.nonvolatile.unwrap_or(0)),
            )
        };
        key(a).cmp(&key(b)).then_with(|| a.key.cmp(&b.key))
    });
}

/// Each row with what it costs, in MAP order. Without a footprint every charge is Unknown: the rows
/// are still evidence of where bytes came from, but not of what they cost.
#[must_use]
pub fn charged(
    rows: &[ObjectContribution],
    footprint: Option<&MemoryFootprint>,
) -> Vec<(ObjectContribution, Charge)> {
    rows.iter()
        .map(|row| {
            let cost = footprint.map_or(
                Charge {
                    nonvolatile: None,
                    runtime_ram: None,
                },
                |footprint| charge(row, footprint),
            );
            (row.clone(), cost)
        })
        .collect()
}

fn exact_total(total: &super::memory::ByteTotal) -> Option<u64> {
    if total.is_exact() {
        total.bytes()
    } else {
        None
    }
}

/// Aggregate rows per object and per library, and state the remainder the rows leave uncovered.
#[must_use]
pub fn summarize(rows: &[ObjectContribution], footprint: &MemoryFootprint) -> AttributionSummary {
    aggregate(
        &charged(rows, Some(footprint)),
        exact_total(&footprint.nonvolatile),
        exact_total(&footprint.runtime_ram),
    )
}

fn accumulate(usages: &mut Vec<ObjectUsage>, key: &str, archive: Option<&String>, cost: Charge) {
    let entry = match usages.iter().position(|u| u.key == key) {
        Some(index) => &mut usages[index],
        None => {
            usages.push(ObjectUsage {
                key: key.to_owned(),
                archive: archive.cloned(),
                nonvolatile: Some(0),
                runtime_ram: Some(0),
                rows: 0,
            });
            usages.last_mut().expect("just pushed")
        }
    };
    entry.nonvolatile = add(entry.nonvolatile, cost.nonvolatile);
    entry.runtime_ram = add(entry.runtime_ram, cost.runtime_ram);
    entry.rows += 1;
}

/// [`summarize`] for rows whose charges were already decided, as storage keeps them. The totals are
/// the footprint budgets, passed only when they are exact; otherwise no remainder is claimed.
#[must_use]
pub fn aggregate(
    rows: &[(ObjectContribution, Charge)],
    exact_nonvolatile: Option<u64>,
    exact_runtime_ram: Option<u64>,
) -> AttributionSummary {
    let mut objects: Vec<ObjectUsage> = Vec::new();
    let mut libraries: Vec<ObjectUsage> = Vec::new();
    let mut attributed = Charge {
        nonvolatile: Some(0),
        runtime_ram: Some(0),
    };

    for (row, cost) in rows {
        attributed.nonvolatile = add(attributed.nonvolatile, cost.nonvolatile);
        attributed.runtime_ram = add(attributed.runtime_ram, cost.runtime_ram);
        accumulate(&mut objects, &row.object, row.archive.as_ref(), *cost);
        if let Some(archive) = &row.archive {
            accumulate(&mut libraries, archive, None, *cost);
        }
    }
    rank(&mut objects);
    rank(&mut libraries);

    let remainder = |total: Option<u64>, covered: Option<u64>| match (total, covered) {
        (Some(total), Some(covered)) if covered <= total => Some(total - covered),
        _ => None,
    };
    AttributionSummary {
        objects,
        libraries,
        unattributed_nonvolatile: remainder(exact_nonvolatile, attributed.nonvolatile),
        unattributed_runtime_ram: remainder(exact_runtime_ram, attributed.runtime_ram),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::identity::Fact;
    use crate::domain::memory::{
        ByteTotal, LayoutSource, MemoryContribution, MemoryEvidenceBasis, MemoryFootprint,
    };

    fn section(name: &str, nonvolatile: Fact<u64>, ram: Fact<u64>) -> MemoryContribution {
        MemoryContribution {
            section_index: 1,
            section_locator: format!("section:1({name})"),
            section_name: Fact::known(name.to_owned()),
            nonvolatile_bytes: nonvolatile,
            runtime_ram_bytes: ram,
            basis: MemoryEvidenceBasis::MapRegionAndElfLoad,
            rule: "test",
            runtime_span: None,
        }
    }

    #[test]
    fn rows_of_a_compressed_section_have_unknown_flash_and_known_ram() {
        // armlink compressed 2572 bytes of RW data into 428 stored bytes: no row's stored share is known.
        let footprint = footprint(
            vec![section("RW_RAM", Fact::known(428), Fact::known(2572))],
            428,
            2572,
        );
        let row = ObjectContribution {
            object: "arm_fir_data.o".to_owned(),
            archive: None,
            member: "arm_fir_data.o".to_owned(),
            input_section: ".data".to_owned(),
            output_section: Some("RW_RAM".to_owned()),
            address: 0x2000_0000,
            size: 2560,
            map_line: 1,
        };
        let cost = charge(&row, &footprint);
        assert_eq!((cost.nonvolatile, cost.runtime_ram), (None, Some(2560)));
        // An ordinary data section stores what it runs: both charges stay the row's size.
        let plain = footprint_plain();
        let cost = charge(
            &ObjectContribution {
                output_section: Some(".data".to_owned()),
                ..row
            },
            &plain,
        );
        assert_eq!(
            (cost.nonvolatile, cost.runtime_ram),
            (Some(2560), Some(2560))
        );
    }

    fn footprint_plain() -> MemoryFootprint {
        footprint(
            vec![section(".data", Fact::known(4096), Fact::known(4096))],
            4096,
            4096,
        )
    }

    #[test]
    fn two_output_sections_of_one_name_are_told_apart_by_the_row_address() {
        // armlink: RW_IRAM1 is both the 4-byte RW data (Flash and RAM) and the 40-byte ZI part (RAM only).
        let mut data = section("RW_IRAM1", Fact::known(4), Fact::known(4));
        data.runtime_span = Some((0x2000_0000, 4));
        let mut zi = section("RW_IRAM1", Fact::known(0), Fact::known(40));
        zi.runtime_span = Some((0x2000_0004, 40));
        let footprint = footprint(vec![data, zi], 4, 44);
        let mut row = ObjectContribution {
            object: "libdrv.lib(spi.o)".to_owned(),
            archive: Some("libdrv.lib".to_owned()),
            member: "spi.o".to_owned(),
            input_section: ".bss.spi_buffer".to_owned(),
            output_section: Some("RW_IRAM1".to_owned()),
            address: 0x2000_0004,
            size: 32,
            map_line: 1,
        };
        let zi_charge = charge(&row, &footprint);
        assert_eq!(
            (zi_charge.nonvolatile, zi_charge.runtime_ram),
            (Some(0), Some(32)),
            "ZI costs RAM only"
        );
        row.address = 0x2000_0000;
        row.size = 4;
        let data_charge = charge(&row, &footprint);
        assert_eq!(
            (data_charge.nonvolatile, data_charge.runtime_ram),
            (Some(4), Some(4)),
            "RW data costs both"
        );
        row.address = 0x3000_0000;
        let nowhere = charge(&row, &footprint);
        assert_eq!(
            (nowhere.nonvolatile, nowhere.runtime_ram),
            (None, None),
            "no candidate holds it: Unknown, not a guess"
        );
    }

    fn footprint(sections: Vec<MemoryContribution>, nonvolatile: u64, ram: u64) -> MemoryFootprint {
        MemoryFootprint {
            nonvolatile: ByteTotal::Exact { bytes: nonvolatile },
            runtime_ram: ByteTotal::Exact { bytes: ram },
            contributions: sections,
            excluded_metadata_bytes: 0,
            weakest_basis: Some(MemoryEvidenceBasis::MapRegionAndElfLoad),
            layout_source: LayoutSource::MapMemoryConfiguration,
            region_usage: Vec::new(),
        }
    }

    fn row(object: &str, archive: Option<&str>, output: &str, size: u64) -> ObjectContribution {
        ObjectContribution {
            object: object.to_owned(),
            archive: archive.map(str::to_owned),
            member: object.to_owned(),
            input_section: format!("{output}.x"),
            output_section: Some(output.to_owned()),
            address: 0,
            size,
            map_line: 1,
        }
    }

    fn sample() -> MemoryFootprint {
        footprint(
            vec![
                section(".text", Fact::known(0x100), Fact::known(0)),
                section(".data", Fact::known(0x10), Fact::known(0x10)),
                section(".bss", Fact::known(0), Fact::known(0x40)),
            ],
            0x110,
            0x50,
        )
    }

    #[test]
    fn rows_are_charged_like_their_output_section() {
        let fp = sample();
        assert_eq!(
            charge(&row("a.o", None, ".text", 8), &fp),
            Charge {
                nonvolatile: Some(8),
                runtime_ram: Some(0)
            }
        );
        assert_eq!(
            charge(&row("a.o", None, ".data", 4), &fp),
            Charge {
                nonvolatile: Some(4),
                runtime_ram: Some(4)
            }
        );
        assert_eq!(
            charge(&row("a.o", None, ".bss", 4), &fp),
            Charge {
                nonvolatile: Some(0),
                runtime_ram: Some(4)
            }
        );
        assert_eq!(
            charge(&row("a.o", None, ".debug_info", 99), &fp),
            Charge {
                nonvolatile: Some(0),
                runtime_ram: Some(0)
            }
        );
    }

    #[test]
    fn an_unknown_section_charge_stays_unknown() {
        let fp = footprint(
            vec![section(
                ".text",
                Fact::unknown("no load evidence"),
                Fact::known(0),
            )],
            0,
            0,
        );
        let cost = charge(&row("a.o", None, ".text", 8), &fp);
        assert_eq!(
            cost,
            Charge {
                nonvolatile: None,
                runtime_ram: Some(0)
            }
        );
        let mut no_section = row("a.o", None, ".text", 8);
        no_section.output_section = None;
        assert_eq!(
            charge(&no_section, &fp),
            Charge {
                nonvolatile: None,
                runtime_ram: None
            }
        );
    }

    #[test]
    fn objects_and_libraries_aggregate_rank_and_leave_an_explicit_remainder() {
        let rows = vec![
            row("main.o", None, ".text", 0x20),
            row("libdrv.a(uart.o)", Some("libdrv.a"), ".text", 0x30),
            row("libdrv.a(spi.o)", Some("libdrv.a"), ".text", 0x40),
            row("main.o", None, ".data", 0x10),
            row("libdrv.a(spi.o)", Some("libdrv.a"), ".bss", 0x20),
        ];
        let summary = summarize(&rows, &sample());
        let keys: Vec<&str> = summary.objects.iter().map(|u| u.key.as_str()).collect();
        assert_eq!(keys, ["libdrv.a(spi.o)", "libdrv.a(uart.o)", "main.o"]);
        assert_eq!(summary.objects[2].nonvolatile, Some(0x30));
        assert_eq!(summary.objects[2].runtime_ram, Some(0x10));
        assert_eq!(summary.objects[2].rows, 2);
        assert_eq!(summary.libraries.len(), 1);
        assert_eq!(summary.libraries[0].key, "libdrv.a");
        assert_eq!(summary.libraries[0].nonvolatile, Some(0x70));
        assert_eq!(summary.unattributed_nonvolatile, Some(0x110 - 0xa0));
        assert_eq!(summary.unattributed_runtime_ram, Some(0x50 - 0x30));
    }

    #[test]
    fn an_unknown_row_makes_its_object_and_the_remainder_unknown_not_smaller() {
        let fp = footprint(
            vec![
                section(".text", Fact::known(0x10), Fact::known(0)),
                section(".odd", Fact::unknown("x"), Fact::known(0)),
            ],
            0x10,
            0,
        );
        let summary = summarize(
            &[
                row("a.o", None, ".text", 8),
                row("a.o", None, ".odd", 4),
                row("b.o", None, ".text", 8),
            ],
            &fp,
        );
        assert_eq!(
            summary
                .objects
                .last()
                .map(|u| (u.key.as_str(), u.nonvolatile)),
            Some(("a.o", None))
        );
        assert_eq!(summary.unattributed_nonvolatile, None);
    }
}
