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

use firmwaresight_core::domain::diff::{
    ByteChange, ChangeKind, Contributor, DiffResult, ObjectChange,
};

use crate::diff::{ByteChangeDto, DiffResultDto, ObjectChangesDto};

/// Every colour, size and radius below is a value from `assets/design-tokens.json` (0.4.0, light
/// theme), restated here because a standalone file cannot read the desktop's `tokens.css`.
/// `html_export_colours_come_from_the_frozen_token_set` re-checks that against the token file, so
/// this block cannot quietly drift into magic values.
pub(crate) const EMBEDDED_STYLE: &str = r#":root {
  --fs-color-bg-canvas: #F7F8FA;
  --fs-color-bg-surface: #FFFFFF;
  --fs-color-bg-subtle: #F1F3F5;
  --fs-color-text-primary: #171A1F;
  --fs-color-text-secondary: #5D6673;
  --fs-color-border-default: #D9DEE5;
  --fs-color-diff-added: #18794E;
  --fs-color-diff-added-bg: #E7F3EC;
  --fs-color-diff-removed: #C7372C;
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

/* R6: a delta is a fact, not a judgement. It is primary text with a glyph, and the words for a screen
   reader; only a policy's state ever carries colour (ADR-0034 D1.4). */
.delta {
  font-variant-numeric: tabular-nums;
}

.spoken {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
}

/* The report grammar (R6): a masthead that names the report and its subject, then the answer. */
.masthead {
  padding-bottom: var(--fs-space-3);
  border-bottom: var(--fs-border-width-hairline) solid var(--fs-color-border-default);
}

.masthead .kind {
  margin: 0 0 var(--fs-space-2);
  color: var(--fs-color-text-secondary);
  font-size: var(--fs-font-size-metadata);
  line-height: 18px;
}

.lead {
  margin: var(--fs-space-4) 0;
  font-size: var(--fs-font-size-section);
  line-height: 22px;
}

.lead strong {
  font-family: var(--fs-font-mono);
  font-variant-numeric: tabular-nums;
}

.limits {
  margin: 0;
  padding-left: var(--fs-space-4);
  color: var(--fs-color-text-secondary);
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
@page { size: A4; margin: 16mm; }
@media print {
  body { padding: 0; background: #FFFFFF; }
  main { max-width: none; }
  .visual-summary { break-inside: avoid; }
  table { width: 100%; table-layout: fixed; }
  thead { display: table-header-group; }
  tr { break-inside: avoid; }
  h1, h2, h3 { break-after: avoid; }
}
th, td, .mono { overflow-wrap: anywhere; }
.visual-scene { display: block; width: 100%; height: auto; }
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
    out.push_str("<title>FirmwareSight comparison report</title>\n");
    out.push_str("<style>\n");
    out.push_str(EMBEDDED_STYLE);
    out.push_str("</style>\n</head>\n<body>\n<main>\n");

    // R6 report grammar: the masthead names the report and the pair, and the answer comes before any table.
    out.push_str("<header class=\"masthead\">\n<p class=\"kind\">FirmwareSight &middot; Comparison report</p>\n");
    out.push_str(&format!(
        "<h1>{} <span class=\"mono\">{}</span> &rarr; {} <span class=\"mono\">{}</span></h1>\n",
        escape_html(&dto.base.artifact.file_name),
        escape_html(short_sha(&dto.base.artifact.sha256)),
        escape_html(&dto.target.artifact.file_name),
        escape_html(short_sha(&dto.target.artifact.sha256)),
    ));
    out.push_str(&format!(
        "<p class=\"brand\">FirmwareSight {} &middot; <span class=\"mono\">{}</span> schema version {}</p>\n</header>\n",
        env!("CARGO_PKG_VERSION"),
        escape_html(dto.schema),
        dto.schema_version
    ));
    out.push_str(&format!(
        "<p class=\"lead\">Nonvolatile / load image {}; runtime RAM {}. Comparability: {}.</p>\n",
        lead_change(&dto.memory.nonvolatile),
        lead_change(&dto.memory.runtime_ram),
        escape_html(dto.memory.comparability)
    ));

    render_sides(dto, &mut out);
    out.push_str(&crate::visual::summary_html(
        &crate::visual::VisualReport::diff(dto),
    ));
    render_memory(dto, &mut out);
    render_warnings(dto, &mut out);
    render_counts(dto, &mut out);
    render_sections(dto, &mut out);
    render_symbols(dto, &mut out);
    render_objects(&dto.object_changes, &mut out);

    out.push_str(
        "<h2>Limits</h2>\n<ul class=\"limits\">\n\
<li>A dash is an absence, never a zero: a side that does not exist has no size and no delta.</li>\n\
<li>A row whose name repeats is left unpaired and is never counted as a change.</li>\n\
<li>Every delta, comparability and ranking here is Core&rsquo;s; this report decides nothing.</li>\n\
</ul>\n",
    );
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
    out.push_str("<thead><tr><th scope=\"col\">Budget</th><th scope=\"col\" class=\"num\">Base</th><th scope=\"col\" class=\"num\">Target</th><th scope=\"col\" class=\"num\">Delta</th><th scope=\"col\">Reading</th></tr></thead>\n<tbody>\n");
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
    let mut row = format!("<tr><th scope=\"row\">{name}</th>");
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
        None => "<td class=\"num unknown\" aria-label=\"no delta\">\u{2014}</td>".to_owned(),
        Some(value) => format!("<td class=\"num delta\">{}</td>", delta_words(value)),
    }
}

/// A delta as neutral text: a glyph, the magnitude, and the words for a screen reader (R6).
fn delta_words(value: i64) -> String {
    match value {
        0 => "0".to_owned(),
        v if v > 0 => format!(
            "<span aria-hidden=\"true\">\u{25B2}{v}</span><span class=\"spoken\">increase of {v}</span>"
        ),
        v => format!(
            "<span aria-hidden=\"true\">\u{25BC}{m}</span><span class=\"spoken\">decrease of {m}</span>",
            m = v.unsigned_abs()
        ),
    }
}

/// One side of the lead: the delta in figure type, and what it moved between.
fn lead_change(change: &ByteChangeDto) -> String {
    let delta = match change.delta {
        Some(value) => format!("<strong>{} bytes</strong>", delta_words(value)),
        None => "<strong>no delta</strong>".to_owned(),
    };
    let side = |bytes: Option<u64>| bytes.map_or_else(|| "\u{2014}".to_owned(), |b| b.to_string());
    format!(
        "{delta} ({} &rarr; {} bytes)",
        side(change.base),
        side(change.target)
    )
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
    out.push_str("<thead><tr><th scope=\"col\">Change</th><th scope=\"col\">Section</th><th scope=\"col\" class=\"num\">Base size</th><th scope=\"col\" class=\"num\">Target size</th><th scope=\"col\" class=\"num\">Delta</th><th scope=\"col\">Fields</th><th scope=\"col\">Where to check</th></tr></thead>\n<tbody>\n");
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
    out.push_str("<thead><tr><th scope=\"col\">Change</th><th scope=\"col\">Symbol</th><th scope=\"col\" class=\"num\">Base size</th><th scope=\"col\" class=\"num\">Target size</th><th scope=\"col\" class=\"num\">Delta</th><th scope=\"col\">Fields</th><th scope=\"col\">Where to check</th></tr></thead>\n<tbody>\n");
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
        out.push_str(&format!(
            "<p class=\"meta\">{}</p>\n",
            escape_html(&attribution.reason)
        ));
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
    use crate::text::{heading, masthead, wrap};

    // R6 report grammar: masthead, the answer, the subject, then the facts.
    let mut out = masthead("Comparison report");
    out.push_str(&wrap(
        &format!(
            "Nonvolatile / load image {}; runtime RAM {}.",
            human_change(&result.memory.nonvolatile),
            human_change(&result.memory.runtime_ram)
        ),
        "",
        "  ",
    ));
    out.push_str(&heading("Subject"));
    out.push_str(&format!(
        "  Base    {}  {}\n  Target  {}  {}\n",
        terminal_safe(&result.base_artifact.file_name),
        short_sha(&result.base_artifact.sha256),
        terminal_safe(&result.target_artifact.file_name),
        short_sha(&result.target_artifact.sha256),
    ));

    out.push_str(&heading("Memory"));
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
        "  Target side states:       nonvolatile {} / RAM {}\n",
        result.memory.target.nonvolatile.state.as_str(),
        result.memory.target.runtime_ram.state.as_str()
    ));

    out.push_str(&heading("Counts"));
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

    push_objects(&mut out, result);

    if !result.warnings.is_empty() {
        out.push_str(&heading("Warnings"));
        for warning in &result.warnings {
            out.push_str(&format!(
                "  [{}] {}\n",
                warning.code,
                terminal_safe(&warning.message)
            ));
        }
    }

    out
}

/// ADR-0033 D8: the per-object and per-library delta, or the one line that says why there is none.
fn push_objects(out: &mut String, result: &DiffResult) {
    let attribution = &result.attribution;
    if !attribution.available {
        out.push_str(&format!(
            "\nObject / library attribution\n  unavailable: {}\n",
            attribution.reason
        ));
        return;
    }
    let tally = |rows: &[ObjectChange], unchanged: usize| {
        let count = |kind: ChangeKind| rows.iter().filter(|row| row.change == kind).count();
        format!(
            "{} changed, {} added, {} removed, {unchanged} unchanged",
            count(ChangeKind::Changed),
            count(ChangeKind::Added),
            count(ChangeKind::Removed)
        )
    };
    out.push_str("\nObject / library attribution (GNU ld MAP rows)\n");
    let unpaired = attribution
        .objects
        .iter()
        .chain(&attribution.libraries)
        .filter(|row| row.ambiguous)
        .count();
    out.push_str(&format!(
        "  Objects    {}\n  Libraries  {}\n",
        tally(&attribution.objects, attribution.unchanged_objects),
        tally(&attribution.libraries, attribution.unchanged_libraries)
    ));
    if unpaired > 0 {
        out.push_str(&format!(
            "  {unpaired} row(s) repeat a name in a build (main.o, main.o#2) and were left unpaired rather than matched by link order\n"
        ));
    }
    push_contributors(
        out,
        "Top object growth, Flash (Changed rows)",
        &result.top_object_growth(false, 5),
    );
    push_contributors(
        out,
        "Largest added objects, Flash (absence, never 0)",
        &result.largest_added_objects(false, 5),
    );
    push_contributors(
        out,
        "Top library growth, Flash (Changed rows)",
        &result.top_object_growth(true, 5),
    );
}

fn push_contributors(out: &mut String, title: &str, rows: &[Contributor]) {
    out.push_str(&crate::text::heading(title));
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
            terminal_safe(&row.key),
            row.change.label()
        ));
    }
}

/// D2C C1: a name read from a user's ELF or MAP may carry characters that drive or mislead a terminal.
/// One place for every value the human report takes from a file: artifact file names, section, symbol,
/// object and library keys, and warning text. Escaped:
///
/// - everything Rust's `char::escape_debug` treats as non-printable: C0/C1 controls and DEL, every
///   bidi control and mark (U+061C, U+200E/F, U+202A–E, U+2066–9), and the zero-width, soft-hyphen and
///   line-separator format characters (U+00AD, U+180E, U+200B–D, U+2028/9, U+FEFF…). A name therefore
///   cannot move the cursor, recolour the screen, or reorder or hide the text around it (review S2-1 of
///   `4cbdf74` and R1 of `dc58c21`). This is the same table `analyze --objects` uses, so the two reports
///   escape the same characters;
/// - the backslash, so the literal text `\u{1b}` in a name cannot pass for an escaped ESC.
///
/// Quotes print as they are (`escape_debug` alone would escape them), and so do spaces, CJK and accents.
pub(crate) fn terminal_safe(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        if c == '"' || c == '\'' {
            out.push(c);
        } else {
            out.extend(c.escape_debug());
        }
    }
    out
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

pub(crate) fn human_change(change: &ByteChange) -> String {
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

#[cfg(test)]
mod tests {
    use super::terminal_safe;

    #[test]
    fn control_characters_are_escaped_and_printable_text_is_left_alone() {
        assert_eq!(terminal_safe("evil\u{1b}[2J.elf"), "evil\\u{1b}[2J.elf");
        assert_eq!(terminal_safe("a\nb\tc\u{7f}"), "a\\nb\\tc\\u{7f}");
        assert_eq!(
            terminal_safe("\u{9b}31m"),
            "\\u{9b}31m",
            "a C1 CSI is a control too"
        );
        assert_eq!(
            terminal_safe(".text \"quoted\" 'x'"),
            ".text \"quoted\" 'x'"
        );
        assert_eq!(terminal_safe("固件 main.o"), "固件 main.o");
    }

    #[test]
    fn bidi_and_invisible_characters_and_the_backslash_are_escaped() {
        // Review S2-1 of 4cbdf74.
        assert_eq!(terminal_safe("lib\u{202e}o.a"), r"lib\u{202e}o.a");
        assert_eq!(terminal_safe("a\u{200b}b\u{feff}"), r"a\u{200b}b\u{feff}");
        assert_eq!(terminal_safe("x\u{2066}y\u{2069}"), r"x\u{2066}y\u{2069}");
        assert_eq!(terminal_safe("p\u{2028}q"), r"p\u{2028}q");
        // Re-review R1 of dc58c21: the Arabic letter mark, the soft hyphen and the Mongolian vowel
        // separator are format characters too.
        assert_eq!(terminal_safe("a\u{61c}b"), r"a\u{61c}b");
        assert_eq!(terminal_safe("soft\u{ad}hy"), r"soft\u{ad}hy");
        assert_eq!(terminal_safe("m\u{180e}n"), r"m\u{180e}n");
        assert_eq!(
            terminal_safe(r"fake\u{1b}"),
            r"fake\\u{1b}",
            "a literal backslash cannot pass for an escaped ESC"
        );
    }
}
