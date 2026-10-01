//! The published portable contracts, compiled into the build that has to check against them.
//!
//! Prompt §44 asks the bundle verifier to prove that `analysis.json`, `diff.json`, `gate-results.json`,
//! `accepted-reviews.json` and `release-manifest.json` are valid against their schemas, and §45 asks it to do
//! that on a machine with no FirmwareSight checkout — a bundle copied to a stranger's folder, verified from
//! its own bytes. A schema read from `schemas/` at runtime would not exist there, so the contract text is
//! embedded at compile time instead: the binary that writes a bundle carries the contracts it wrote it under,
//! which is also the honest answer to "which version produced this?" when the schema has since moved on.
//!
//! This module owns no schema. `schemas/*.schema.json` at the repository root stays the single authored copy,
//! and `the_embedded_contract_is_the_file_in_the_repository` fails if the two ever diverge.
//!
//! Validation runs through [`crate::schema_check`], the dependency-free subset of draft/2020-12 these
//! documents actually use (prompt §15, §64: no JSON-Schema crate is admitted). The subset reports an
//! unimplemented keyword as a failure rather than ignoring it, so a schema that starts using a keyword the
//! checker cannot evaluate is caught here instead of silently passing a document that violates it.

use serde_json::Value;

use crate::schema_check::{self, Failure};

/// The portable `analysis:1` contract (§14).
pub const ANALYSIS: &str = include_str!("../../../schemas/analysis.schema.json");
/// The portable `diff:1` contract (§16).
pub const DIFF: &str = include_str!("../../../schemas/diff.schema.json");
/// The portable `gate-results:1` contract (§17).
pub const GATE_RESULTS: &str = include_str!("../../../schemas/gate-results.schema.json");
/// The portable `accepted-reviews:1` contract (§18).
pub const ACCEPTED_REVIEWS: &str = include_str!("../../../schemas/accepted-reviews.schema.json");
/// The portable `release-manifest:1` contract (§19).
pub const RELEASE_MANIFEST: &str = include_str!("../../../schemas/release-manifest.schema.json");

/// The contract one bundle document is written under, by its canonical bundle-relative name.
///
/// `release-notes.md` and the files under `artifacts/` have none: they are the release owner's bytes, copied
/// byte-for-byte (§24, §22), and a document this engine composed rather than copied is the only kind a schema
/// can say anything about. `SHA256SUMS` is checked by its own format rule (§21).
#[must_use]
pub fn for_document(name: &str) -> Option<&'static str> {
    match name {
        firmwaresight_core::domain::release::ANALYSIS_DOC_NAME => Some(ANALYSIS),
        firmwaresight_core::domain::release::DIFF_DOC_NAME => Some(DIFF),
        firmwaresight_core::domain::release::GATE_RESULTS_DOC_NAME => Some(GATE_RESULTS),
        firmwaresight_core::domain::release::ACCEPTED_REVIEWS_DOC_NAME => Some(ACCEPTED_REVIEWS),
        firmwaresight_core::domain::release::MANIFEST_DOC_NAME => Some(RELEASE_MANIFEST),
        _ => None,
    }
}

/// One embedded contract, parsed.
///
/// A schema that does not parse is a build defect rather than a runtime condition: every one of the five is
/// read at compile time and validated by `the_embedded_contract_is_the_file_in_the_repository`, so a failure
/// here means someone shipped a file `schemas/` never held.
#[must_use]
pub fn parse(text: &str) -> Value {
    serde_json::from_str(text).expect("an embedded contract parses as JSON")
}

/// Check one bundle document against the contract its name selects.
///
/// # Errors
///
/// Returns the subset checker's failures, each with the JSON pointer it was found at, when the document does
/// not satisfy its contract. An empty list means it does.
pub fn validate(name: &str, document: &Value) -> Result<Vec<Failure>, String> {
    let schema = for_document(name).ok_or_else(|| format!("`{name}` has no portable contract"))?;
    let schema = parse(schema);
    Ok(schema_check::validate(&schema, document))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crates/firmwaresight-report lives at <root>/crates/<name>")
            .to_path_buf()
    }

    #[test]
    fn the_embedded_contract_is_the_file_in_the_repository() {
        for (file, embedded) in [
            ("analysis.schema.json", ANALYSIS),
            ("diff.schema.json", DIFF),
            ("gate-results.schema.json", GATE_RESULTS),
            ("accepted-reviews.schema.json", ACCEPTED_REVIEWS),
            ("release-manifest.schema.json", RELEASE_MANIFEST),
        ] {
            let path = repo_root().join("schemas").join(file);
            let authored = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
            assert_eq!(
                embedded.trim_end(),
                authored.trim_end(),
                "{file} was edited without rebuilding the embedded copy this verifier checks against"
            );
        }
    }

    #[test]
    fn every_embedded_contract_parses_and_declares_its_identity() {
        for (name, text) in [
            ("analysis", ANALYSIS),
            ("diff", DIFF),
            ("gate-results", GATE_RESULTS),
            ("accepted-reviews", ACCEPTED_REVIEWS),
            ("release-manifest", RELEASE_MANIFEST),
        ] {
            let schema = parse(text);
            let urn = schema["$id"]
                .as_str()
                .unwrap_or_else(|| panic!("{name} carries no $id"));
            assert!(
                urn.starts_with("urn:firmwaresight:schema:") && urn.ends_with(":1"),
                "{name} identifies itself as {urn}"
            );
            assert!(
                schema.get("properties").is_some(),
                "{name} declares no properties"
            );
            assert!(
                schema.get("required").and_then(Value::as_array).is_some(),
                "{name} declares no required keys, which would let an empty object validate"
            );
        }
    }

    #[test]
    fn only_the_five_composed_documents_have_a_contract() {
        for name in [
            "analysis.json",
            "diff.json",
            "gate-results.json",
            "accepted-reviews.json",
            "release-manifest.json",
        ] {
            assert!(
                for_document(name).is_some(),
                "`{name}` should have a contract"
            );
        }
        for name in [
            "release-notes.md",
            "release-report.html",
            "SHA256SUMS",
            "artifacts/firmware.elf",
            "",
        ] {
            assert!(
                for_document(name).is_none(),
                "`{name}` has no schema to be valid against"
            );
        }
    }

    #[test]
    fn validate_reports_where_a_document_falls_short_instead_of_silently_passing_it() {
        let document = serde_json::json!({ "schema": 1 });
        let failures = validate("analysis.json", &document).expect("analysis.json has a contract");
        assert!(!failures.is_empty(), "a stub document must not validate");
        assert!(
            failures
                .iter()
                .any(|failure| failure.path.contains("schemaVersion")
                    || failure.message.contains("schemaVersion")),
            "the report should name the missing key: {failures:?}"
        );
        let error =
            validate("release-notes.md", &serde_json::json!({})).expect_err("notes have no schema");
        assert!(error.contains("release-notes.md"), "{error}");
    }
}
