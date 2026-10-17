//! Text and JSON rendering of a Gate run.
//!
//! The human form orders findings by the five states in the sequence a release owner reads them —
//! `BLOCK`, `REVIEW`, `UNKNOWN`, `PASS`, `N/A` — because the first three are the work and the last two
//! are the reassurance (prompt §37). Inside a state the canonical rule order is kept, so two runs of
//! the same build print the same lines in the same order.
//!
//! Where a finding's factual state and its effective severity are different words, both are printed.
//! An `UNKNOWN` whose disposition is `BLOCK` must not read as a soft answer, and that distinction is
//! the whole content of ADR-0023's two-field rule.
//!
//! The Markdown form lives in `markdown.rs` (R6) and reads in the same order.

use firmwaresight_core::domain::gate::{EffectiveSeverity, FindingState, GateEvaluation};

use crate::gate::{AcceptedReviewsDto, GateResultsDto};

/// The `--json` contract: exactly one JSON document on stdout, nothing else.
#[must_use]
pub fn render_json(dto: &GateResultsDto) -> String {
    let mut out = crate::render::to_json(dto);
    out.push('\n');
    out
}

/// The accepted-reviews document, for a run whose trail is exported beside its verdict.
#[must_use]
pub fn render_reviews_json(dto: &AcceptedReviewsDto) -> String {
    let mut out = crate::render::to_json(dto);
    out.push('\n');
    out
}

/// The state order a person reads, not the order the rules were evaluated in.
///
/// Shared with the release report so one document type has one reading order; a second order would be a
/// second definition of "what to look at first" (`AGENTS.md` 3).
pub(crate) const STATE_ORDER: [FindingState; 5] = [
    FindingState::Block,
    FindingState::Review,
    FindingState::Unknown,
    FindingState::Pass,
    FindingState::NotApplicable,
];

/// The default human-readable rendering, in the report grammar (R6): masthead, the answer, the subject,
/// the findings by state, and the limit. Diagnostics never go here; they go to stderr.
///
/// The answer counts the findings whose effective severity carries the verdict, so a REVIEW reached
/// through `[gate.on_unknown]` is counted, and it says how many of them are missing evidence. A finding's
/// effective severity is printed beside it only when it says something its state does not; an `N/A`
/// finding never shows one, because its `PASS` is a schema filler rather than a verdict (`03_DESIGN/09`
/// §8.2, the old `[PASS]` under N/A).
#[must_use]
pub fn render_human(evaluation: &GateEvaluation, policy_sha256: &str) -> String {
    use crate::text::{heading, masthead, state_tag, wrap};

    let verdict = evaluation.overall_effective_severity;
    let mut out = masthead("Release Gate report");
    out.push_str(&wrap(
        &verdict_sentence(evaluation),
        &state_tag(&verdict.to_string()),
        "          ",
    ));
    out.push_str(&format!(
        "          BLOCK {} · REVIEW {} · UNKNOWN {} · PASS {} · N/A {}\n",
        evaluation.counts.block,
        evaluation.counts.review,
        evaluation.counts.unknown,
        evaluation.counts.pass,
        evaluation.counts.not_applicable
    ));
    out.push_str(&wrap(
        "FirmwareSight policy readiness: the answer of this project's policy, not a certification and not a permission to release.",
        "          ",
        "          ",
    ));

    out.push_str(&heading("Subject"));
    out.push_str(&format!("  Run        {}\n", evaluation.run_id));
    out.push_str(&format!("  Snapshot   {}\n", evaluation.snapshot_id));
    out.push_str(&format!(
        "  Baseline   {}\n",
        evaluation
            .baseline_snapshot_id
            .as_deref()
            .unwrap_or("none (no comparison in this run)")
    ));
    out.push_str(&format!("  Policy     sha256 {policy_sha256}\n"));

    for state in STATE_ORDER {
        let findings: Vec<_> = evaluation
            .findings
            .iter()
            .filter(|finding| finding.state == state)
            .collect();
        if findings.is_empty() {
            continue;
        }
        out.push_str(&heading(&format!(
            "{} ({})",
            state.as_str(),
            findings.len()
        )));
        for finding in findings {
            let carried = state != FindingState::NotApplicable
                && finding.effective_severity.as_str() != state.as_str();
            let rule = if carried {
                format!(
                    "{}  -> effective {}",
                    finding.rule_id.as_str(),
                    finding.effective_severity
                )
            } else {
                finding.rule_id.as_str().to_owned()
            };
            out.push_str(&format!("  {}{rule}\n", state_tag(state.as_str())));
            out.push_str(&wrap(&finding.summary, INDENT, INDENT));
            if let Some(remediation) = &finding.remediation {
                out.push_str(&wrap(&format!("next: {remediation}"), INDENT, INDENT));
            }
            if !finding.evidence_refs.is_empty() {
                out.push_str(&wrap(
                    &format!("refs: {}", finding.evidence_refs.join(", ")),
                    INDENT,
                    INDENT,
                ));
            }
        }
    }

    if evaluation.counts.review > 0 {
        out.push_str(&heading("Next"));
        out.push_str(&wrap(
            "A REVIEW is disposed of by accepting it with a reason in the desktop Release page, which records who accepted it and never rewrites this finding.",
            "  ",
            "  ",
        ));
    }
    out
}

/// The hanging indent under a finding: two spaces and the width of a state tag.
const INDENT: &str = "            ";

/// One sentence that answers the run, counted from Core's own fields on each finding.
pub(crate) fn verdict_sentence(evaluation: &GateEvaluation) -> String {
    let verdict = evaluation.overall_effective_severity;
    let carrying: Vec<_> = evaluation
        .findings
        .iter()
        .filter(|finding| {
            finding.state != FindingState::NotApplicable && finding.effective_severity == verdict
        })
        .collect();
    let unknown = carrying
        .iter()
        .filter(|finding| finding.state == FindingState::Unknown)
        .count();
    let missing = if unknown == 0 {
        String::new()
    } else {
        format!(", {unknown} of them because their evidence is missing")
    };
    match verdict {
        EffectiveSeverity::Block => format!(
            "Blocked: {} finding(s) block this build{missing}. A blocked build cannot produce a release bundle.",
            carrying.len()
        ),
        EffectiveSeverity::Review => format!(
            "Needs a decision: {} finding(s) wait on an explicit acceptance{missing}. Nothing is blocked.",
            carrying.len()
        ),
        EffectiveSeverity::Pass => format!(
            "Clear: {} rule(s) pass, nothing is blocked and nothing waits on a review.",
            evaluation.counts.pass
        ),
    }
}
