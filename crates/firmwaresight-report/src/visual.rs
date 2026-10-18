//! Bounded, presentation-only exports. Portable v1 documents remain unchanged.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_display_rejects_embedded_paths_controls_and_markup() {
        for input in [
            "build C:\\Users\\private-canary\\x",
            "build /home/private-canary/x",
            "<script onload='x'>",
            "a\u{1b}[2J",
            "a\u{202e}b",
        ] {
            let safe = safe_label(input);
            assert!(!safe.contains("private-canary"));
            assert!(!safe.contains('<'));
            assert!(!safe.contains('\u{1b}'));
            assert!(!safe.contains('\u{202e}'));
        }
        assert_eq!(safe_label("固件 .text main"), "固件 .text main");
        assert!(safe_label(&"x".repeat(10000)).chars().count() <= 160);
    }

    #[test]
    fn unknown_never_becomes_zero_and_large_bytes_stay_exact() {
        assert_eq!(bytes_text(None), "Unknown");
        assert_eq!(bytes_text(Some(0)), "0 bytes");
        assert_eq!(bytes_text(Some(u64::MAX)), "18446744073709551615 bytes");
    }

    #[test]
    fn ordinary_explanations_remain_readable_but_paths_are_withheld() {
        let explanation = "No configured budget; rule is not applicable.";
        assert_eq!(safe_label(explanation), explanation);
        assert_eq!(
            safe_label("Build's evidence: unavailable"),
            "Build's evidence: unavailable"
        );
        for path in [
            "C:/Users/private-canary",
            "build C:private-canary",
            "build /private-canary",
            "\\\\server\\private-canary",
            "<svg onload=x>",
        ] {
            assert!(!safe_label(path).contains("private-canary"));
            assert!(!safe_label(path).contains("onload"));
        }
    }

    /// The sentences Core actually writes must survive, because a withheld reason is no reason.
    #[test]
    fn core_rule_prose_with_code_quotes_and_dashes_stays_readable() {
        for sentence in [
            "No `release.expected_commit` is declared for this release.",
            "No `[version]` policy is configured, so no tag has to match.",
            "Supply the missing evidence — a MAP, a declared version — or accept the review.",
            "Runtime RAM footprint 76 B is within the 256 B budget (180 B headroom).",
        ] {
            assert_eq!(safe_label(sentence), sentence);
        }
        // A backtick is prose; a slash, a drive prefix, markup and a control character are not.
        for withheld in [
            "read C:\\Users\\owner\\secrets.map",
            "open /etc/passwd",
            "see `D:` drive notes",
            "<tspan>`x`</tspan>",
            "tail\u{7}",
        ] {
            assert!(
                safe_label(withheld).starts_with("[label withheld"),
                "{withheld:?} must not reach a shared image"
            );
        }
    }

    #[test]
    fn legacy_detailed_html_uses_a4_page_rules() {
        assert!(crate::diff_render::EMBEDDED_STYLE.contains("@page"));
        assert!(crate::diff_render::EMBEDDED_STYLE.contains("size: A4"));
        assert!(crate::diff_render::EMBEDDED_STYLE.contains("table-header-group"));
    }

    #[test]
    fn share_scene_style_uses_frozen_light_roles_and_scaled_type_tokens() {
        let tokens: serde_json::Value =
            serde_json::from_str(include_str!("../../../assets/design-tokens.json")).unwrap();
        let source = include_str!("visual.rs");
        for color in [
            "#FFFFFF", "#171A1F", "#5D6673", "#667085", "#F1F3F5", "#2563EB",
        ] {
            assert!(
                tokens["color"]["light"]
                    .as_object()
                    .unwrap()
                    .values()
                    .any(|v| v == color)
            );
            assert!(source.contains(color));
        }
        for (size, token, scale) in [
            (40, "page", 2),
            (30, "section", 2),
            (26, "body", 2),
            (24, "metadata", 2),
            (20, "page", 1),
        ] {
            assert_eq!(
                tokens["typography"]["size"][token].as_u64().unwrap() * scale,
                size
            );
        }
    }
}

use crate::diff_render::{EMBEDDED_STYLE, escape_html};
use crate::release_render::ReleaseBundleReport;
use crate::{AnalysisDocumentDto, AnalyzeResultDto, DiffResultDto, GateResultsDto};
use firmwaresight_core::domain::diff::DiffResult;
use firmwaresight_core::domain::memory::{MemoryRegionUsage, RegionUsageState};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub const WIDTH: u32 = 1600;
pub const HEIGHT: u32 = 900;
/// Where the masthead hairline is drawn: under the title, above the subject line.
const MASTHEAD_RULE_Y: u32 = 80;
const LABEL_LIMIT: usize = 160;
const MAX_CARD_ROWS: usize = 7;
/// Punctuation a display label may carry. Everything outside it that is not alphanumeric withholds
/// the whole label; see `safe_label`.
const SAFE_LABEL_PUNCTUATION: &str = " .,_-+()[]@;:'!?=%`—–‘’“”";

#[derive(Debug, Clone, Copy)]
pub enum VisualFormat {
    Svg,
    Png,
    Html,
}

#[derive(Debug, thiserror::Error)]
pub enum VisualError {
    #[error("Visual rendering failed; check installed local fonts and retry")]
    Render,
    #[error("Export destination exists; choose a new file")]
    Exists,
    #[error("Export could not be written; choose a writable directory and retry")]
    Write,
}

/// Transient projection. Exact decimal integers are retained; only bar geometry uses floats.
#[derive(Debug, Clone)]
pub struct VisualReport {
    kind: &'static str,
    subject: String,
    identities: Vec<(String, String)>,
    metrics: Vec<Metric>,
    rows: Vec<Row>,
    limitations: Vec<String>,
    review_disposition: Option<(firmwaresight_core::domain::gate::EffectiveSeverity, usize)>,
}
#[derive(Debug, Clone)]
struct Metric {
    label: &'static str,
    value: Option<u64>,
    state: String,
    classification: String,
}
#[derive(Debug, Clone)]
struct Row {
    label: String,
    value: String,
    basis: String,
    bar: Option<(u64, u64)>,
}

/// Transient Gate facts; deliberately excludes portable-only policy/config metadata.
pub struct GateVisualInput<'a> {
    pub run_id: &'a str,
    pub snapshot_id: &'a str,
    pub findings: &'a [crate::GateFindingDto],
    pub original_severity: &'a str,
}

/// Display-only allowlist. Embedded paths, markup and controls are redacted as a whole.
#[must_use]
pub fn safe_label(value: &str) -> String {
    // Ordinary prose punctuation is useful evidence. Slash/backslash and drive
    // prefixes remain forbidden even when embedded in otherwise innocuous text. The backtick,
    // dash and curly-quote set stays allowed because Core writes its own rule sentences with
    // them — `release.expected_commit`, `[version] pattern`, "… budget …" — and withholding a
    // whole explanation for that punctuation would hide the reason a finding exists, which is
    // the opposite of what this list protects. Markup metacharacters, path separators and
    // control characters are not in it.
    let drive = value.as_bytes().windows(2).enumerate().any(|(i, w)| {
        w[0].is_ascii_alphabetic()
            && w[1] == b':'
            && (i == 0 || !value.as_bytes()[i - 1].is_ascii_alphanumeric())
    });
    if drive
        || value
            .chars()
            .any(|c| !c.is_alphanumeric() && !SAFE_LABEL_PUNCTUATION.contains(c))
    {
        return "[label withheld: unsafe or private text]".to_owned();
    }
    let mut result: String = value.chars().take(LABEL_LIMIT - 3).collect();
    if value.chars().count() > LABEL_LIMIT - 3 {
        result.push_str("...");
    }
    if result.trim().is_empty() {
        "[unnamed]".to_owned()
    } else {
        result
    }
}
fn bytes_text(value: Option<u64>) -> String {
    value.map_or_else(|| "Unknown".to_owned(), |n| format!("{n} bytes"))
}
fn budget(label: &'static str, b: &crate::dto::BudgetDto) -> Metric {
    Metric {
        label,
        value: b.bytes,
        state: safe_label(b.state),
        classification: safe_label(b.classification),
    }
}
fn memory_metrics(m: &crate::dto::MemoryDto) -> Vec<Metric> {
    vec![
        budget("FLASH / nonvolatile image", &m.nonvolatile_image_footprint),
        budget("RAM / runtime footprint", &m.runtime_ram_footprint),
    ]
}
impl VisualReport {
    #[must_use]
    pub fn analyze(dto: &AnalyzeResultDto, regions: &[MemoryRegionUsage]) -> Self {
        let mut report = Self {
            kind: "Analyze", subject: safe_label(&dto.artifact.file_name),
            identities: vec![("Snapshot ID".into(), safe_label(&dto.snapshot_id)), ("Observed artifact SHA-256".into(), safe_label(&dto.artifact.sha256)), ("FirmwareSight version".into(), safe_label(&dto.created_by_fwsight_version))],
            metrics: memory_metrics(&dto.memory), rows: Vec::new(), review_disposition: None,
            limitations: vec![format!("MAP capability: {}. Object attribution: {}.", dto.capabilities.map, dto.capabilities.object_attribution), "Capacity unknown for footprint budgets; region rows show mapped interval occupancy only.".into(), "Region remaining space does not prove allocator or stack/heap headroom.".into()],
        };
        for region in regions.iter().take(16) {
            report.rows.push(Row {
                label: safe_label(&region.name),
                value: format!(
                    "Mapped {} / capacity {}",
                    bytes_text(region.used_bytes),
                    bytes_text(region.capacity_bytes)
                ),
                basis: format!(
                    "{:?} / {:?} / {} / {:?}",
                    region.state,
                    region.address_kind,
                    safe_label(region.evidence_source),
                    region.risk
                ),
                bar: if region.state == RegionUsageState::Exact {
                    region
                        .used_bytes
                        .zip(region.capacity_bytes)
                        .filter(|(_, c)| *c > 0)
                } else {
                    None
                },
            });
            report
                .limitations
                .extend(region.reasons.iter().take(2).map(|r| safe_label(r)));
        }
        report.limitations.push(format!(
            "Regions shown: {} of {}. Full section/symbol evidence remains in analysis JSON.",
            report.rows.len(),
            regions.len()
        ));
        for b in [
            &dto.memory.nonvolatile_image_footprint,
            &dto.memory.runtime_ram_footprint,
        ] {
            if let Some(reason) = &b.reason {
                report.limitations.push(safe_label(reason));
            }
        }
        report
    }
    #[must_use]
    pub fn compare(dto: &DiffResultDto, core: &DiffResult) -> Self {
        let mut report = Self::diff(dto);
        let growth = core.top_symbol_growth(5);
        let max = growth
            .iter()
            .filter_map(|r| r.delta.and_then(|v| u64::try_from(v).ok()))
            .max()
            .unwrap_or(0);
        for r in growth {
            report.rows.push(Row {
                label: safe_label(&r.key),
                value: r
                    .delta
                    .map_or_else(|| "Unknown".into(), |v| format!("{v:+} bytes")),
                basis: "Core Top 5 changed-symbol size growth (not footprint attribution)".into(),
                bar: r
                    .delta
                    .and_then(|v| u64::try_from(v).ok())
                    .filter(|_| max > 0)
                    .map(|n| (n, max)),
            });
        }
        report
    }
    /// The legacy detailed HTML has DTO facts but no Core ranking input.
    #[must_use]
    pub fn diff(dto: &DiffResultDto) -> Self {
        let mut report = Self {
            kind: "Compare", subject: format!("A {} -> B {}", safe_label(&dto.base.artifact.file_name), safe_label(&dto.target.artifact.file_name)),
            identities: vec![("A Snapshot ID".into(), safe_label(&dto.base.snapshot_id)), ("B Snapshot ID".into(), safe_label(&dto.target.snapshot_id)), ("A observed SHA-256".into(), safe_label(&dto.base.artifact.sha256)), ("B observed SHA-256".into(), safe_label(&dto.target.artifact.sha256))],
            metrics: Vec::new(), rows: Vec::new(), review_disposition: None, limitations: vec![format!("Core comparability: {}. A -> B; delta = B minus A.", dto.memory.comparability), "Capacity unknown. Missing sides are absent, not measured zero.".into(), "Symbol/section size changes are navigation evidence; their sum does not explain FLASH/RAM growth.".into(), if dto.object_changes.available { "Per-object and per-library deltas come from linker MAP rows and are listed in the desktop Compare view and CLI text, not in this report.".into() } else { "Unattributed footprint remainder is not reliably quantifiable; object/library attribution unavailable.".into() }, format!("Changed symbols {}. Added {}. Removed {}. Ambiguous {}.", dto.counts.symbols_changed, dto.counts.symbols_added, dto.counts.symbols_removed, dto.counts.symbols_ambiguous)],
        };
        for (label, change) in [
            ("FLASH image", &dto.memory.nonvolatile),
            ("RAM runtime", &dto.memory.runtime_ram),
        ] {
            let (a, b) = if label == "FLASH image" {
                (&dto.base.memory.nonvolatile, &dto.target.memory.nonvolatile)
            } else {
                (&dto.base.memory.runtime_ram, &dto.target.memory.runtime_ram)
            };
            report.rows.push(Row {
                label: label.into(),
                value: format!(
                    "A {} | B {} | Delta {}",
                    bytes_text(change.base),
                    bytes_text(change.target),
                    change
                        .delta
                        .map_or_else(|| "Unknown".into(), |v| format!("{v:+} bytes"))
                ),
                basis: format!(
                    "{} / A {} ({}) / B {} ({})",
                    change.comparability, a.state, a.classification, b.state, b.classification
                ),
                bar: None,
            });
            if let Some(reason) = &change.reason {
                report.limitations.push(safe_label(reason));
            }
        }
        report
    }
    /// Optional pair is (actual Release ID, project release version), never inferred.
    #[must_use]
    pub fn gate(
        analysis: &AnalysisDocumentDto,
        gate: &GateResultsDto,
        release: Option<(&str, &str)>,
    ) -> Self {
        Self::gate_evidence(
            analysis,
            &GateVisualInput {
                run_id: &gate.run_id,
                snapshot_id: &gate.snapshot_id,
                findings: &gate.findings,
                original_severity: gate.extensions.overall_effective_severity,
            },
            release,
        )
    }
    #[must_use]
    pub fn gate_evidence(
        analysis: &AnalysisDocumentDto,
        gate: &GateVisualInput<'_>,
        release: Option<(&str, &str)>,
    ) -> Self {
        let mut report = Self {
            kind: if release.is_some() { "Release" } else { "Gate review" }, subject: release.map_or_else(|| "Current Gate evidence".into(), |(_, version)| safe_label(version)),
            identities: vec![("Snapshot ID".into(), safe_label(gate.snapshot_id)), ("Gate Run ID".into(), safe_label(gate.run_id)), ("FirmwareSight version".into(), safe_label(&analysis.snapshot.fwsight_version))],
            metrics: memory_metrics(&analysis.memory), rows: Vec::new(), review_disposition: None, limitations: vec![format!("Original Gate effective severity: {}. Findings retain their original states.", safe_label(gate.original_severity)), "PASS is a policy result, not shipping permission or security certification.".into(), "SHA-256 checks bytes, not publisher identity or build provenance.".into(), "Capacity unknown. Workspace Git facts are not proof of the shipped firmware build.".into()],
        };
        if let Some((id, version)) = release {
            report
                .identities
                .push(("Release ID".into(), safe_label(id)));
            report
                .identities
                .push(("Project release version".into(), safe_label(version)));
        }
        for finding in gate.findings.iter().take(32) {
            report.rows.push(Row {
                label: safe_label(&finding.rule_id),
                // An N/A finding's PASS is a schema filler, not a verdict: it is not printed beside N/A
                // (R6, `03_DESIGN/09` §8.2), the same rule as the text and Markdown reports.
                value: if finding.state == "N/A" {
                    format!("{} {}", state_icon(finding.state), finding.state)
                } else {
                    format!(
                        "{} {} / severity {}",
                        state_icon(finding.state),
                        finding.state,
                        finding.effective_severity
                    )
                },
                basis: safe_label(&finding.summary),
                bar: None,
            });
        }
        report.limitations.push(format!(
            "Findings shown: {} of {}. Read gate-results JSON for full evidence and remediation.",
            report.rows.len(),
            gate.findings.len()
        ));
        for artifact in analysis.artifacts.iter().take(8) {
            report.identities.push((
                format!(
                    "Observed {} bytes: {}",
                    artifact.byte_size,
                    safe_label(&artifact.file_name)
                ),
                safe_label(artifact.sha256.hex()),
            ));
        }
        report
    }
    /// Both original findings and the Core-owned disposition after recorded review acceptances.
    #[must_use]
    pub fn gate_review(
        analysis: &AnalysisDocumentDto,
        gate: &GateVisualInput<'_>,
        disposition: firmwaresight_core::domain::gate::EffectiveSeverity,
        accepted_reviews: usize,
    ) -> Self {
        let mut report = Self::gate_evidence(analysis, gate, None);
        report.review_disposition = Some((disposition, accepted_reviews));
        report.limitations.push(format!("Disposition after {accepted_reviews} recorded review acceptances: {}. Original findings are unchanged.", disposition.as_str()));
        report
    }
    #[must_use]
    pub fn release(input: &ReleaseBundleReport<'_>) -> Self {
        let mut report = Self::gate(
            input.analysis,
            input.gate,
            Some((input.release_id, input.model.version.as_str())),
        );
        report.review_disposition =
            Some((input.model.disposition, input.reviews.acceptances.len()));
        report.limitations.push(format!("Release disposition after accepted reviews: {:?}. Original findings above remain unchanged.", input.model.disposition));
        report
    }
}
fn state_icon(state: &str) -> &'static str {
    match state {
        "PASS" => "+",
        "REVIEW" => "!",
        "BLOCK" => "x",
        "N/A" => "-",
        _ => "o",
    }
}

/// A4 paper geometry is separate from the 16:9 share scene. No native PDF claim.
pub const PRINT_STYLE: &str = r#"
@page { size: A4; margin: 16mm; }
@media print {
 body { background: #FFFFFF; padding: 0; }
 main { max-width: none; }
 .visual-summary { break-inside: avoid; }
 table { width: 100%; table-layout: fixed; }
 thead { display: table-header-group; }
 tr { break-inside: avoid; }
 th, td { overflow-wrap: anywhere; }
 h1, h2, h3 { break-after: avoid; }
 .visual-scene { width: 100%; height: auto; }
}
.visual-scene { display: block; width: 100%; height: auto; }
td, th, .mono { overflow-wrap: anywhere; }
"#;

/// Embedded overview, reused by the existing detailed reports.
#[must_use]
pub fn summary_html(report: &VisualReport) -> String {
    let mut out = format!(
        "<section class=\"visual-summary\"><h2>{} visual overview</h2>",
        report.kind
    );
    // HTML's SVG namespace is implicit. Omit the XML namespace declaration only when
    // embedding, so the unchanged Bundle offline-reference verifier remains strict.
    out.push_str(&svg(report).replace(" xmlns=\"http://www.w3.org/2000/svg\"", ""));
    out.push_str("<p>Visual summary only. Detailed evidence follows. Browser Print / Save as PDF; native PDF export is unavailable.</p></section>");
    out
}
fn html(report: &VisualReport) -> String {
    let mut out = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>FirmwareSight {}</title><style>{EMBEDDED_STYLE}{PRINT_STYLE}</style><body><main>",
        report.kind
    );
    out.push_str(&summary_html(report));
    out.push_str("<h2>Identity and evidence detail</h2><table><caption>The identities this summary is about.</caption><thead><tr><th scope=\"col\">Identity</th><th scope=\"col\">Value</th></tr></thead><tbody>");
    for (label, value) in &report.identities {
        out.push_str(&format!(
            "<tr><td>{}</td><td class=\"mono\">{}</td></tr>",
            escape_html(label),
            escape_html(value)
        ));
    }
    out.push_str("</tbody></table><h2>Measurements (exact bytes)</h2><table><caption>Every figure in the summary, in exact bytes, with its basis.</caption><thead><tr><th scope=\"col\">Fact</th><th scope=\"col\">Value</th><th scope=\"col\">Basis</th></tr></thead><tbody>");
    for m in &report.metrics {
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{} / {}</td></tr>",
            m.label,
            bytes_text(m.value),
            m.state,
            m.classification
        ));
    }
    for row in &report.rows {
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&row.label),
            escape_html(&row.value),
            escape_html(&row.basis)
        ));
    }
    out.push_str("</tbody></table><h2>Limits and next steps</h2><ul>");
    for text in &report.limitations {
        out.push_str(&format!("<li>{}</li>", escape_html(text)));
    }
    out.push_str("</ul><p>Use the original JSON / detailed Compare HTML / verified Release Bundle for complete engineering evidence.</p></main></body></html>");
    out
}
fn clipped(text: &str, limit: usize) -> String {
    // Wide CJK glyphs consume two ASCII cells. Never truncate measured numbers with this helper.
    let width: usize = text.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum();
    if width <= limit {
        return text.into();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in text.chars() {
        used += if c.is_ascii() { 1 } else { 2 };
        if used > limit.saturating_sub(3) {
            break;
        }
        out.push(c);
    }
    out.push_str("...");
    out
}
fn svg_text(out: &mut String, x: u32, y: u32, size: u32, text: &str, color: &str, mono: bool) {
    out.push_str(&format!("<text x=\"{x}\" y=\"{y}\" font-size=\"{size}\" fill=\"{color}\" font-family=\"{}\">{}</text>", if mono { "Consolas, Liberation Mono, monospace" } else { "Segoe UI, Arial, Microsoft YaHei, Noto Sans CJK SC, sans-serif" }, escape_html(text)));
}
fn svg(report: &VisualReport) -> String {
    // Frozen light colors; typography uses 2x tokens on the share scene, 1x on paper.
    // Geometry follows 16px/24px/48px token multiples, independently of numeric facts.
    // R6 report grammar: `<title>` and `<desc>` carry the same facts as the picture, so a reader who cannot
    // see it gets the subject, the figures and the first limit in words (`03_DESIGN/09` §8.2).
    let desc = std::iter::once(report.subject.clone())
        .chain(report.metrics.iter().map(|m| {
            format!(
                "{}: {} ({} / {})",
                m.label,
                bytes_text(m.value),
                m.state,
                m.classification
            )
        }))
        .chain(report.limitations.first().cloned())
        .collect::<Vec<_>>()
        .join(". ");
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"visual-scene\" viewBox=\"0 0 {WIDTH} {HEIGHT}\" width=\"{WIDTH}\" height=\"{HEIGHT}\" role=\"img\" aria-label=\"FirmwareSight {} visual summary\"><title>FirmwareSight {} — {}</title><desc>{}</desc><rect width=\"1600\" height=\"900\" fill=\"#FFFFFF\"/>",
        report.kind,
        report.kind,
        escape_html(&report.subject),
        escape_html(&desc)
    );
    svg_text(
        &mut out,
        48,
        64,
        40,
        &format!("FirmwareSight · {} summary", report.kind),
        "#171A1F",
        false,
    );
    // The masthead title ends in a hairline, the same rule the HTML and text reports draw under theirs. It
    // sits between the title (baseline 64) and the subject (cap top ~90), clear of every line below it.
    out.push_str(&format!(
        "<line x1=\"48\" y1=\"{MASTHEAD_RULE_Y}\" x2=\"1552\" y2=\"{MASTHEAD_RULE_Y}\" stroke=\"#D9DEE5\" stroke-width=\"2\"/>"
    ));
    svg_text(
        &mut out,
        48,
        112,
        30,
        &clipped(&report.subject, 80),
        "#5D6673",
        false,
    );
    let identity = report
        .identities
        .iter()
        .take(2)
        .map(|(label, value)| format!("{label}: {}", clipped(value, 36)))
        .collect::<Vec<_>>()
        .join(" / ");
    svg_text(
        &mut out,
        48,
        144,
        24,
        &clipped(&identity, 102),
        "#5D6673",
        true,
    );
    for (i, m) in report.metrics.iter().enumerate() {
        let x = 48 + u32::try_from(i).unwrap_or(0) * 768;
        svg_text(&mut out, x, 176, 30, m.label, "#5D6673", false);
        svg_text(
            &mut out,
            x,
            224,
            40,
            &bytes_text(m.value),
            if m.value.is_some() {
                "#171A1F"
            } else {
                "#667085"
            },
            true,
        );
        svg_text(
            &mut out,
            x,
            264,
            26,
            &format!("{} / {}", m.state, m.classification),
            "#5D6673",
            false,
        );
    }
    if !report.metrics.is_empty() {
        svg_text(
            &mut out,
            48,
            296,
            24,
            "Capacity unknown for footprint budgets. Missing/partial evidence remains qualified.",
            "#667085",
            false,
        );
    }
    let start = if report.metrics.is_empty() { 192 } else { 320 };
    for (i, row) in report.rows.iter().take(MAX_CARD_ROWS).enumerate() {
        let y = start + u32::try_from(i).unwrap_or(0) * 64;
        if y > 704 {
            break;
        }
        if let Some((n, d)) = row.bar {
            let width = ((n as f64 / d as f64).clamp(0.0, 1.0) * 144.0).round();
            out.push_str(&format!(
                "<rect x=\"1392\" y=\"{}\" width=\"144\" height=\"16\" fill=\"#F1F3F5\"/>",
                y + 12
            ));
            // Exact integers remain in the row; a rounded-zero decorative bar
            // has no area and must not be clamped into a misleading positive area.
            if width > 0.0 {
                out.push_str(&format!(
                    "<rect x=\"1392\" y=\"{}\" width=\"{width}\" height=\"16\" fill=\"#2563EB\"/>",
                    y + 12
                ));
            }
        }
        svg_text(
            &mut out,
            48,
            y,
            26,
            &clipped(&row.label, 28),
            "#171A1F",
            false,
        );
        if row.value.contains(" | ") {
            // Each A/B/delta value gets its own column; even full u64 values remain exact.
            for (column, value) in row.value.split(" | ").enumerate() {
                svg_text(
                    &mut out,
                    480 + u32::try_from(column).unwrap_or(0) * 352,
                    y,
                    20,
                    value,
                    "#171A1F",
                    true,
                );
            }
        } else {
            svg_text(&mut out, 480, y, 20, &row.value, "#171A1F", true);
        }
        svg_text(
            &mut out,
            48,
            y + 28,
            24,
            &clipped(&row.basis, 100),
            "#5D6673",
            false,
        );
    }
    let shown = if report.metrics.is_empty() {
        MAX_CARD_ROWS.min(report.rows.len())
    } else {
        7.min(report.rows.len())
    };
    let scope = if let Some((disposition, accepted)) = report.review_disposition {
        format!(
            "Disposition after {accepted} recorded acceptances: {}. Original findings unchanged.",
            disposition.as_str()
        )
    } else if report.kind == "Compare" {
        report
            .limitations
            .iter()
            .find(|s| s.starts_with("Changed symbols"))
            .cloned()
            .unwrap_or_default()
    } else {
        format!(
            "Summary rows {} of {}. Full labels, basis and identities in HTML detail.",
            shown,
            report.rows.len()
        )
    };
    svg_text(
        &mut out,
        48,
        760,
        26,
        &clipped(&scope, 102),
        "#5D6673",
        false,
    );
    svg_text(
        &mut out,
        48,
        800,
        26,
        if report.kind == "Compare" {
            "Core symbol ranking is not FLASH/RAM attribution. Unknown is not zero."
        } else if report.kind == "Analyze" {
            "Mapped occupancy is not footprint utilisation or allocator headroom."
        } else {
            "PASS is not shipping permission. SHA-256 checks bytes, not publisher identity."
        },
        "#667085",
        false,
    );
    svg_text(
        &mut out,
        48,
        840,
        26,
        &clipped(report.limitations.first().map_or("", String::as_str), 102),
        "#5D6673",
        false,
    );
    svg_text(
        &mut out,
        48,
        884,
        20,
        &format!(
            "Generated by FirmwareSight {}. PNG text uses this computer's installed fonts.",
            env!("CARGO_PKG_VERSION")
        ),
        "#667085",
        false,
    );
    out.push_str("</svg>");
    out
}

/// PNG comes from the exact SVG. Fixed input/font environment gives deterministic bytes.
/// System font variation is an explicit portability limit.
/// # Errors
/// Missing local fonts, parse error or raster allocation failure.
pub fn render(report: &VisualReport, format: VisualFormat) -> Result<Vec<u8>, VisualError> {
    match format {
        VisualFormat::Svg => Ok(svg(report).into_bytes()),
        VisualFormat::Html => Ok(html(report).into_bytes()),
        VisualFormat::Png => {
            let source = svg(report);
            let mut options = resvg::usvg::Options::default();
            options.fontdb_mut().load_system_fonts();
            if options.fontdb.faces().next().is_none() {
                return Err(VisualError::Render);
            }
            let tree =
                resvg::usvg::Tree::from_str(&source, &options).map_err(|_| VisualError::Render)?;
            let mut pixmap =
                resvg::tiny_skia::Pixmap::new(WIDTH, HEIGHT).ok_or(VisualError::Render)?;
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::default(),
                &mut pixmap.as_mut(),
            );
            pixmap.encode_png().map_err(|_| VisualError::Render)
        }
    }
}

static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
/// Write and flush a same-directory temp file, then publish atomically without replacement.
/// Unsupported hard-link filesystems fail closed, leaving no partial destination.
/// # Errors
/// Existing destination, inaccessible directory, or unsupported atomic publication.
pub fn write_new(destination: &Path, bytes: &[u8]) -> Result<(), VisualError> {
    if destination.exists() {
        return Err(VisualError::Exists);
    }
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let temp = parent.join(format!(
        ".fws-visual-{}-{}.tmp",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)
        .map_err(|_| VisualError::Write)?;
    let result = (|| {
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| VisualError::Write)?;
        drop(file);
        std::fs::hard_link(&temp, destination).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                VisualError::Exists
            } else {
                VisualError::Write
            }
        })
    })();
    let _ = std::fs::remove_file(&temp);
    result
}
