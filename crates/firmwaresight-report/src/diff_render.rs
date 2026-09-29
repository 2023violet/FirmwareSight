//! Rendering of a `DiffResultDto`: portable JSON, the CLI's human default, and the self-contained
//! HTML export.
//!
//! Three renderings, one source. Nothing here compares, sums or re-derives a fact; the DTO already
//! carries what Core decided, and re-deciding it in a renderer is how a UI ends up disagreeing with
//! the file it exported (`AGENTS.md` 3).
//!
//! The HTML is a single file with embedded CSS: no CDN, no remote stylesheet, no script, no font
//! fetch, no telemetry, and no generated timestamp. It is therefore byte-stable, which makes the
//! golden in `golden/reports/` a real check instead of a fuzzy one.

use firmwaresight_core::domain::diff::{ByteChange, Contributor, DiffResult};

use crate::diff::{ByteChangeDto, DiffResultDto, ObjectChangesDto};

/// Every colour, size and radius below is a value from `assets/design-tokens.json` (0.2.1, light
/// theme), restated here because a standalone file cannot read the desktop's `tokens.css`.
/// `html_export_colours_come_from_the_frozen_token_set` re-checks that against the token file, so
/// this block cannot quietly drift into magic values.
const EMBEDDED_STYLE: &str = r#":root {
  --fs-color-bg-canvas: #F7F8FA;
  --fs-color-bg-surface: #FFFFFF;
  --fs-color-bg-subtle: #F1F3F5;
  --fs-color-text-primary: #171A1F;
  --fs-color-text-secondary: #5D6673;
  --fs-color-border-default: #D9DEE5;
  --fs-color-diff-added: #18794E;
  --fs-color-diff-added-bg: #E7F3EC;
  --fs-color-diff-removed: #C9372C;
  --fs-color-diff-removed-bg: #FBECEA;
  --fs-color-diff-changed: #2563EB;
  --fs-color-diff-changed-bg: #E9F0FD;
  --fs-color-status-unknown: #667085;
  --fs-font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial, sans-serif;
  --fs-font-mono: ui-monospace, "SFMono-Regular", Consolas, "Liberation Mono", monospace;
  --fs-font-size-metadata: 12px;
  --fs-font-size-body: 13px;
  --fs-font-size-section: 15px;
  --fs-font-size-page: 20px;
  --fs-space-2: 8px;
  --fs-space-3: 12px;
  --fs-space-4: 16px;
  --fs-space-6: 24px;
  --fs-radius-control: 6px;
  --fs-radius-panel: 8px;
  --fs-radius-pill: 999px;
  --fs-border-width-hairline: 1px;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  padding: var(--fs-space-6);
  background: var(--fs-color-bg-canvas);
  color: var(--fs-color-text-primary);
  font-family: var(--fs-font-ui);
  font-size: var(--fs-font-size-body);
  line-height: 20px;
}

main {
  max-width: 1100px;
  margin: 0 auto;
}

h1 {
  margin: 0 0 var(--fs-space-2);
  font-size: var(--fs-font-size-page);
  line-height: 28px;
  font-weight: 600;
}

h2 {
  margin: var(--fs-space-6) 0 var(--fs-space-3);
  font-size: var(--fs-font-size-section);
  line-height: 22px;
  font-weight: 600;
}

.brand,
.meta,
.empty {
  color: var(--fs-color-text-secondary);
  font-size: var(--fs-font-size-metadata);
  line-height: 18px;
  margin: 0;
}

.card {
  background: var(--fs-color-bg-surface);
  border: var(--fs-border-width-hairline) solid var(--fs-color-border-default);
  border-radius: var(--fs-radius-panel);
  padding: var(--fs-space-4);
}

.notice {
  background: var(--fs-color-bg-subtle);
  border: var(--fs-border-width-hairline) solid var(--fs-color-border-default);
  border-radius: var(--fs-radius-control);
  padding: var(--fs-space-3);
  margin: 0 0 var(--fs-space-2);
}

.notice ul {
  margin: var(--fs-space-2) 0 0;
  padding-left: var(--fs-space-4);
}

table {
  width: 100%;
  border-collapse: collapse;
  background: var(--fs-color-bg-surface);
  border: var(--fs-border-width-hairline) solid var(--fs-color-border-default);
  border-radius: var(--fs-radius-panel);
}

caption {
  caption-side: top;
  text-align: left;
  padding: 0 0 var(--fs-space-2);
  color: var(--fs-color-text-secondary);
  font-size: var(--fs-font-size-metadata);
  line-height: 18px;
}

th,
td {
  border-top: var(--fs-border-width-hairline) solid var(--fs-color-border-default);
  padding: var(--fs-space-2) var(--fs-space-3);
  text-align: left;
  vertical-align: top;
}

th {
  font-weight: 600;
  background: var(--fs-color-bg-subtle);
}

td.num,
th.num {
  text-align: right;
  font-family: var(--fs-font-mono);
  white-space: nowrap;
}

.mono,
.sha,
.locator {
  font-family: var(--fs-font-mono);
}

.delta.positive::before {
  content: "+";
}

.delta.negative {
  color: var(--fs-color-diff-removed);
}

.added {
  color: var(--fs-color-diff-added);
  background: var(--fs-color-diff-added-bg);
}

.removed {
  color: var(--fs-color-diff-removed);
  background: var(--fs-color-diff-removed-bg);
}

.changed {
  color: var(--fs-color-diff-changed);
  background: var(--fs-color-diff-changed-bg);
}

.chip {
  display: inline-block;
  padding: 0 var(--fs-space-2);
  border-radius: var(--fs-radius-pill);
  font-size: var(--fs-font-size-metadata);
  line-height: 18px;
  white-space: nowrap;
}

.unknown {
  color: var(--fs-color-status-unknown);
}

.side {
  display: grid;
  grid-template-columns: 96px 1fr;
  gap: var(--fs-space-2) var(--fs-space-3);
}

.side dt {
  color: var(--fs-color-text-secondary);
  font-size: var(--fs-font-size-metadata);
  line-height: 18px;
}

.side dd {
  margin: 0;
}

footer {
  margin-top: var(--fs-space-6);
  color: var(--fs-color-text-secondary);
  font-size: var(--fs-font-size-metadata);
  line-height: 18px;
}
"#;

/// The `--json` contract: exactly one JSON document on stdout, nothing else.
#[must_use]
pub fn render_json(dto: &DiffResultDto) -> String {
    let mut out = crate::render::to_json(dto);
    out.push('\n');
    out
}

/// The text a reader sees in `golden/reports/p2-diff.html` and in the desktop's exported file.
#[must_use]
pub fn render_html(dto: &DiffResultDto) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("<meta charset=\"utf-8\">\n");
    out.push_str("<title>FirmwareSight build diff</title>\n");
    out.push_str("<style>\n");
    out.push_str(EMBEDDED_STYLE);
    out.push_str("</style>\n</head>\n<body>\n<main>\n");

    out.push_str("<h1>Build diff</h1>\n");
    out.push_str(&format!(
        "<p class=\"brand\">FirmwareSight {} &middot; <span class=\"mono\">{}</span> schema version {}</p>\n",
        env!("CARGO_PKG_VERSION"),
        escape_html(dto.schema),
        dto.schema_version
    ));

    render_sides(dto, &mut out);
    render_memory(dto, &mut out);
    render_warnings(dto, &mut out);
    render_counts(dto, &mut out);
    render_sections(dto, &mut out);
    render_symbols(dto, &mut out);
    render_objects(&dto.object_changes, &mut out);

    out.push_str(&format!(
        "<footer>Deterministic export: no timestamp, no scripts, no network requests. \
FirmwareSight {}.</footer>\n",
        env!("CARGO_PKG_VERSION")
    ));
    out.push_str("</main>\n</body>\n</html>\n");
    out
}

fn render_sides(dto: &DiffResultDto, out: &mut String) {
    out.push_str("<h2>Builds compared</h2>\n<div class=\"card\">\n");
    for (label, side) in [("Base (old)", &dto.base), ("Target (new)", &dto.target)] {
        out.push_str(&format!("<h3>{label}</h3>\n<dl class=\"side\">\n"));
        out.push_str(&term("Artifact", &escape_html(&side.artifact.file_name)));
        out.push_str(&term(
            "SHA-256",
            &format!(
                "<span class=\"sha\">{}</span>",
                escape_html(&side.artifact.sha256)
            ),
        ));
        out.push_str(&term("Size", &format!("{} bytes", side.artifact.byte_size)));
        out.push_str(&term(
            "Snapshot",
            &format!(
                "<span class=\"locator\">{}</span>",
                escape_html(&side.snapshot_id)
            ),
        ));
        let evidence = &side.memory.evidence;
        out.push_str(&term(
            "Layout",
            &format!(
                "{} &middot; {}",
                escape_html(&evidence.layout_source),
                evidence
                    .weakest_evidence_basis
                    .as_deref()
                    .map(escape_html)
                    .unwrap_or_else(|| "no recorded basis".to_owned())
            ),
        ));
        out.push_str(&term(
            "MAP",
            if evidence.map_backed {
                "stored for this build"
            } else {
                "no MAP stored for this build"
            },
        ));
        out.push_str("</dl>\n");
    }
    out.push_str("</div>\n");
}

fn term(label: &str, value: &str) -> String {
    format!("<dt>{label}</dt><dd>{value}</dd>\n")
}

fn render_memory(dto: &DiffResultDto, out: &mut String) {
    out.push_str("<h2>Memory totals</h2>\n");
    out.push_str(&format!(
        "<p class=\"meta\">Comparability: {} &mdash; the weaker side caps both budgets.</p>\n",
        escape_html(dto.memory.comparability)
    ));
    out.push_str("<table>\n<caption>Delta = target &minus; base. A dash is an absence, not a zero.</caption>\n");
    out.push_str("<thead><tr><th>Budget</th><th class=\"num\">Base</th><th class=\"num\">Target</th><th class=\"num\">Delta</th><th>Reading</th></tr></thead>\n<tbody>\n");
    out.push_str(&budget_row(
        "Nonvolatile / load image",
        &dto.memory.nonvolatile,
    ));
    out.push_str(&budget_row("Runtime RAM", &dto.memory.runtime_ram));
    out.push_str("</tbody>\n</table>\n");

    for (label, side) in [("Base", &dto.base), ("Target", &dto.target)] {
        out.push_str(&format!(
            "<p class=\"meta\">{label} side: nonvolatile {} ({}) &middot; runtime RAM {} ({})</p>\n",
            bytes_text(side.memory.nonvolatile.bytes),
            side.memory.nonvolatile.state,
            bytes_text(side.memory.runtime_ram.bytes),
            side.memory.runtime_ram.state,
        ));
    }
}

fn bytes_text(bytes: Option<u64>) -> String {
    match bytes {
        Some(value) => format!("{value} bytes"),
        None => "\u{2014} not recorded".to_owned(),
    }
}

fn budget_row(name: &str, change: &ByteChangeDto) -> String {
    let mut row = format!("<tr><th>{name}</th>");
    row.push_str(&num_cell(change.base));
    row.push_str(&num_cell(change.target));
    row.push_str(&delta_cell(change.delta));
    row.push_str(&format!(
        "<td>{}</td></tr>\n",
        change
            .reason
            .as_deref()
            .map(escape_html)
            .unwrap_or_else(|| "exact on both sides".to_owned())
    ));
    row
}

fn num_cell(value: Option<u64>) -> String {
    match value {
        Some(bytes) => format!("<td class=\"num\">{bytes}</td>"),
        None => "<td class=\"num unknown\" aria-label=\"not recorded\">\u{2014}</td>".to_owned(),
    }
}

fn delta_cell(delta: Option<i64>) -> String {
    match delta {
        Some(0) => "<td class=\"num\">0</td>".to_owned(),
        Some(value) if value > 0 => {
            format!("<td class=\"num delta positive\">{}</td>", value.abs())
        }
        Some(value) => format!("<td class=\"num delta negative\">{value}</td>"),
        None => "<td class=\"num unknown\" aria-label=\"no delta\">\u{2014}</td>".to_owned(),
    }
}

fn render_warnings(dto: &DiffResultDto, out: &mut String) {
    if dto.warnings.is_empty() {
        return;
    }
    out.push_str(
        "<div class=\"notice\">\n<p><strong>Read this before the tables</strong></p>\n<ul>\n",
    );
    for warning in &dto.warnings {
        out.push_str(&format!(
            "<li><span class=\"mono\">{}</span> &mdash; {}</li>\n",
            escape_html(&warning.code),
            escape_html(&warning.message)
        ));
    }
    out.push_str("</ul>\n</div>\n");
}

fn render_counts(dto: &DiffResultDto, out: &mut String) {
    out.push_str("<h2>What moved</h2>\n<div class=\"card\"><p class=\"meta\">");
    out.push_str(&format!(
        "Sections: {} added, {} removed, {} changed, {} left unpaired because the name repeats.",
        dto.counts.sections_added,
        dto.counts.sections_removed,
        dto.counts.sections_changed,
        dto.counts.sections_ambiguous,
    ));
    out.push_str(&format!(
        " Symbols: {} added, {} removed, {} changed, {} left unpaired because the key repeats.",
        dto.counts.symbols_added,
        dto.counts.symbols_removed,
        dto.counts.symbols_changed,
        dto.counts.symbols_ambiguous,
    ));
    out.push_str("</p><p class=\"meta\">");
    out.push_str(&format!(
        "Found identical and left out of the tables: {} section(s), {} symbol(s).",
        dto.unchanged.sections, dto.unchanged.symbols
    ));
    out.push_str("</p></div>\n");
}

fn render_sections(dto: &DiffResultDto, out: &mut String) {
    out.push_str("<h2>Section changes</h2>\n");
    if dto.section_changes.is_empty() {
        out.push_str("<p class=\"empty\">No section row differs between these two builds.</p>\n");
        return;
    }
    out.push_str("<table>\n<caption>Every changed row, not a truncated top list.</caption>\n");
    out.push_str("<thead><tr><th>Change</th><th>Section</th><th class=\"num\">Base size</th><th class=\"num\">Target size</th><th class=\"num\">Delta</th><th>Fields</th><th>Where to check</th></tr></thead>\n<tbody>\n");
    for row in &dto.section_changes {
        out.push_str(&change_row(
            row.change,
            &row.key,
            &row.file_size.base,
            &row.file_size.target,
            row.file_size.delta,
            &fields_text(&row.differing_fields, &row.indeterminate_fields),
            &locator_text(
                row.base.as_ref().map(|s| s.index),
                row.target.as_ref().map(|s| s.index),
            ),
            row.ambiguous,
        ));
    }
    out.push_str("</tbody>\n</table>\n");
}

fn render_symbols(dto: &DiffResultDto, out: &mut String) {
    out.push_str("<h2>Symbol changes</h2>\n");
    if dto.symbol_changes.is_empty() {
        out.push_str("<p class=\"empty\">No symbol row differs between these two builds.</p>\n");
        return;
    }
    out.push_str(
        "<table>\n<caption>Every changed row. Growth ranking never replaces this list.</caption>\n",
    );
    out.push_str("<thead><tr><th>Change</th><th>Symbol</th><th class=\"num\">Base size</th><th class=\"num\">Target size</th><th class=\"num\">Delta</th><th>Fields</th><th>Where to check</th></tr></thead>\n<tbody>\n");
    for row in &dto.symbol_changes {
        out.push_str(&change_row(
            row.change,
            &row.name,
            &row.size.base,
            &row.size.target,
            row.size.delta,
            &fields_text(&row.differing_fields, &row.indeterminate_fields),
            &locator_text(
                row.base.as_ref().map(|s| s.ordinal),
                row.target.as_ref().map(|s| s.ordinal),
            ),
            row.ambiguous,
        ));
    }
    out.push_str("</tbody>\n</table>\n");
}

#[allow(clippy::too_many_arguments)]
fn change_row(
    change: &str,
    name: &str,
    base: &Option<u64>,
    target: &Option<u64>,
    delta: Option<i64>,
    fields: &str,
    locator: &str,
    ambiguous: bool,
) -> String {
    let mut row = format!(
        "<tr><td><span class=\"chip {}\">{} {}</span></td>",
        change_class(change),
        change_glyph(change),
        escape_html(change)
    );
    let label = if ambiguous {
        format!(
            "{} <span class=\"unknown\">(name repeats on one or both sides)</span>",
            escape_html(name)
        )
    } else {
        escape_html(name)
    };
    row.push_str(&format!("<td class=\"mono\">{label}</td>"));
    row.push_str(&num_cell(*base));
    row.push_str(&num_cell(*target));
    row.push_str(&delta_cell(delta));
    row.push_str(&format!("<td>{}</td>", escape_html(fields)));
    row.push_str(&format!(
        "<td class=\"locator\">{}</td></tr>\n",
        escape_html(locator)
    ));
    row
}

fn change_class(change: &str) -> &'static str {
    match change {
        "Added" => "added",
        "Removed" => "removed",
        _ => "changed",
    }
}

fn change_glyph(change: &str) -> &'static str {
    match change {
        "Added" => "+",
        "Removed" => "\u{2212}",
        _ => "~",
    }
}

fn fields_text(differing: &[String], indeterminate: &[String]) -> String {
    let mut parts = Vec::new();
    if !differing.is_empty() {
        parts.push(format!("differs: {}", differing.join(", ")));
    }
    if !indeterminate.is_empty() {
        parts.push(format!("not decidable: {}", indeterminate.join(", ")));
    }
    if parts.is_empty() {
        "no recorded field differs".to_owned()
    } else {
        parts.join(" \u{00b7} ")
    }
}

fn locator_text(base_index: Option<i64>, target_index: Option<i64>) -> String {
    match (base_index, target_index) {
        (Some(from), Some(to)) => format!("base row {from} / target row {to}"),
        (Some(from), None) => format!("base row {from}"),
        (None, Some(to)) => format!("target row {to}"),
        (None, None) => String::new(),
    }
}

fn render_objects(attribution: &ObjectChangesDto, out: &mut String) {
    out.push_str("<h2>Object and module changes</h2>\n");
    if attribution.available {
        out.push_str("<p class=\"meta\">Attribution is available for these builds.</p>\n");
        return;
    }
    out.push_str(&format!(
        "<p class=\"meta\">Not claimable here: {}</p>\n",
        escape_html(&attribution.reason)
    ));
}

/// The CLI's human default: summary plus the contributors a reader asks about first.
///
/// This reads the Core result directly rather than a re-derived view of the DTO, and its ranking
/// comes from Core's own `top_*_growth` / `largest_added_*` selectors. A renderer that ranked rows
/// itself would be a second definition of "major contributor" (`AGENTS.md` 3).
#[must_use]
pub fn render_human(result: &DiffResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Base    {}  {}\nTarget  {}  {}\n\n",
        result.base_artifact.file_name,
        short_sha(&result.base_artifact.sha256),
        result.target_artifact.file_name,
        short_sha(&result.target_artifact.sha256),
    ));

    out.push_str("Memory\n");
    out.push_str(&format!(
        "  Nonvolatile / load image: {}\n",
        human_change(&result.memory.nonvolatile)
    ));
    out.push_str(&format!(
        "  Runtime RAM:              {}\n",
        human_change(&result.memory.runtime_ram)
    ));
    out.push_str(&format!(
        "  Comparability:            {}\n",
        result.memory.comparability.label()
    ));
    out.push_str(&format!(
        "  Base side states:         nonvolatile {} / RAM {}\n",
        result.memory.base.nonvolatile.state.as_str(),
        result.memory.base.runtime_ram.state.as_str()
    ));
    out.push_str(&format!(
        "  Target side states:       nonvolatile {} / RAM {}\n\n",
        result.memory.target.nonvolatile.state.as_str(),
        result.memory.target.runtime_ram.state.as_str()
    ));

    out.push_str("Counts\n");
    out.push_str(&format!(
        "  Sections  +{} -{} ~{} ({} rows whose name repeats)\n",
        result.counts.sections_added,
        result.counts.sections_removed,
        result.counts.sections_changed,
        result.counts.sections_ambiguous
    ));
    out.push_str(&format!(
        "  Symbols   +{} -{} ~{} ({} rows whose key repeats)\n",
        result.counts.symbols_added,
        result.counts.symbols_removed,
        result.counts.symbols_changed,
        result.counts.symbols_ambiguous
    ));
    out.push_str(&format!(
        "  Identical, so left out of the tables: {} section(s), {} symbol(s)\n",
        result.unchanged.sections, result.unchanged.symbols
    ));

    push_contributors(
        &mut out,
        "Top section growth (Changed rows)",
        &result.top_section_growth(5),
    );
    push_contributors(
        &mut out,
        "Largest added sections (absence, never 0)",
        &result.largest_added_sections(5),
    );
    push_contributors(
        &mut out,
        "Top symbol growth (Changed rows)",
        &result.top_symbol_growth(5),
    );
    push_contributors(
        &mut out,
        "Largest added symbols (absence, never 0)",
        &result.largest_added_symbols(5),
    );

    if !result.attribution.available {
        out.push_str("\nObject / module attribution\n  unavailable\n");
    }

    if !result.warnings.is_empty() {
        out.push_str("\nWarnings\n");
        for warning in &result.warnings {
            out.push_str(&format!("  [{}] {}\n", warning.code, warning.message));
        }
    }

    out
}

fn push_contributors(out: &mut String, title: &str, rows: &[Contributor]) {
    out.push_str(&format!("\n{title}\n"));
    if rows.is_empty() {
        out.push_str("  none\n");
        return;
    }
    for row in rows {
        let delta = match row.delta {
            Some(value) if value > 0 => format!("+{value}"),
            Some(value) => format!("{value}"),
            None => "-".to_owned(),
        };
        let bytes = row
            .bytes
            .map(|value| format!("{value} bytes"))
            .unwrap_or_else(|| "-".to_owned());
        out.push_str(&format!(
            "  {delta:>9}  {:<28} {bytes}  [{}]\n",
            row.key,
            row.change.label()
        ));
    }
}

fn short_sha(sha: &str) -> &str {
    &sha[..12.min(sha.len())]
}

fn bytes_or_absence(bytes: Option<u64>) -> String {
    match bytes {
        Some(value) => format!("{value} bytes"),
        None => "\u{2014}".to_owned(),
    }
}

fn human_change(change: &ByteChange) -> String {
    let base = bytes_or_absence(change.base);
    let target = bytes_or_absence(change.target);
    let delta = match change.delta {
        Some(value) if value > 0 => format!("+{value}"),
        Some(value) => format!("{value}"),
        None => "\u{2014}".to_owned(),
    };
    let mut line = format!(
        "{base} \u{2192} {target}   delta {delta} [{}]",
        change.comparability.label()
    );
    if let Some(reason) = &change.reason {
        line.push_str(&format!(" ({reason})"));
    }
    line
}

/// Escape text that came from an artifact: section and symbol names, reasons, file names.
///
/// A firmware build can contain `<`, `&` and quote characters in a symbol name, and an exported
/// report is read in a browser. Escaping at the point of output means a name cannot inject markup.
#[must_use]
pub fn escape_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}
