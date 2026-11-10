//! `fwsight gate --junit` (ADR-0040 D1): the Gate as one JUnit test suite, for the test tab of a CI system.
//!
//! One `<testcase>` per finding, named by its rule id. A finding with effective severity BLOCK is a
//! `<failure>`. One with REVIEW is `<skipped>` with "needs review", because a review is a decision a person
//! owes, not a broken test, and the process exit code still stops a pipeline. `N/A` is `<skipped>` as not
//! applicable, and PASS is a test that passed.
//!
//! The document is deterministic. It has no timestamp, host name or path, so the same run writes the same
//! bytes. Text is XML-escaped, and a character XML 1.0 cannot carry is written as its `\u{…}` code
//! instead of being dropped.

use crate::gate::{GateFindingDto, GateResultsDto};

/// How one finding reads in JUnit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseOutcome {
    Passed,
    Failed,
    NeedsReview,
    NotApplicable,
}

/// The mapping of ADR-0040 D1, from the finding's factual state and its effective severity.
#[must_use]
pub fn outcome(finding: &GateFindingDto) -> CaseOutcome {
    if finding.state == "N/A" {
        return CaseOutcome::NotApplicable;
    }
    match finding.effective_severity {
        "BLOCK" => CaseOutcome::Failed,
        "REVIEW" => CaseOutcome::NeedsReview,
        _ => CaseOutcome::Passed,
    }
}

/// The JUnit XML document for one Gate run.
#[must_use]
pub fn gate(results: &GateResultsDto) -> String {
    let outcomes: Vec<CaseOutcome> = results.findings.iter().map(outcome).collect();
    let failures = outcomes
        .iter()
        .filter(|o| **o == CaseOutcome::Failed)
        .count();
    let skipped = outcomes
        .iter()
        .filter(|o| matches!(o, CaseOutcome::NeedsReview | CaseOutcome::NotApplicable))
        .count();

    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<testsuites name=\"FirmwareSight\" tests=\"{tests}\" failures=\"{failures}\" errors=\"0\" skipped=\"{skipped}\">\n  <testsuite name=\"FirmwareSight release Gate\" tests=\"{tests}\" failures=\"{failures}\" errors=\"0\" skipped=\"{skipped}\" time=\"0\">\n",
        tests = results.findings.len()
    ));
    out.push_str("    <properties>\n");
    for (name, value) in [
        ("firmwaresight.run_id", results.run_id.as_str()),
        ("firmwaresight.snapshot_id", results.snapshot_id.as_str()),
        (
            "firmwaresight.verdict",
            results.extensions.overall_effective_severity,
        ),
        (
            "firmwaresight.policy_sha256",
            results.extensions.policy_sha256.as_str(),
        ),
    ] {
        out.push_str(&format!(
            "      <property name=\"{name}\" value=\"{}\"/>\n",
            xml(value)
        ));
    }
    out.push_str("    </properties>\n");

    for (finding, outcome) in results.findings.iter().zip(&outcomes) {
        let open = format!(
            "    <testcase classname=\"firmwaresight.gate\" name=\"{}\" time=\"0\"",
            xml(&finding.rule_id)
        );
        let detail = detail(finding);
        match outcome {
            CaseOutcome::Passed => out.push_str(&format!("{open}/>\n")),
            CaseOutcome::Failed => out.push_str(&format!(
                "{open}>\n      <failure type=\"{}\" message=\"{}\">{}</failure>\n    </testcase>\n",
                // ADR-0040 D1: the effective severity, so an UNKNOWN that policy blocks reads BLOCK; the
                // factual state stays in the body.
                xml(finding.effective_severity),
                xml(&finding.summary),
                xml(&detail)
            )),
            CaseOutcome::NeedsReview => out.push_str(&format!(
                "{open}>\n      <skipped message=\"needs review: {}\"/>\n      <system-out>{}</system-out>\n    </testcase>\n",
                xml(&finding.summary),
                xml(&detail)
            )),
            CaseOutcome::NotApplicable => out.push_str(&format!(
                "{open}>\n      <skipped message=\"not applicable: {}\"/>\n    </testcase>\n",
                xml(&finding.summary)
            )),
        }
    }
    out.push_str("  </testsuite>\n</testsuites>\n");
    out
}

/// The body a CI system shows when the case is opened: the state words, the next step and the evidence.
fn detail(finding: &GateFindingDto) -> String {
    let mut text = format!(
        "state {} · effective {}\n{}",
        finding.state, finding.effective_severity, finding.summary
    );
    if let Some(next) = &finding.remediation {
        text.push_str(&format!("\nnext step: {next}"));
    }
    if !finding.evidence_refs.is_empty() {
        text.push_str(&format!("\nevidence: {}", finding.evidence_refs.join(", ")));
    }
    text
}

/// XML 1.0 text and attribute escaping. The five markup characters become entities. A character XML 1.0
/// forbids (C0 controls other than tab, line feed and carriage return, and U+FFFE/U+FFFF) is shown as
/// `\u{…}`. Line feeds are kept in element text and written as `&#10;` so attributes keep them too.
#[must_use]
pub fn xml(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\n' => out.push_str("&#10;"),
            '\t' => out.push_str("&#9;"),
            '\r' => out.push_str("&#13;"),
            c if (c as u32) < 0x20 || c == '\u{FFFE}' || c == '\u{FFFF}' => {
                out.push_str(&c.escape_unicode().to_string());
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::GateExtensionsDto;

    fn finding(rule: &str, state: &'static str, effective: &'static str) -> GateFindingDto {
        GateFindingDto {
            id: format!("f-{rule}"),
            rule_id: rule.to_owned(),
            state,
            effective_severity: effective,
            summary: format!("{rule} summary"),
            evidence_refs: vec![],
            remediation: None,
        }
    }

    fn results(findings: Vec<GateFindingDto>) -> GateResultsDto {
        GateResultsDto {
            schema_version: 1,
            run_id: "run-1".to_owned(),
            snapshot_id: "snap-1".to_owned(),
            findings,
            extensions: GateExtensionsDto {
                policy_sha256: "abc".to_owned(),
                project_config_schema_version: 1,
                baseline_snapshot_id: None,
                overall_effective_severity: "BLOCK",
                canonical_input_label: None,
                attachments: vec![],
            },
        }
    }

    #[test]
    fn each_state_maps_to_the_junit_outcome_adr_0040_names() {
        assert_eq!(outcome(&finding("a", "PASS", "PASS")), CaseOutcome::Passed);
        assert_eq!(
            outcome(&finding("a", "BLOCK", "BLOCK")),
            CaseOutcome::Failed
        );
        assert_eq!(
            outcome(&finding("a", "REVIEW", "REVIEW")),
            CaseOutcome::NeedsReview
        );
        assert_eq!(
            outcome(&finding("a", "UNKNOWN", "BLOCK")),
            CaseOutcome::Failed
        );
        assert_eq!(
            outcome(&finding("a", "UNKNOWN", "REVIEW")),
            CaseOutcome::NeedsReview
        );
        assert_eq!(
            outcome(&finding("a", "N/A", "PASS")),
            CaseOutcome::NotApplicable
        );
    }

    #[test]
    fn the_suite_counts_and_cases_follow_the_findings() {
        let doc = gate(&results(vec![
            finding("memory.flash_budget", "BLOCK", "BLOCK"),
            finding("git.clean", "REVIEW", "REVIEW"),
            finding("release.notes", "N/A", "PASS"),
            finding("artifacts.hashes", "PASS", "PASS"),
        ]));
        assert!(doc.contains("tests=\"4\" failures=\"1\" errors=\"0\" skipped=\"2\""));
        assert!(doc.contains("<failure type=\"BLOCK\" message=\"memory.flash_budget summary\">"));
        assert!(doc.contains("<skipped message=\"needs review: git.clean summary\"/>"));
        assert!(doc.contains("<skipped message=\"not applicable: release.notes summary\"/>"));
        assert!(doc.contains("name=\"artifacts.hashes\" time=\"0\"/>"));
        assert!(doc.contains("<property name=\"firmwaresight.verdict\" value=\"BLOCK\"/>"));
        assert_eq!(
            doc,
            gate(&results(vec![
                finding("memory.flash_budget", "BLOCK", "BLOCK"),
                finding("git.clean", "REVIEW", "REVIEW"),
                finding("release.notes", "N/A", "PASS"),
                finding("artifacts.hashes", "PASS", "PASS"),
            ])),
            "the same run writes the same bytes"
        );
    }

    #[test]
    fn text_from_a_policy_cannot_become_markup_or_invalid_xml() {
        assert_eq!(xml("a<b>&\"c'"), "a&lt;b&gt;&amp;&quot;c&apos;");
        assert_eq!(xml("x\u{1b}[31my"), "x\\u{1b}[31my");
        assert_eq!(xml("line\nbreak"), "line&#10;break");
        assert_eq!(xml("\u{FFFF}"), "\\u{ffff}");
        let mut evil = finding("x", "BLOCK", "BLOCK");
        evil.summary = "</failure><testcase name=\"forged\"/>".to_owned();
        let doc = gate(&results(vec![evil]));
        assert!(!doc.contains("name=\"forged\""));
    }

    #[test]
    fn an_unknown_that_policy_blocks_fails_as_block_and_keeps_its_state_in_the_body() {
        let doc = gate(&results(vec![finding(
            "evidence.unknown_review",
            "UNKNOWN",
            "BLOCK",
        )]));
        assert!(doc.contains("<failure type=\"BLOCK\""), "{doc}");
        assert!(doc.contains("state UNKNOWN · effective BLOCK"), "{doc}");
    }
}
