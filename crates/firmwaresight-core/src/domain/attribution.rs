//! Object and library attribution (ADR-0033): which object file or library member the image's bytes
//! came from, as the GNU ld MAP states it.
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
    let section = footprint
        .contributions
        .iter()
        .find(|c| c.section_name.value().map(String::as_str) == Some(output));
    Charge {
        nonvolatile: side(
            section.map(|c| c.nonvolatile_bytes.value().copied()),
            row.size,
        ),
        runtime_ram: side(
            section.map(|c| c.runtime_ram_bytes.value().copied()),
            row.size,
        ),
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

/// Aggregate rows per object and per library, and state the remainder the rows leave uncovered.
#[must_use]
pub fn summarize(rows: &[ObjectContribution], footprint: &MemoryFootprint) -> AttributionSummary {
    let mut objects: Vec<ObjectUsage> = Vec::new();
    let mut libraries: Vec<ObjectUsage> = Vec::new();
    let mut attributed = Charge {
        nonvolatile: Some(0),
        runtime_ram: Some(0),
    };

    for row in rows {
        let cost = charge(row, footprint);
        attributed.nonvolatile = add(attributed.nonvolatile, cost.nonvolatile);
        attributed.runtime_ram = add(attributed.runtime_ram, cost.runtime_ram);

        let entry = match objects.iter_mut().find(|u| u.key == row.object) {
            Some(entry) => entry,
            None => {
                objects.push(ObjectUsage {
                    key: row.object.clone(),
                    archive: row.archive.clone(),
                    nonvolatile: Some(0),
                    runtime_ram: Some(0),
                    rows: 0,
                });
                objects.last_mut().expect("just pushed")
            }
        };
        entry.nonvolatile = add(entry.nonvolatile, cost.nonvolatile);
        entry.runtime_ram = add(entry.runtime_ram, cost.runtime_ram);
        entry.rows += 1;

        if let Some(archive) = &row.archive {
            let library = match libraries.iter_mut().find(|u| &u.key == archive) {
                Some(library) => library,
                None => {
                    libraries.push(ObjectUsage {
                        key: archive.clone(),
                        archive: None,
                        nonvolatile: Some(0),
                        runtime_ram: Some(0),
                        rows: 0,
                    });
                    libraries.last_mut().expect("just pushed")
                }
            };
            library.nonvolatile = add(library.nonvolatile, cost.nonvolatile);
            library.runtime_ram = add(library.runtime_ram, cost.runtime_ram);
            library.rows += 1;
        }
    }
    rank(&mut objects);
    rank(&mut libraries);

    let remainder = |total: Option<u64>, exact: bool, covered: Option<u64>| match (total, covered) {
        (Some(total), Some(covered)) if exact && covered <= total => Some(total - covered),
        _ => None,
    };
    AttributionSummary {
        objects,
        libraries,
        unattributed_nonvolatile: remainder(
            footprint.nonvolatile.bytes(),
            footprint.nonvolatile.is_exact(),
            attributed.nonvolatile,
        ),
        unattributed_runtime_ram: remainder(
            footprint.runtime_ram.bytes(),
            footprint.runtime_ram.is_exact(),
            attributed.runtime_ram,
        ),
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
        }
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
