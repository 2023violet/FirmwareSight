//! `--summary-md` (ADR-0040 D2): a short Markdown block for a pull-request comment or a CI job summary.
//!
//! The full Markdown reports in [`crate::markdown`] are several screens long. This is the part a reviewer
//! reads in a PR: the two budgets, the largest five movers, and the Gate's verdict with what is not PASS.
//! The block follows the report rules: it is deterministic, contains no HTML apart from its marker,
//! escapes every name, and writes Unknown and absent values as words, never as 0.
//!
//! Every block starts with [`MARKER`], so a script can find the comment it wrote earlier and update it.
//! A name is cut to [`NAME_LIMIT`] characters, so the block stays far below GitHub's 65,536-character
//! comment limit whatever the input holds.

use firmwaresight_core::domain::composition::Side;
use firmwaresight_core::domain::diff::DiffResult;
use firmwaresight_core::domain::waterfall::{StepKind, growth_waterfall};

use crate::gate::GateResultsDto;
use crate::gate_render::sentence_from_words;
use crate::markdown::{cell_code, delta_text, escape, side};

/// The first line of every summary: an HTML comment GitHub does not render, used to find the comment again.
pub const MARKER: &str = "<!-- firmwaresight-summary -->";

/// The longest name shown, in characters; the rest is replaced by an ellipsis.
pub const NAME_LIMIT: usize = 120;

/// How many movers each budget lists.
const MOVERS: usize = 5;

fn short(raw: &str) -> String {
    if raw.chars().count() <= NAME_LIMIT {
        raw.to_owned()
    } else {
        let mut cut: String = raw.chars().take(NAME_LIMIT - 1).collect();
        cut.push('…');
        cut
    }
}

fn footer(out: &mut String) {
    out.push_str(&format!(
        "\n_FirmwareSight {} · analysed locally, nothing uploaded._\n",
        env!("CARGO_PKG_VERSION")
    ));
}

/// `fwsight diff --summary-md`: the two budgets, the largest movers, and the warning codes.
#[must_use]
pub fn comparison(result: &DiffResult) -> String {
    let memory = &result.memory;
    let mut out = format!("{MARKER}\n### FirmwareSight · size change\n\n");
    out.push_str(&format!(
        "{} → {}\n\n",
        cell_code(&short(&result.base_artifact.file_name)),
        cell_code(&short(&result.target_artifact.file_name))
    ));
    out.push_str("| Budget | Base | Target | Δ | Comparability |\n|---|---:|---:|---:|---|\n");
    for (label, change) in [
        ("Flash (load image)", &memory.nonvolatile),
        ("RAM (runtime)", &memory.runtime_ram),
    ] {
        out.push_str(&format!(
            "| {label} | {} | {} | **{}** | {} |\n",
            side(change.base),
            side(change.target),
            delta_text(change.delta),
            change.comparability.label()
        ));
    }

    let mut any = false;
    for (budget, name) in [(Side::Nonvolatile, "Flash"), (Side::Runtime, "RAM")] {
        let fall = growth_waterfall(result, budget, false, MOVERS);
        if !fall.available {
            continue;
        }
        let items: Vec<_> = fall
            .steps
            .iter()
            .filter(|step| step.kind == StepKind::Item)
            .collect();
        if items.is_empty() {
            continue;
        }
        any = true;
        out.push_str(&format!(
            "\n**Largest movers · {name}** (by object)\n\n| Object | Change | Δ |\n|---|---|---:|\n"
        ));
        for step in items {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                cell_code(&short(&step.label)),
                step.change
                    .map_or("—".to_owned(), |c| c.label().to_ascii_lowercase()),
                delta_text(step.delta)
            ));
        }
    }
    if !any {
        let rows = result.top_section_growth(MOVERS);
        if rows.is_empty() {
            out.push_str("\nNo section that costs the device grew.\n");
        } else {
            out.push_str(&format!(
                "\n**Largest section growth** (object attribution: {})\n\n| Section | Δ |\n|---|---:|\n",
                if result.attribution.available {
                    "no object moved".to_owned()
                } else {
                    escape(&short(&result.attribution.reason))
                }
            ));
            for row in rows {
                out.push_str(&format!(
                    "| {} | {} |\n",
                    cell_code(&short(&row.key)),
                    delta_text(row.delta)
                ));
            }
        }
    }

    if !result.warnings.is_empty() {
        let codes: Vec<String> = result
            .warnings
            .iter()
            .map(|warning| cell_code(&short(&warning.code)))
            .collect();
        out.push_str(&format!("\nWarnings: {}\n", codes.join(", ")));
    }
    footer(&mut out);
    out
}

/// `fwsight gate --summary-md`: the verdict, the sentence that explains it, and every finding that is
/// neither PASS nor N/A.
#[must_use]
pub fn gate(results: &GateResultsDto) -> String {
    let verdict = results.extensions.overall_effective_severity;
    let passed = results
        .findings
        .iter()
        .filter(|finding| finding.state == "PASS")
        .count();
    let sentence = sentence_from_words(
        verdict,
        results
            .findings
            .iter()
            .map(|finding| (finding.state, finding.effective_severity)),
        passed,
    );
    let mut out = format!(
        "{MARKER}\n### FirmwareSight · release Gate: **{verdict}**\n\n{}\n",
        escape(&sentence)
    );
    let open: Vec<_> = results
        .findings
        .iter()
        .filter(|finding| finding.state != "PASS" && finding.state != "N/A")
        .collect();
    if !open.is_empty() {
        out.push_str("\n| Rule | State | Effective | Summary |\n|---|---|---|---|\n");
        for finding in open {
            out.push_str(&format!(
                "| {} | **{}** | **{}** | {} |\n",
                cell_code(&finding.rule_id),
                finding.state,
                finding.effective_severity,
                escape(&short(&finding.summary))
            ));
        }
    }
    out.push_str(&format!(
        "\nRun {} · policy readiness, not a certification.\n",
        cell_code(&results.run_id)
    ));
    footer(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_name_is_cut_and_says_so() {
        let long = "x".repeat(500);
        let cut = short(&long);
        assert_eq!(cut.chars().count(), NAME_LIMIT);
        assert!(cut.ends_with('…'));
        assert_eq!(short("main.o"), "main.o");
    }
}
