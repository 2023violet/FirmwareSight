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
//! No HTML form exists here: a Release Bundle is P4 scope, and a Gate document that ships in one would
//! arrive before the contract that governs it.

use firmwaresight_core::domain::gate::{FindingState, GateEvaluation};

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

/// The default human-readable rendering. Diagnostics never go here; they go to stderr.
#[must_use]
pub fn render_human(evaluation: &GateEvaluation, policy_sha256: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Release Gate: {}\n\n",
        evaluation.overall_effective_severity
    ));
    out.push_str(&format!("  Run       {}\n", evaluation.run_id));
    out.push_str(&format!("  Snapshot  {}\n", evaluation.snapshot_id));
    out.push_str(&format!(
        "  Baseline  {}\n",
        evaluation
            .baseline_snapshot_id
            .as_deref()
            .unwrap_or("none (no comparison in this run)")
    ));
    out.push_str(&format!("  Policy    sha256 {policy_sha256}\n"));

    out.push('\n');
    out.push_str(&format!(
        "  BLOCK {}   REVIEW {}   UNKNOWN {}   PASS {}   N/A {}\n",
        evaluation.counts.block,
        evaluation.counts.review,
        evaluation.counts.unknown,
        evaluation.counts.pass,
        evaluation.counts.not_applicable
    ));

    for state in STATE_ORDER {
        let findings: Vec<_> = evaluation
            .findings
            .iter()
            .filter(|finding| finding.state == state)
            .collect();
        if findings.is_empty() {
            continue;
        }
        out.push_str(&format!("\n{}\n", state.as_str()));
        for finding in findings {
            // The severity is named only when it says something the state does not.
            let severity = if finding.effective_severity.as_str() == state.as_str() {
                String::new()
            } else {
                format!("  [{}]", finding.effective_severity)
            };
            out.push_str(&format!("  {}{severity}\n", finding.rule_id.as_str()));
            out.push_str(&wrap(&finding.summary, "    "));
            if let Some(remediation) = &finding.remediation {
                out.push_str(&wrap(&format!("next: {remediation}"), "    "));
            }
            if !finding.evidence_refs.is_empty() {
                out.push_str(&wrap(
                    &format!("refs: {}", finding.evidence_refs.join(", ")),
                    "    ",
                ));
            }
        }
    }

    if evaluation.counts.review > 0 {
        out.push_str(
            "\n  A REVIEW is disposed of by accepting it with a reason in the desktop Release page, \
             which records who accepted it and never rewrites this finding.\n",
        );
    }
    out
}

/// Indented, 88-column wrapping so a multi-sentence summary stays readable in a terminal and a diff of
/// the output stays line-oriented.
fn wrap(text: &str, indent: &str) -> String {
    const WIDTH: usize = 88;
    let mut out = String::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let extra = word.len() + usize::from(!line.is_empty());
        if !line.is_empty() && indent.len() + line.len() + extra > WIDTH {
            out.push_str(&format!("{indent}{line}\n"));
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push_str(&format!("{indent}{line}\n"));
    }
    out
}
