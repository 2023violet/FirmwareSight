//! Rendering of a Release Bundle's generated content: the manifest JSON, the `SHA256SUMS` text and the
//! self-contained `release-report.html`.
//!
//! Three renderings, one source, and none of them decides anything. Every state word, delta, count and
//! capability judgement in these outputs was already in a DTO that projected it from Core (`AGENTS.md` 3).
//! The ranking rows in the compare summary come from Core's own selectors for the same reason — a report
//! that sorted contributors itself would be a second definition of "major growth" that could disagree with
//! the CLI that printed the first one.
//!
//! The HTML is a single file with embedded CSS and no script: no CDN, no remote stylesheet, no font fetch,
//! no telemetry. It carries no generation timestamp either (prompt §42), so it is byte-stable, which makes
//! the golden in `golden/reports/` a real check. The one timestamp it does print is an `accepted_at` from a
//! stored review record — an audit fact that already lives in the underlying documents, which §25 allows.
//!
//! Wording is constrained twice over. §43: the FirmwareSight app version, the project release version, an
//! artifact digest and a workspace Git fact are four different things and are labelled as four different
//! things; a Git HEAD is "observed by the Release Gate", never "the commit the firmware was built from".
//! §62: nothing here calls a bundle trusted, authentic, signed or tamper-proof. Two layers of SHA-256
//! indexes give *hash verification* and *bundle consistency*, which is a weaker and true claim: it tells a
//! reader the bytes match the list, and nothing about who wrote the list.

use firmwaresight_core::domain::diff::Contributor;
use firmwaresight_core::domain::gate::{FindingState, GateStateCounts};
use firmwaresight_core::domain::release::{
    ACCEPTED_REVIEWS_DOC_NAME, ARTIFACTS_DIR, DIFF_DOC_NAME, MANIFEST_DOC_NAME, REPORT_DOC_NAME,
    ReleaseModel, SHA256SUMS_NAME, SUMS_EXCLUDED_NAMES, bundle_path_order,
};

use crate::analysis::AnalysisDocumentDto;
use crate::diff::{ByteChangeDto, DiffResultDto};
use crate::diff_render::{EMBEDDED_STYLE, escape_html};
use crate::dto::{CapabilitiesDto, EvidenceDto};
use crate::gate::{AcceptedReviewsDto, GateResultsDto};
use crate::gate_render::STATE_ORDER;
use crate::release::{ManifestFileDto, ReleaseManifestDto, version_source_word};

/// The additions the release report needs on top of the shared token block: the five Gate states, each
/// with a word and a glyph as well as a colour (`AGENTS.md` 11 forbids a colour-only state), and the two
/// sub-heading levels the nine sections need.
///
/// Every value below is a frozen `assets/design-tokens.json` (0.2.1, light theme) value restated as a
/// variable, and the three `status.*` names are the ones the diff export had no use for — `status.review`
/// (#9A6700) in particular has no equivalent anywhere in the shared block, so declaring it here is what
/// makes the REVIEW chip a colour rather than an unset custom property. No new token is minted, and
/// `the_release_report_colours_come_from_the_frozen_token_set` re-checks every hex against the token file.
const RELEASE_EXTRA_STYLE: &str = r#":root {
  --fs-color-status-pass: #18794E;
  --fs-color-status-review: #9A6700;
  --fs-color-status-block: #C9372C;
}

.state {
  font-weight: 600;
}

.chip.pass {
  color: var(--fs-color-status-pass);
  background: var(--fs-color-diff-added-bg);
}

.chip.review {
  color: var(--fs-color-status-review);
  background: var(--fs-color-bg-subtle);
}

.chip.block {
  color: var(--fs-color-status-block);
  background: var(--fs-color-diff-removed-bg);
}

.chip.unknown {
  color: var(--fs-color-status-unknown);
  background: var(--fs-color-bg-subtle);
}

.chip.na {
  color: var(--fs-color-text-secondary);
  background: var(--fs-color-bg-subtle);
}

h3 {
  margin: var(--fs-space-4) 0 var(--fs-space-2);
  font-size: var(--fs-font-size-body);
  line-height: 20px;
  font-weight: 600;
}

h4 {
  margin: var(--fs-space-3) 0 var(--fs-space-2);
  font-size: var(--fs-font-size-metadata);
  line-height: 18px;
  font-weight: 600;
  color: var(--fs-color-text-secondary);
}

.steps {
  margin: var(--fs-space-2) 0 0;
  padding-left: var(--fs-space-4);
}

.steps li {
  margin: 0 0 var(--fs-space-2);
}
"#;

/// The `--json` contract for `release-manifest.json`: exactly one JSON document, one trailing newline.
#[must_use]
pub fn render_json(manifest: &ReleaseManifestDto) -> String {
    crate::render::document_json(manifest)
}

/// The `SHA256SUMS` text (§21): `<64 hex><two spaces><relative/path>`, LF endings, sorted
/// case-insensitively by bundle-relative path, carrying every payload file except itself and the manifest.
///
/// The exclusions are applied here rather than left to the caller, because the file that lists what a
/// bundle contains must not be able to list itself. A caller that passes the full `files` list gets the
/// correct subset; a caller that pre-filtered gets the same bytes.
#[must_use]
pub fn render_sums(entries: &[ManifestFileDto]) -> String {
    let mut lines: Vec<&ManifestFileDto> = entries
        .iter()
        .filter(|entry| !SUMS_EXCLUDED_NAMES.contains(&entry.path.as_str()))
        .collect();
    lines.sort_by(|left, right| bundle_path_order(&left.path, &right.path));

    let mut out = String::new();
    for entry in lines {
        // Two spaces, not one: that is the text-mode form `sha256sum -c` parses, and the binary-mode
        // spelling of the same tool (` *path`) is deliberately not used here.
        out.push_str(&format!("{}  {}\n", entry.sha256, entry.path));
    }
    out
}

/// The compare section's inputs, bundled so a report cannot have a baseline and no ranking, or a ranking
/// and no baseline.
#[derive(Debug, Clone, Copy)]
pub struct CompareSummary<'a> {
    /// The document that ships beside this report as `diff.json`.
    pub result: &'a DiffResultDto,
    /// `DiffResult::top_section_growth`, taken from Core. Not re-sorted and not truncated again here.
    pub top_sections: &'a [Contributor],
    pub top_symbols: &'a [Contributor],
}

/// How many growth rows the report prints per side. The count is this crate's presentation bound; the
/// *selection* is Core's, which is why [`CompareSummary`] carries rows rather than a `DiffResult` to sort.
pub const REPORT_GROWTH_ROWS: usize = 5;

/// Everything `release-report.html` renders, as the portable documents that ship beside it.
///
/// There is no `manifest` field, and that is the self-reference rule at work rather than an oversight:
/// the report is payload, so `SHA256SUMS` and `release-manifest.json` are both written *after* it (§41)
/// and neither of their digests can be known while it is composed. The identity facts come from the
/// validated `ReleaseModel` and the digest that names them, and `staged` carries the files already
/// written — every one of which the report can honestly print a hash for.
#[derive(Debug, Clone, Copy)]
pub struct ReleaseBundleReport<'a> {
    /// The semantic inputs of this release, as Core validated them.
    pub model: &'a ReleaseModel,
    /// `release-<sha256>`: the digest of the canonical release input.
    pub release_id: &'a str,
    /// The semantic policy fingerprint the Gate run was identified by.
    pub policy_sha256: &'a str,
    /// Every bundle file written before this report, with the digest and size of the bytes as written.
    pub staged: &'a [ManifestFileDto],
    pub analysis: &'a AnalysisDocumentDto,
    /// `None` when this release has no baseline: the report then says there is no comparison, which is a
    /// fact about the release rather than an empty table.
    pub compare: Option<CompareSummary<'a>>,
    /// Core's per-state counts for the run this bundle is the output of. Never recounted here.
    pub gate_counts: GateStateCounts,
    pub gate: &'a GateResultsDto,
    pub reviews: &'a AcceptedReviewsDto,
}

/// The report a reader double-clicks (§25). The sections follow the prompt's order and numbering so a
/// reviewer checking acceptance can point at one.
#[must_use]
pub fn render_html(report: &ReleaseBundleReport<'_>) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("<meta charset=\"utf-8\">\n");
    out.push_str("<title>FirmwareSight Release Bundle</title>\n");
    out.push_str("<style>\n");
    out.push_str(EMBEDDED_STYLE);
    out.push_str(RELEASE_EXTRA_STYLE);
    out.push_str("</style>\n</head>\n<body>\n<main>\n");

    out.push_str("<h1>Release Bundle</h1>\n");
    out.push_str(&format!(
        "<p class=\"brand\">FirmwareSight {} &middot; release <span class=\"mono\">{}</span></p>\n",
        escape_html(&report.model.fwsight_version),
        escape_html(report.release_id)
    ));

    render_identity(report, &mut out);
    render_shipped(report, &mut out);
    render_analysis(report, &mut out);
    render_compare(report, &mut out);
    render_gate(report, &mut out);
    render_reviews(report, &mut out);
    render_notes(report, &mut out);
    render_integrity(report, &mut out);
    render_limits(report, &mut out);

    out.push_str(&format!(
        "<footer>Deterministic export: no timestamp, no scripts, no network requests. \
Written by FirmwareSight {} over the documents in this bundle.</footer>\n",
        escape_html(&report.model.fwsight_version)
    ));
    out.push_str("</main>\n</body>\n</html>\n");
    out
}

// --------------------------------------------------------------------------- 1. identity

fn render_identity(report: &ReleaseBundleReport<'_>, out: &mut String) {
    let model = report.model;
    out.push_str("<h2>1. Release identity</h2>\n<table>\n");
    out.push_str(
        "<caption>Four different identities, named separately on purpose.</caption>\n<tbody>\n",
    );
    mono_row(out, "Release id", report.release_id);
    row(
        out,
        "Project release version",
        format!(
            "<span class=\"mono\">{}</span> &mdash; resolved from <span class=\"mono\">{}</span>",
            escape_html(model.version.as_str()),
            escape_html(version_source_word(model.version_source))
        ),
    );
    mono_row(out, "FirmwareSight app version", &model.fwsight_version);
    mono_row(out, "Snapshot analyzed", &model.snapshot_id);
    mono_row(
        out,
        "Analysis normalization rule",
        report.analysis.snapshot.normalization_version,
    );
    let git = &model.workspace_git;
    row(
        out,
        "Workspace HEAD observed by the Release Gate",
        git.head_commit
            .as_deref()
            .map(mono)
            .unwrap_or_else(not_observed),
    );
    row(
        out,
        "Workspace tag observed by the Release Gate",
        git.exact_tag
            .as_deref()
            .map(mono)
            .unwrap_or_else(not_observed),
    );
    row(
        out,
        "Workspace dirty observed by the Release Gate",
        git.dirty
            .map_or_else(not_observed, |value| value.to_string()),
    );
    out.push_str(
        "<tr><td colspan=\"2\" class=\"meta\">These Git facts describe the workspace the Release Gate ran \
         in. They are not evidence about how the shipped bytes were compiled, and no claim of that kind is \
         made here.</td></tr>\n",
    );
    out.push_str("</tbody>\n</table>\n");
}

// --------------------------------------------------------------------------- 2. shipped artifacts

fn render_shipped(report: &ReleaseBundleReport<'_>, out: &mut String) {
    out.push_str("<h2>2. Shipped artifacts</h2>\n");
    out.push_str(&format!(
        "<p class=\"meta\">{} file(s) under <span class=\"mono\">{ARTIFACTS_DIR}/</span>. The baseline \
         build is never shipped: it exists only to explain the comparison in section 4.</p>\n",
        report.model.artifacts.len()
    ));
    out.push_str(
        "<table>\n<caption>The bytes this release consists of, in the order the release fingerprint \
         lists them.</caption>\n",
    );
    out.push_str("<thead><tr><th>Bundle path</th><th>Kind</th><th>SHA-256</th><th class=\"num\">Bytes</th></tr></thead>\n<tbody>\n");
    for artifact in &report.model.artifacts {
        out.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td>{}</td><td class=\"mono\">{}</td><td class=\"num\">{}</td></tr>\n",
            escape_html(&format!("{ARTIFACTS_DIR}/{}", artifact.file_name)),
            escape_html(artifact.kind_word_for()),
            escape_html(artifact.sha256.hex()),
            artifact.byte_size
        ));
    }
    out.push_str("</tbody>\n</table>\n");
}

// --------------------------------------------------------------------------- 3. analysis

fn render_analysis(report: &ReleaseBundleReport<'_>, out: &mut String) {
    let analysis = report.analysis;
    out.push_str("<h2>3. Analysis summary</h2>\n");
    out.push_str("<table>\n<caption>What was read off the build, per input file.</caption>\n");
    out.push_str("<thead><tr><th>File</th><th>Format</th><th>Entry point</th><th>Build id</th></tr></thead>\n<tbody>\n");
    for artifact in &analysis.artifacts {
        out.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td>{} {}-bit {} &middot; <span class=\"mono\">{}</span></td><td>{}</td><td>{}</td></tr>\n",
            escape_html(&artifact.file_name),
            escape_html(&artifact.architecture),
            escape_html(artifact.bitness),
            escape_html(artifact.endianness),
            escape_html(artifact.kind),
            known_or_reason(&artifact.entry_point, &artifact.entry_point_unknown_reason),
            known_or_reason(&artifact.build_id, &artifact.build_id_unknown_reason)
        ));
    }
    out.push_str("</tbody>\n</table>\n");

    let memory = &analysis.memory;
    out.push_str("<h3>Memory budgets</h3>\n<table>\n");
    out.push_str(
        "<caption>Absence is printed as absence. A budget that is not a number was not measured, \
                  and is never shown as zero.</caption>\n<tbody>\n",
    );
    budget_row(
        out,
        "Nonvolatile / load image",
        memory.nonvolatile_image_footprint.state,
        memory.nonvolatile_image_footprint.bytes,
        memory.nonvolatile_image_footprint.reason.as_deref(),
    );
    budget_row(
        out,
        "Runtime RAM",
        memory.runtime_ram_footprint.state,
        memory.runtime_ram_footprint.bytes,
        memory.runtime_ram_footprint.reason.as_deref(),
    );
    mono_row(out, "Accounting rule", memory.accounting_rule);
    mono_row(out, "Layout source", memory.layout_source);
    row(
        out,
        "Weakest evidence basis",
        memory
            .weakest_evidence_basis
            .map(escape_html)
            .unwrap_or_else(|| "no basis recorded".to_owned()),
    );
    row(
        out,
        "Admissible for a hard block",
        memory.admissible_for_hard_block.to_string(),
    );
    row(
        out,
        "Device metadata held out of both budgets",
        format!("{} bytes", memory.excluded_metadata_bytes),
    );
    out.push_str("</tbody>\n</table>\n");

    out.push_str("<h3>Coverage of this build</h3>\n<table>\n<tbody>\n");
    row(
        out,
        "Rows carried in the analysis document",
        format!(
            "{} section(s), {} symbol(s), {} evidence record(s)",
            analysis.sections.len(),
            analysis.symbols.len(),
            analysis.evidence.len()
        ),
    );
    for (label, value) in capability_rows(&analysis.capabilities) {
        mono_row(out, label, value);
    }
    out.push_str("</tbody>\n</table>\n");
}

// --------------------------------------------------------------------------- 4. compare

fn render_compare(report: &ReleaseBundleReport<'_>, out: &mut String) {
    out.push_str("<h2>4. Compare summary</h2>\n");
    let Some(compare) = report.compare else {
        out.push_str(
            "<p class=\"empty\">No baseline build was compared in this release, so there is no \
             <span class=\"mono\">diff.json</span> in the bundle and nothing here to rank. That is a \
             statement about the release, not a missing file.</p>\n",
        );
        return;
    };
    let dto = compare.result;
    out.push_str(
        "<table>\n<caption>Totals moved between the two builds. The full row-by-row comparison is \
                  in <span class=\"mono\">diff.json</span>.</caption>\n<tbody>\n",
    );
    mono_row(out, "Base (old) snapshot", &dto.base.snapshot_id);
    mono_row(out, "Target (new) snapshot", &dto.target.snapshot_id);
    delta_row(out, "Nonvolatile / load image", &dto.memory.nonvolatile);
    delta_row(out, "Runtime RAM", &dto.memory.runtime_ram);
    row(
        out,
        "Comparability of the pair",
        escape_html(dto.memory.comparability),
    );
    row(
        out,
        "Rows that differ",
        format!(
            "sections: {} added, {} removed, {} changed, {} unpaired &middot; symbols: {} added, \
             {} removed, {} changed, {} unpaired",
            dto.counts.sections_added,
            dto.counts.sections_removed,
            dto.counts.sections_changed,
            dto.counts.sections_ambiguous,
            dto.counts.symbols_added,
            dto.counts.symbols_removed,
            dto.counts.symbols_changed,
            dto.counts.symbols_ambiguous
        ),
    );
    row(
        out,
        "Identical, so left out of the tables",
        format!(
            "{} section(s), {} symbol(s)",
            dto.unchanged.sections, dto.unchanged.symbols
        ),
    );
    out.push_str("</tbody>\n</table>\n");

    out.push_str("<h3>Top growth, as Core ranked it</h3>\n");
    growth_table(out, "section", compare.top_sections);
    growth_table(out, "symbol", compare.top_symbols);

    out.push_str(&format!(
        "<p class=\"meta\">The complete comparison ships as <span class=\"mono\">{DIFF_DOC_NAME}</span> \
         under schema <span class=\"mono\">{}</span> version {}.</p>\n",
        escape_html(dto.schema),
        dto.schema_version
    ));
}

fn growth_table(out: &mut String, noun: &str, rows: &[Contributor]) {
    out.push_str(&format!("<h4>Top {noun} growth</h4>\n"));
    if rows.is_empty() {
        out.push_str(&format!(
            "<p class=\"empty\">No {noun} row exists on both sides with a decidable growth.</p>\n"
        ));
        return;
    }
    out.push_str("<table>\n<thead><tr><th class=\"num\">Delta</th><th>Row</th><th class=\"num\">Bytes</th><th>Change</th></tr></thead>\n<tbody>\n");
    for entry in rows {
        out.push_str(&format!(
            "<tr><td class=\"num mono\">{}</td><td class=\"mono\">{}</td><td class=\"num\">{}</td><td>{}</td></tr>\n",
            signed(entry.delta),
            escape_html(&entry.key),
            entry
                .bytes
                .map_or_else(|| "\u{2014}".to_owned(), |value| value.to_string()),
            escape_html(entry.change.label())
        ));
    }
    out.push_str("</tbody>\n</table>\n");
}

// --------------------------------------------------------------------------- 5. gate

fn render_gate(report: &ReleaseBundleReport<'_>, out: &mut String) {
    let gate = report.gate;
    out.push_str("<h2>5. Release Gate</h2>\n");
    out.push_str(&format!(
        "<p class=\"meta\">{} &mdash; the disposition that made this bundle possible. A bundle is written \
         only for a <span class=\"state\">PASS</span> after accepted reviews; a <span class=\"state\">REVIEW</span> \
         or a <span class=\"state\">BLOCK</span> produces no release, and an evidence gap never becomes a \
         review.</p>\n",
        chip(report.reviews.extensions.overall_effective_severity)
    ));
    out.push_str("<table>\n<tbody>\n");
    mono_row(out, "Run id", &gate.run_id);
    mono_row(out, "Snapshot", &gate.snapshot_id);
    row(
        out,
        "Baseline",
        gate.extensions
            .baseline_snapshot_id
            .as_deref()
            .map(mono)
            .unwrap_or_else(|| "none (this run carried no comparison)".to_owned()),
    );
    mono_row(out, "Policy digest", &gate.extensions.policy_sha256);
    mono_row(
        out,
        "Project config schema",
        gate.extensions.project_config_schema_version.to_string(),
    );
    row(
        out,
        "Severity before reviews",
        chip(gate.extensions.overall_effective_severity),
    );
    row(
        out,
        "Severity after accepted reviews",
        chip(report.reviews.extensions.overall_effective_severity),
    );
    out.push_str("</tbody>\n</table>\n");

    out.push_str("<h3>Findings by state</h3>\n<table>\n");
    out.push_str("<caption>Core's counts for this run, carried rather than recounted.</caption>\n");
    out.push_str("<thead><tr><th>State</th><th class=\"num\">Count</th></tr></thead>\n<tbody>\n");
    for state in STATE_ORDER {
        let count = match state {
            FindingState::Pass => report.gate_counts.pass,
            FindingState::Review => report.gate_counts.review,
            FindingState::Block => report.gate_counts.block,
            FindingState::Unknown => report.gate_counts.unknown,
            FindingState::NotApplicable => report.gate_counts.not_applicable,
        };
        out.push_str(&format!(
            "<tr><td>{}</td><td class=\"num\">{count}</td></tr>\n",
            chip(state.as_str())
        ));
    }
    out.push_str("</tbody>\n</table>\n");

    out.push_str("<h3>All findings</h3>\n<table>\n");
    out.push_str("<caption>In the order a release owner reads them: what blocks, what needs a decision, \
                  what is unknown, what passed. Every finding of the run is listed, not a truncated top \
                  list.</caption>\n");
    out.push_str("<thead><tr><th>Rule</th><th>State</th><th>Effective severity</th><th>Finding</th><th>Evidence</th><th>Next step</th></tr></thead>\n<tbody>\n");
    for state in STATE_ORDER {
        for finding in gate
            .findings
            .iter()
            .filter(|finding| finding.state == state.as_str())
        {
            let severity = if finding.effective_severity == finding.state {
                "<span class=\"meta\">same as state</span>".to_owned()
            } else {
                chip(finding.effective_severity)
            };
            out.push_str(&format!(
                "<tr><td class=\"mono\">{}</td><td>{}</td><td>{severity}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                escape_html(&finding.rule_id),
                chip(finding.state),
                escape_html(&finding.summary),
                evidence_refs(&finding.evidence_refs),
                finding
                    .remediation
                    .as_deref()
                    .map(escape_html)
                    .unwrap_or_else(|| "none recorded".to_owned())
            ));
        }
    }
    out.push_str("</tbody>\n</table>\n");
}

fn evidence_refs(refs: &[String]) -> String {
    if refs.is_empty() {
        return "no refs recorded".to_owned();
    }
    refs.iter()
        .map(|reference| format!("<span class=\"locator\">{}</span>", escape_html(reference)))
        .collect::<Vec<_>>()
        .join(" &middot; ")
}

/// The five Gate states as glyph plus word (`AGENTS.md` 11), never colour alone.
fn chip(state: &str) -> String {
    let (class, glyph) = match state {
        "PASS" => ("pass", "\u{2713}"),
        "REVIEW" => ("review", "\u{25b2}"),
        "BLOCK" => ("block", "\u{2715}"),
        "UNKNOWN" => ("unknown", "?"),
        "N/A" => ("na", "\u{2013}"),
        other => return escape_html(other),
    };
    format!(
        "<span class=\"chip {class}\">{glyph} <span class=\"state\">{}</span></span>",
        escape_html(state)
    )
}

// --------------------------------------------------------------------------- 6. reviews

fn render_reviews(report: &ReleaseBundleReport<'_>, out: &mut String) {
    out.push_str("<h2>6. Accepted reviews</h2>\n");
    if report.reviews.acceptances.is_empty() {
        out.push_str(
            "<p class=\"empty\">No review was accepted for this run, so the disposition in section 5 is the \
             one Core computed with nothing set aside. An empty list is a fact, not a missing export.</p>\n",
        );
        return;
    }
    out.push_str(
        "<table>\n<caption>Who answered which finding, when, and why. An accepted review never \
                  rewrites the finding it answers: the original state stays REVIEW.</caption>\n",
    );
    out.push_str("<thead><tr><th>Finding</th><th>Original state</th><th>Actor</th><th>Accepted at</th><th>Reason</th></tr></thead>\n<tbody>\n");
    for acceptance in &report.reviews.acceptances {
        out.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td>{}</td><td class=\"mono\">{}</td><td>{}</td></tr>\n",
            escape_html(&acceptance.finding_id),
            escape_html(acceptance.original_state),
            escape_html(&acceptance.actor),
            escape_html(&acceptance.accepted_at),
            escape_html(&acceptance.reason)
        ));
    }
    out.push_str("</tbody>\n</table>\n");
    out.push_str(&format!(
        "<p class=\"meta\">The audit trail ships as <span class=\"mono\">{ACCEPTED_REVIEWS_DOC_NAME}</span> \
         beside this report, with the run id <span class=\"mono\">{}</span>.</p>\n",
        escape_html(&report.reviews.run_id)
    ));
}

// --------------------------------------------------------------------------- 7. release notes

fn render_notes(report: &ReleaseBundleReport<'_>, out: &mut String) {
    out.push_str("<h2>7. Release notes</h2>\n<table>\n<tbody>\n");
    match &report.model.release_notes {
        Some(notes) => {
            row(
                out,
                "Included",
                format!(
                    "yes, as <span class=\"mono\">{}</span>",
                    escape_html(&notes.bundle_name)
                ),
            );
            mono_row(out, "SHA-256 of the shipped copy", notes.sha256.hex());
            if let Some(entry) = staged_entry(report, &notes.bundle_name) {
                row(out, "Bytes", entry.size.to_string());
            }
            out.push_str(
                "<tr><td colspan=\"2\" class=\"meta\">The notes ship as their own file and are not \
                 reproduced inside this report. Markdown a reader wrote is never parsed, rewritten or \
                 embedded as raw HTML by the bundle, which is what keeps this document free of markup a \
                 note could carry.</td></tr>\n",
            );
        }
        None => out.push_str(
            "<tr><td>Included</td><td>no &mdash; the Gate context for this release observed no release \
             notes, and a passing run does not require them. Nothing was synthesized to fill the \
             slot.</td></tr>\n",
        ),
    }
    out.push_str("</tbody>\n</table>\n");
}

// --------------------------------------------------------------------------- 8. integrity

fn render_integrity(report: &ReleaseBundleReport<'_>, out: &mut String) {
    // The same statement of the rule the manifest will carry, built from the same code path, so the two
    // documents cannot drift into describing the coverage differently (§20).
    let model = crate::release::IntegrityModelDto::standard();
    out.push_str("<h2>8. Bundle integrity</h2>\n");
    out.push_str(&format!(
        "<p class=\"meta\">Two layers of SHA-256 lists cover this bundle. \
         <span class=\"mono\">{}</span> hashes every payload file except itself and the manifest. \
         <span class=\"mono\">{}</span> hashes every file except itself, that list included.</p>\n",
        model.sums_file, model.manifest_file
    ));

    out.push_str(
        "<table>\n<caption>Every file a digest can be named for here. This report is payload: it is \
         written before both index files, so it cannot print their digests, or its own.</caption>\n",
    );
    out.push_str("<thead><tr><th>Bundle path</th><th>SHA-256</th><th class=\"num\">Bytes</th></tr></thead>\n<tbody>\n");
    for entry in report.staged {
        out.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"num\">{}</td></tr>\n",
            escape_html(&entry.path),
            escape_html(&entry.sha256),
            entry.size
        ));
    }
    for name in [REPORT_DOC_NAME, SHA256SUMS_NAME, MANIFEST_DOC_NAME] {
        out.push_str(&format!(
            "<tr><td class=\"mono\">{}</td><td colspan=\"2\" class=\"meta\">indexed by <span class=\"mono\">{}</span>, \
             which is written after this report</td></tr>\n",
            escape_html(name),
            MANIFEST_DOC_NAME
        ));
    }
    out.push_str("</tbody>\n</table>\n");

    out.push_str("<h3>The self-reference rule</h3>\n<table>\n<tbody>\n");
    row(
        out,
        format!("<span class=\"mono\">{SHA256SUMS_NAME}</span> covers"),
        escape_html(model.sums_covers),
    );
    row(
        out,
        format!("<span class=\"mono\">{SHA256SUMS_NAME}</span> excludes"),
        name_list(model.sums_excludes),
    );
    row(
        out,
        format!("<span class=\"mono\">{MANIFEST_DOC_NAME}</span> covers"),
        escape_html(model.manifest_covers),
    );
    row(
        out,
        format!("<span class=\"mono\">{MANIFEST_DOC_NAME}</span> excludes"),
        name_list(model.manifest_excludes),
    );
    out.push_str(&format!(
        "<tr><td colspan=\"2\" class=\"meta\">A file cannot contain its own cryptographic hash without a \
         fixed-point scheme, so no self-digest is faked: <span class=\"mono\">self_digest_written</span> is \
         {}. {}</td></tr>\n",
        model.self_digest_written,
        escape_html(model.rule)
    ));
    out.push_str("</tbody>\n</table>\n");

    out.push_str("<h3>Verifying this bundle without FirmwareSight</h3>\n");
    out.push_str(
        "<p class=\"meta\">Every file here is a plain file. Any SHA-256 tool and any text editor is \
         enough. The check below proves that the bytes match the lists &mdash; an integrity and consistency \
         statement about the bundle as written. It says nothing about who wrote it, and nothing in this \
         bundle is a signature.</p>\n",
    );
    out.push_str("<ol class=\"steps\">\n");
    for step in VERIFICATION_STEPS {
        out.push_str(&format!("<li>{step}</li>\n"));
    }
    out.push_str("</ol>\n");
}

/// Spelled once so the report and whoever reads this file cannot diverge (§25.8, §44, §45).
///
/// The markup inside these strings is authored here and fixed; no value interpolated into the report
/// reaches it, so the section needs no escaping beyond what each sentence already carries.
const VERIFICATION_STEPS: [&str; 6] = [
    "List the directory. It must contain exactly the files <span class=\"mono\">release-manifest.json</span> \
     indexes, plus that manifest itself, and no temporary or hidden file.",
    "Recompute the SHA-256 of every file named in <span class=\"mono\">SHA256SUMS</span> and compare it with \
     the digest on that line. On a POSIX host one command does the whole list: \
     <span class=\"mono\">sha256sum -c SHA256SUMS</span>.",
    "On Windows, <span class=\"mono\">Get-FileHash -Algorithm SHA256 &lt;path&gt;</span> prints the same \
     digest in upper case; lower-case it before comparing, because both lists are written in lowercase hex.",
    "Compare every line of <span class=\"mono\">SHA256SUMS</span> with the entry of the same path in the \
     <span class=\"mono\">files[]</span> array of <span class=\"mono\">release-manifest.json</span>. Digests \
     and sizes must agree, and the manifest must additionally carry an entry for \
     <span class=\"mono\">SHA256SUMS</span> itself.",
    "Confirm that neither list contains a blank digest, an all-zero digest, or the digest of the file it sits \
     in. There should be none of those, and looking for them is what makes the rule in the table above \
     checkable rather than a claim.",
    "Open <span class=\"mono\">analysis.json</span>, <span class=\"mono\">gate-results.json</span> and \
     <span class=\"mono\">accepted-reviews.json</span> as plain JSON. Each names the schema it is an instance \
     of, and the manifest's <span class=\"mono\">extensions</span> block states which schema majors this \
     bundle was written against.",
];

fn name_list(names: &[&str]) -> String {
    names
        .iter()
        .map(|name| format!("<span class=\"mono\">{}</span>", escape_html(name)))
        .collect::<Vec<_>>()
        .join(", ")
}

// --------------------------------------------------------------------------- 9. limits

fn render_limits(report: &ReleaseBundleReport<'_>, out: &mut String) {
    out.push_str("<h2>9. Known capability limits of this release</h2>\n");
    out.push_str(
        "<p class=\"meta\">What the inputs of this build did not allow FirmwareSight to establish. A limit \
         is stated as a limit instead of being left as an empty cell.</p>\n",
    );
    let mut stated = false;

    let partial: Vec<(&str, &str)> = capability_rows(&report.analysis.capabilities)
        .into_iter()
        .filter(|(_, value)| !is_full_capability(value))
        .collect();
    if !partial.is_empty() {
        stated = true;
        out.push_str(
            "<table>\n<caption>Capability reporting, taken from the analysis document in this \
                      bundle.</caption>\n",
        );
        out.push_str("<thead><tr><th>Capability</th><th>Reported</th><th>What that means for this release</th></tr></thead>\n<tbody>\n");
        for (label, value) in partial {
            out.push_str(&format!(
                "<tr><td>{}</td><td class=\"mono\">{value}</td><td>{}</td></tr>\n",
                escape_html(label),
                escape_html(limit_sentence(label, value))
            ));
        }
        out.push_str("</tbody>\n</table>\n");
    }

    if let Some(compare) = report.compare {
        let objects = &compare.result.object_changes;
        if !objects.available {
            stated = true;
            out.push_str(&format!(
                "<div class=\"notice\"><p><strong>Object and module attribution</strong></p>\n<p class=\"meta\">{}</p></div>\n",
                escape_html(&objects.reason)
            ));
        }
        if !compare.result.warnings.is_empty() {
            stated = true;
            out.push_str(
                "<div class=\"notice\"><p><strong>Comparison warnings</strong></p>\n<ul>\n",
            );
            for warning in &compare.result.warnings {
                out.push_str(&format!(
                    "<li><span class=\"mono\">{}</span> &mdash; {}</li>\n",
                    escape_html(&warning.code),
                    escape_html(&warning.message)
                ));
            }
            out.push_str("</ul>\n</div>\n");
        }
    }

    let unknowns: Vec<&EvidenceDto> = report
        .analysis
        .evidence
        .iter()
        .filter(|item| item.classification == "unknown")
        .collect();
    if !unknowns.is_empty() {
        stated = true;
        out.push_str(
            "<table>\n<caption>Evidence rows the analysis classified as unknown, with the reason \
                      the analysis document carries.</caption>\n",
        );
        out.push_str(
            "<thead><tr><th>Field</th><th>Value</th><th>Rule</th></tr></thead>\n<tbody>\n",
        );
        for item in unknowns {
            out.push_str(&format!(
                "<tr><td class=\"mono\">{}</td><td>{}</td><td>{}</td></tr>\n",
                escape_html(&item.field),
                escape_html(&item.value),
                escape_html(&item.rule)
            ));
        }
        out.push_str("</tbody>\n</table>\n");
    }

    if !stated {
        out.push_str(
            "<p class=\"empty\">The analysis of this build recorded no capability gap, and the comparison \
             carried no warning. That is a report of what was measured, not a claim that no limit \
             exists.</p>\n",
        );
    }
}

fn capability_rows(capabilities: &CapabilitiesDto) -> [(&'static str, &'static str); 7] {
    [
        ("ELF support", capabilities.elf),
        ("Sections", capabilities.sections),
        ("Symbols", capabilities.symbols),
        ("Debug info", capabilities.debug_info),
        ("MAP companion", capabilities.map),
        ("Object attribution", capabilities.object_attribution),
        ("Git facts", capabilities.git),
    ]
}

/// The words that mean "nothing was missing here". The vocabulary is Core's, copied rather than judged: a
/// renderer that invented its own threshold for "good enough" would be a second rule book.
fn is_full_capability(value: &str) -> bool {
    matches!(value, "supported" | "available" | "provided")
}

fn limit_sentence(label: &str, value: &str) -> &'static str {
    if label == "Object attribution" && !is_full_capability(value) {
        return "Totals are reported per section and per symbol only. No per-object-file or per-module total \
                is claimed here, and none should be inferred from the section rows.";
    }
    match (label, value) {
        ("Debug info", _) => {
            "No readable debug information, so no source file or line is attributed to a symbol."
        }
        ("MAP companion", "not-provided") => {
            "No linker MAP was supplied for this build, so the layout rests on ELF headers and, where \
             configured, memory-region declarations."
        }
        ("Symbols", _) => {
            "No symbol table was readable, so symbol rows are absent because nothing was found, \
                           not because the build has no symbols."
        }
        _ => {
            "This capability is reported at less than full coverage by the analysis document in this bundle."
        }
    }
}

// --------------------------------------------------------------------------- shared helpers

fn row(out: &mut String, label: impl Into<String>, value: impl Into<String>) {
    let label = label.into();
    out.push_str(&format!(
        "<tr><th>{label}</th><td>{}</td></tr>\n",
        value.into()
    ));
}

fn mono_row(out: &mut String, label: impl Into<String>, value: impl AsRef<str>) {
    let value = mono(value.as_ref());
    row(out, label, value);
}

fn mono(value: &str) -> String {
    format!("<span class=\"mono\">{}</span>", escape_html(value))
}

fn not_observed() -> String {
    "<span class=\"unknown\">\u{2014} not observed by the Release Gate</span>".to_owned()
}

fn budget_row(
    out: &mut String,
    label: &str,
    state: &str,
    bytes: Option<u64>,
    reason: Option<&str>,
) {
    let mut text = match bytes {
        Some(bytes) => format!("{bytes} bytes <span class=\"meta\">({state})</span>"),
        None => format!("\u{2014} not a number <span class=\"meta\">({state})</span>"),
    };
    if let Some(reason) = reason {
        text.push_str(&format!(" &mdash; {}", escape_html(reason)));
    }
    row(out, label, text);
}

/// A delta against a base and target, from the diff document. Absence stays a dash: `0` would be a
/// measurement that was not taken (`04_TECH/11`).
fn delta_row(out: &mut String, label: &str, change: &ByteChangeDto) {
    let mut text = format!(
        "{} \u{2192} {} &middot; delta {} <span class=\"meta\">({})</span>",
        bytes_text(change.base),
        bytes_text(change.target),
        signed(change.delta),
        escape_html(change.comparability)
    );
    if let Some(reason) = &change.reason {
        text.push_str(&format!(" &mdash; {}", escape_html(reason)));
    }
    row(out, label, text);
}

fn bytes_text(bytes: Option<u64>) -> String {
    bytes.map_or_else(
        || "\u{2014} nothing recorded".to_owned(),
        |value| format!("{value} bytes"),
    )
}

fn signed(delta: Option<i64>) -> String {
    match delta {
        Some(value) if value > 0 => format!("+{value}"),
        Some(value) => value.to_string(),
        None => "\u{2014}".to_owned(),
    }
}

fn known_or_reason(value: &Option<String>, reason: &Option<String>) -> String {
    match (value, reason) {
        (Some(value), _) => mono(value),
        (None, Some(reason)) => format!("<span class=\"unknown\">{}</span>", escape_html(reason)),
        (None, None) => "<span class=\"unknown\">not recorded</span>".to_owned(),
    }
}

fn staged_entry<'a>(report: &ReleaseBundleReport<'a>, path: &str) -> Option<&'a ManifestFileDto> {
    report.staged.iter().find(|entry| entry.path == path)
}
