//! JSON rendering and the determinism checks that make "deterministic output" testable rather
//! than aspirational.

use serde::Serialize;

use firmwaresight_core::domain::memory::{MemoryRegionUsage, RegionUsageRisk, RegionUsageState};

use crate::dto::AnalyzeResultDto;
use firmwaresight_core::domain::attribution::{AttributionSummary, ObjectUsage};

/// Compact machine-readable form.
///
/// Generic over the DTO on purpose: the analyze payload and the diff payload must go through the
/// same serializer, or "deterministic" would have two definitions.
pub fn to_json<T: Serialize + ?Sized>(value: &T) -> String {
    // Serializing a DTO cannot fail: every field is a plain serializable type and no path
    // collector or float is involved.
    serde_json::to_string(value).unwrap_or_else(infallible)
}

/// Human-readable JSON, used for goldens so a semantic diff stays reviewable by eye.
pub fn to_json_pretty<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(infallible)
}

fn infallible(err: serde_json::Error) -> String {
    panic!("DTO serialization must not fail: {err}")
}

/// Re-serialize JSON with object keys in sorted order.
///
/// Two outputs that differ only in key order collapse to the same canonical text, which lets a
/// test assert semantic equality without depending on field declaration order.
#[must_use]
pub fn canonical_form(json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .map(|value| serde_json::to_string(&value).unwrap_or_else(infallible))
}

#[must_use]
pub fn semantically_equal(left: &str, right: &str) -> bool {
    match (canonical_form(left), canonical_form(right)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// The `--json` contract: exactly one JSON document on stdout, nothing else.
#[must_use]
pub fn render_json(value: &AnalyzeResultDto) -> String {
    document_json(value)
}

/// The one form a portable document takes on disk or on a stream: compact JSON, one trailing newline.
///
/// Every bundle file shares this spelling, because `SHA256SUMS` and the manifest hash these bytes and a
/// second definition of "the document" would make a bundle half-verifiable. The newline is part of the
/// payload, so it is written here rather than by whichever caller happens to add it.
#[must_use]
pub fn document_json<T: Serialize + ?Sized>(value: &T) -> String {
    let mut out = to_json(value);
    out.push('\n');
    out
}

/// The default human-readable rendering. Diagnostics never go here; they go to stderr.
pub fn render_human(value: &AnalyzeResultDto) -> String {
    render_human_inner(value, None)
}

/// `fwsight analyze --objects`: the per-object and per-library table of one build (ADR-0033).
///
/// The figures are Core's: each MAP row is charged like its output section, and the ranking is
/// [`AttributionSummary`]'s own. An Unknown charge is printed as `unknown`, never `0`, and a build
/// without object rows says so instead of printing an empty table. Every key is escaped, because a
/// MAP is user input and its names reach a terminal (the same rule as the region names).
#[must_use]
pub fn render_objects(summary: Option<&AttributionSummary>) -> String {
    let mut out = crate::text::heading(
        "Objects and libraries (GNU ld MAP rows, charged like their output section)",
    );
    let Some(summary) = summary else {
        out.push_str(
            "  unavailable: this build has no GNU ld MAP object rows; pass --map FILE with the linker's .map\n",
        );
        return out;
    };
    let bytes = |value: Option<u64>| {
        value.map_or_else(|| "unknown".to_owned(), |bytes| format!("{bytes} bytes"))
    };
    let table = |out: &mut String, title: &str, rows: &[ObjectUsage]| {
        out.push_str(&format!("  {title} ({})\n", rows.len()));
        out.push_str(&format!(
            "    {:>14}  {:>14}  {:>5}  Name\n",
            "Flash", "RAM", "Rows"
        ));
        for row in rows {
            out.push_str(&format!(
                "    {:>14}  {:>14}  {:>5}  {}\n",
                bytes(row.nonvolatile),
                bytes(row.runtime_ram),
                row.rows,
                safe_display_text(&row.key)
            ));
        }
    };
    table(&mut out, "Objects", &summary.objects);
    if summary.libraries.is_empty() {
        out.push_str("  Libraries (0): no row came from a static library\n");
    } else {
        table(&mut out, "Libraries", &summary.libraries);
    }
    // Each side on its own: an exact Flash remainder stays visible when the RAM one is Unknown.
    out.push_str(&format!(
        "  Not attributed to an object: {} Flash, {} RAM (padding, linker stubs, sections without object rows; unknown where a budget total or an object charge is not exact)\n",
        bytes(summary.unattributed_nonvolatile),
        bytes(summary.unattributed_runtime_ram)
    ));
    out
}

/// Human CLI rendering with Core-computed region evidence. The portable `analysis:1` document is
/// intentionally unchanged; this is a presentation projection over the same immutable analysis.
#[must_use]
pub fn render_human_with_region_usage(
    value: &AnalyzeResultDto,
    regions: &[MemoryRegionUsage],
) -> String {
    render_human_inner(value, Some(regions))
}

fn render_human_inner(value: &AnalyzeResultDto, regions: Option<&[MemoryRegionUsage]>) -> String {
    use crate::text::{heading, masthead, wrap};

    // R6 report grammar: masthead, the answer, then the facts by section.
    let mut out = masthead("Analysis report");
    out.push_str(&wrap(
        &format!(
            "{}: nonvolatile / load image {}; runtime RAM {}.",
            safe_display_text(&value.artifact.file_name),
            budget_line(&value.memory.nonvolatile_image_footprint),
            budget_line(&value.memory.runtime_ram_footprint)
        ),
        "",
        "  ",
    ));
    out.push_str(&heading("Build"));
    out.push_str(&format!(
        "  File         {}\n",
        safe_display_text(&value.artifact.file_name)
    ));
    out.push_str(&format!("  SHA-256      {}\n", value.artifact.sha256));
    out.push_str(&format!(
        "  Size         {} bytes\n",
        value.artifact.byte_size
    ));
    out.push_str(&format!(
        "  Format       {} {}-bit {}\n",
        value.artifact.architecture, value.artifact.bitness, value.artifact.endianness
    ));
    out.push_str(&format!(
        "  Entry        {}\n",
        value
            .artifact
            .entry_point
            .clone()
            .unwrap_or_else(|| format!(
                "unknown ({})",
                value
                    .artifact
                    .entry_point_unknown_reason
                    .clone()
                    .unwrap_or_default()
            ))
    ));

    out.push_str(&heading("Memory"));
    out.push_str(&format!(
        "  Nonvolatile / load image: {}\n",
        budget_line(&value.memory.nonvolatile_image_footprint)
    ));
    out.push_str(&format!(
        "  Runtime RAM:              {}\n",
        budget_line(&value.memory.runtime_ram_footprint)
    ));
    out.push_str(&format!(
        "  Evidence basis:           {} (layout: {})\n",
        value
            .memory
            .weakest_evidence_basis
            .unwrap_or("no allocatable sections"),
        value.memory.layout_source
    ));
    if !value.memory.dual_accounted_sections.is_empty() {
        out.push_str(&format!(
            "  Dual-accounted sections:  {}\n",
            value
                .memory
                .dual_accounted_sections
                .iter()
                .map(|locator| safe_display_text(locator))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !value.memory.contributions.is_empty() {
        out.push_str("  Section contributions (nonvolatile / runtime RAM bytes; evidence basis)\n");
        for contribution in &value.memory.contributions {
            let name = contribution
                .section_name
                .as_deref()
                .map(safe_display_text)
                .unwrap_or_else(|| "unknown section".to_owned());
            out.push_str(&format!(
                "    {name} ({}): {} / {} bytes [{}; {}]\n",
                safe_display_text(&contribution.section_locator),
                contribution
                    .nonvolatile_bytes
                    .map_or_else(|| "unknown".to_owned(), |bytes| bytes.to_string()),
                contribution
                    .runtime_ram_bytes
                    .map_or_else(|| "unknown".to_owned(), |bytes| bytes.to_string()),
                contribution.classification,
                contribution.evidence_basis,
            ));
        }
    }
    if let Some(regions) = regions {
        render_memory_regions(&mut out, value.memory.layout_source, regions);
    }

    out.push_str(&heading("Counts"));
    out.push_str(&format!(
        "  Sections {}   Symbols {}   Evidence {}\n",
        value.counts.sections, value.counts.symbols, value.counts.evidence
    ));

    out.push_str(&heading("Capabilities"));
    for (label, field) in [
        ("ELF", value.capabilities.elf),
        ("Sections", value.capabilities.sections),
        ("Symbols", value.capabilities.symbols),
        ("Debug info", value.capabilities.debug_info),
        ("MAP", value.capabilities.map),
        ("Object attribution", value.capabilities.object_attribution),
        ("Git", value.capabilities.git),
    ] {
        out.push_str(&format!("  {label:<20} {field}\n"));
    }

    out.push_str(&heading("Limits"));
    out.push_str(&wrap(
        "Every figure comes from the ELF and, when given, the GNU ld MAP. A mapped section range is not free space, and BIN or HEX files are not analyzed.",
        "  ",
        "  ",
    ));
    out
}

fn render_memory_regions(out: &mut String, layout_source: &str, regions: &[MemoryRegionUsage]) {
    out.push_str(&crate::text::heading(
        "Memory regions (mapped section ranges; not contiguous free space)",
    ));
    if regions.is_empty() {
        out.push_str(&format!(
            "  Unknown: no supported memory-region table is available (layout: {layout_source})\n"
        ));
        return;
    }

    for region in regions {
        let kind = match region.kind {
            firmwaresight_core::domain::memory::RegionKind::Nonvolatile => {
                "nonvolatile / load image"
            }
            firmwaresight_core::domain::memory::RegionKind::VolatileRam => "runtime RAM",
            firmwaresight_core::domain::memory::RegionKind::Unknown => "unknown memory kind",
        };
        out.push_str(&format!(
            "  {} ({kind}) [{}: {}]\n",
            safe_display_text(&region.name),
            region.evidence_source,
            safe_display_text(&region.evidence_id)
        ));
        let address_range = region
            .capacity_bytes
            .and_then(|capacity| {
                region
                    .origin
                    .checked_add(capacity)
                    .map(|end| (capacity, end))
            })
            .map_or_else(
                || format!("{} .. unknown", address_text(region.origin)),
                |(_, end)| format!("{} .. {}", address_text(region.origin), address_text(end)),
            );
        out.push_str(&format!("    Address range: {address_range}\n"));
        match region.capacity_bytes {
            Some(capacity) => out.push_str(&format!("    Capacity: {capacity} bytes [declared]\n")),
            None => out.push_str("    Capacity: unknown\n"),
        }
        let used = match (region.used_bytes, region.state) {
            (Some(bytes), RegionUsageState::Exact) => format!("{bytes} bytes [exact]"),
            (Some(bytes), RegionUsageState::Partial) => format!("at least {bytes} bytes [partial]"),
            _ => "unknown".to_owned(),
        };
        out.push_str(&format!("    Mapped section ranges: {used}\n"));
        out.push_str(&format!(
            "    Remaining contiguous space: unknown ({})\n",
            region.remaining_reason
        ));
        let risk = match region.risk {
            RegionUsageRisk::WithinCapacity => {
                "no observed section range exceeds the declared limit; this is not a free-space claim"
            }
            RegionUsageRisk::Exceeded => "observed section range exceeds the declared capacity",
            RegionUsageRisk::Review => "review required; attribution is partial or ambiguous",
            RegionUsageRisk::Unknown => "unknown",
        };
        out.push_str(&format!("    Capacity check: {risk}\n"));
        if !region.sections.is_empty() {
            out.push_str("    Section evidence:\n");
            for section in &region.sections {
                let name = section.section_name.value().map_or_else(
                    || "unknown section".to_owned(),
                    |name| safe_display_text(name),
                );
                let vma = section
                    .virtual_address
                    .value()
                    .map_or_else(|| "unknown".to_owned(), |value| address_text(*value));
                let lma = section
                    .load_address
                    .value()
                    .map_or_else(|| "unknown".to_owned(), |value| address_text(*value));
                out.push_str(&format!(
                    "      {name} ({}) VMA {vma}; LMA {lma}: {} bytes [{}]\n",
                    safe_display_text(&section.section_locator),
                    section.bytes,
                    evidence_basis_text(section.evidence_basis)
                ));
            }
        }
        for reason in &region.reasons {
            out.push_str(&format!("    Review: {}\n", safe_display_text(reason)));
        }
    }
}

fn address_text(address: u64) -> String {
    format!("0x{address:08x}")
}

const fn evidence_basis_text(
    basis: firmwaresight_core::domain::memory::MemoryEvidenceBasis,
) -> &'static str {
    match basis {
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::RegionConfigAndElfLoad => {
            "region-config+elf-load"
        }
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::MapRegionAndElfLoad => {
            "map-memory-configuration+elf-load"
        }
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::ElfAddressAndFlags => {
            "elf-address-and-flags"
        }
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::SectionNameHeuristic => {
            "section-name-heuristic"
        }
        firmwaresight_core::domain::memory::MemoryEvidenceBasis::Insufficient => "unattributed",
    }
}

pub(crate) fn safe_display_text(value: &str) -> String {
    value.chars().flat_map(char::escape_debug).collect()
}

pub(crate) fn budget_line(budget: &crate::dto::BudgetDto) -> String {
    match (budget.state, budget.bytes) {
        ("exact", Some(bytes)) => format!("{bytes} bytes [{}]", budget.classification),
        ("partial", Some(bytes)) => format!(
            "{bytes} bytes or more, {} section(s) unattributed [{}]",
            budget.unattributed.len(),
            budget.classification
        ),
        _ => format!("unknown [{}]", budget.classification),
    }
}

/// Stable code for "the comparison is fine, the file the user asked for could not be written".
/// Shared by the CLI export path and the desktop save path so one failure has one code.
pub const EXPORT_FAILED_CODE: &str = "ERR-EXPORT-6001";

/// The stable error shape the desktop and CLI both present.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
    pub operation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

impl ErrorEnvelope {
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        operation_id: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            operation_id: operation_id.into(),
            details: None,
            remediation: None,
        }
    }

    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    #[must_use]
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(infallible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_form_ignores_object_key_order() {
        let left = r#"{"a":1,"b":{"x":1,"y":2}}"#;
        let right = r#"{"b":{"y":2,"x":1},"a":1}"#;

        assert!(semantically_equal(left, right));
    }

    #[test]
    fn canonical_form_still_detects_a_real_difference() {
        assert!(!semantically_equal(r#"{"a":1}"#, r#"{"a":2}"#));
        assert!(!semantically_equal(r#"{"a":1,"b":2}"#, r#"{"a":1}"#));
    }

    #[test]
    fn malformed_json_is_not_equal_to_anything() {
        assert!(!semantically_equal("not json", r#"{"a":1}"#));
        assert!(canonical_form("not json").is_none());
    }

    #[test]
    fn error_envelope_carries_code_operation_and_remediation() {
        let envelope =
            ErrorEnvelope::new("ERR-PARSE-2002", "Could not parse artifact as ELF.", "op-7")
                .with_remediation("Choose the linker ELF output.");

        let json = envelope.to_json();

        assert!(json.contains("\"code\":\"ERR-PARSE-2002\""));
        assert!(json.contains("\"operationId\":\"op-7\""));
        assert!(json.contains("linker ELF output"));
        // Absent optional fields must not appear as null noise.
        assert!(!json.contains("null"), "unexpected null in {json}");
    }

    #[test]
    fn the_object_table_escapes_map_names_and_keeps_unknown_apart_from_zero() {
        use firmwaresight_core::domain::attribution::{AttributionSummary, ObjectUsage};

        let summary = AttributionSummary {
            objects: vec![ObjectUsage {
                key: "evil\u{1b}[2J.o".to_owned(),
                archive: None,
                nonvolatile: Some(0),
                runtime_ram: None,
                rows: 1,
            }],
            libraries: Vec::new(),
            unattributed_nonvolatile: None,
            unattributed_runtime_ram: Some(4),
        };
        let text = super::render_objects(Some(&summary));
        assert!(
            !text.contains('\u{1b}'),
            "no raw escape reaches a terminal: {text:?}"
        );
        assert!(text.contains("evil\\u{1b}[2J.o"), "{text}");
        assert!(text.contains("0 bytes"), "a known zero is a zero");
        assert!(text.contains("unknown"), "an unknown charge is not a zero");
        assert!(
            text.contains("Not attributed to an object: unknown Flash, 4 bytes RAM"),
            "{text}"
        );
        assert!(text.contains("Libraries (0)"), "{text}");
    }
}
