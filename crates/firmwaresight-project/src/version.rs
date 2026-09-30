//! Compile and apply `[version] pattern`.
//!
//! The regex crate lives here rather than in `firmwaresight-core`, because Core declares no external
//! dependencies (ADR-0027). Core receives the *outcome* — matched or not, and what the `version`
//! capture yielded — and decides what that means for the rule.

use firmwaresight_core::domain::gate::GateVersionFacts;
use firmwaresight_core::domain::identity::Fact;
use regex::Regex;

use crate::error::ProjectError;

/// Compile a user-declared pattern. A pattern that does not compile is a config error, reported with
/// the regex crate's own reason so the owner can fix the pattern rather than guess.
pub fn compile_pattern(pattern: &str) -> Result<Regex, ProjectError> {
    Regex::new(pattern).map_err(|error| ProjectError::InvalidVersionPattern {
        detail: error.to_string(),
    })
}

/// Match the workspace tag against the pattern and read the `version` capture.
///
/// `tag` is `None` when no exact tag points at HEAD; the caller reports that as a determinate failure
/// before this runs, so a `None` here yields an unknown match rather than a guess.
#[must_use]
pub fn evaluate_tag(pattern: &str, tag: Option<&str>) -> GateVersionFacts {
    let Some(tag) = tag else {
        return GateVersionFacts {
            pattern_matches: Fact::unknown("no exact tag points at workspace HEAD"),
            captured_version: Fact::unknown("no exact tag points at workspace HEAD"),
        };
    };
    match compile_pattern(pattern) {
        Err(error) => GateVersionFacts {
            pattern_matches: Fact::unknown(error.to_string()),
            captured_version: Fact::unknown(error.to_string()),
        },
        Ok(compiled) => {
            let Some(captures) = compiled.captures(tag) else {
                return GateVersionFacts {
                    pattern_matches: Fact::known(false),
                    captured_version: Fact::unknown(format!(
                        "tag `{tag}` did not match the configured pattern"
                    )),
                };
            };
            GateVersionFacts {
                pattern_matches: Fact::known(true),
                captured_version: match captures.name("version") {
                    Some(captured) => Fact::known(captured.as_str().to_owned()),
                    None => Fact::unknown(
                        "the pattern matched but declares no `version` capture".to_owned(),
                    ),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATTERN: &str = r"^v(?P<version>\d+\.\d+\.\d+)$";

    #[test]
    fn a_matching_tag_yields_the_named_capture() {
        let facts = evaluate_tag(PATTERN, Some("v1.23.4"));
        assert_eq!(facts.pattern_matches, Fact::known(true));
        assert_eq!(facts.captured_version, Fact::known("1.23.4".to_owned()));
    }

    #[test]
    fn a_non_matching_tag_matches_nothing_and_carries_the_reason() {
        let facts = evaluate_tag(PATTERN, Some("release-1.2.3"));
        assert_eq!(facts.pattern_matches, Fact::known(false));
        assert!(facts.captured_version.value().is_none());
    }

    #[test]
    fn a_pattern_without_the_capture_matches_but_yields_no_version() {
        let facts = evaluate_tag(r"^v\d+\.\d+\.\d+$", Some("v1.2.3"));
        assert_eq!(facts.pattern_matches, Fact::known(true));
        assert_eq!(
            facts.captured_version.reason_if_unknown(),
            Some("the pattern matched but declares no `version` capture")
        );
    }

    #[test]
    fn no_tag_is_an_absent_fact_not_a_mismatch() {
        let facts = evaluate_tag(PATTERN, None);
        assert!(!facts.pattern_matches.is_known());
        assert!(
            !facts
                .pattern_matches
                .reason_if_unknown()
                .unwrap_or_default()
                .contains("did not match"),
            "an absent tag is a different fact from a mismatched one"
        );
    }

    #[test]
    fn an_uncompilable_pattern_is_reported_not_panic() {
        assert!(compile_pattern("(?P<version>").is_err());
        let facts = evaluate_tag("(?P<version>", Some("v1.2.3"));
        assert!(!facts.pattern_matches.is_known());
    }
}
