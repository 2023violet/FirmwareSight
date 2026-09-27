//! JSON rendering and the determinism checks that make "deterministic output" testable rather
//! than aspirational.

use serde::Serialize;

use crate::dto::AnalyzeResultDto;

/// Compact machine-readable form.
pub fn to_json(value: &AnalyzeResultDto) -> String {
    // Serializing a DTO cannot fail: every field is a plain serializable type and no path
    // collector or float is involved.
    serde_json::to_string(value).unwrap_or_else(infallible)
}

/// Human-readable JSON, used for goldens so a semantic diff stays reviewable by eye.
pub fn to_json_pretty(value: &AnalyzeResultDto) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(infallible)
}

fn infallible(err: serde_json::Error) -> String {
    panic!("DTO serialization must not fail: {err}")
}

/// Re-serialize JSON with object keys in sorted order.
///
/// Two outputs that differ only in key order collapse to the same canonical text, which lets a
/// test assert semantic equality without depending on field declaration order.
#[must_use]
pub fn canonical_form(json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .map(|value| serde_json::to_string(&value).unwrap_or_else(infallible))
}

#[must_use]
pub fn semantically_equal(left: &str, right: &str) -> bool {
    match (canonical_form(left), canonical_form(right)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// The `--json` contract: exactly one JSON document on stdout, nothing else.
#[must_use]
pub fn render_json(value: &AnalyzeResultDto) -> String {
    let mut out = to_json(value);
    out.push('\n');
    out
}

/// The default human-readable rendering. Diagnostics never go here; they go to stderr.
pub fn render_human(value: &AnalyzeResultDto) -> String {
    let mut out = String::new();
    out.push_str(&format!("{}\n\n", value.artifact.file_name));
    out.push_str(&format!("  SHA-256      {}\n", value.artifact.sha256));
    out.push_str(&format!(
        "  Size         {} bytes\n",
        value.artifact.byte_size
    ));
    out.push_str(&format!(
        "  Format       {} {}-bit {}\n",
        value.artifact.architecture, value.artifact.bitness, value.artifact.endianness
    ));
    out.push_str(&format!(
        "  Entry        {}\n",
        value
            .artifact
            .entry_point
            .clone()
            .unwrap_or_else(|| format!(
                "unknown ({})",
                value
                    .artifact
                    .entry_point_unknown_reason
                    .clone()
                    .unwrap_or_default()
            ))
    ));

    out.push_str("\nMemory\n");
    out.push_str(&format!(
        "  Nonvolatile / load image: {}\n",
        budget_line(&value.memory.nonvolatile_image_footprint)
    ));
    out.push_str(&format!(
        "  Runtime RAM:              {}\n",
        budget_line(&value.memory.runtime_ram_footprint)
    ));
    out.push_str(&format!(
        "  Evidence basis:           {} (layout: {})\n",
        value
            .memory
            .weakest_evidence_basis
            .unwrap_or("no allocatable sections"),
        value.memory.layout_source
    ));
    if !value.memory.dual_accounted_sections.is_empty() {
        out.push_str(&format!(
            "  Dual-accounted sections:  {}\n",
            value.memory.dual_accounted_sections.join(", ")
        ));
    }

    out.push_str("\nCounts\n");
    out.push_str(&format!(
        "  Sections {}   Symbols {}   Evidence {}\n",
        value.counts.sections, value.counts.symbols, value.counts.evidence
    ));

    out.push_str("\nCapabilities\n");
    for (label, field) in [
        ("ELF", value.capabilities.elf),
        ("Sections", value.capabilities.sections),
        ("Symbols", value.capabilities.symbols),
        ("Debug info", value.capabilities.debug_info),
        ("MAP", value.capabilities.map),
        ("Object attribution", value.capabilities.object_attribution),
        ("Git", value.capabilities.git),
    ] {
        out.push_str(&format!("  {label:<20} {field}\n"));
    }

    out
}

fn budget_line(budget: &crate::dto::BudgetDto) -> String {
    match (budget.state, budget.bytes) {
        ("exact", Some(bytes)) => format!("{bytes} bytes [{}]", budget.classification),
        ("partial", Some(bytes)) => format!(
            "{bytes} bytes or more, {} section(s) unattributed [{}]",
            budget.unattributed.len(),
            budget.classification
        ),
        _ => format!("unknown [{}]", budget.classification),
    }
}

/// The stable error shape the desktop and CLI both present.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
    pub operation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

impl ErrorEnvelope {
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        operation_id: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            operation_id: operation_id.into(),
            details: None,
            remediation: None,
        }
    }

    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    #[must_use]
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(infallible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_form_ignores_object_key_order() {
        let left = r#"{"a":1,"b":{"x":1,"y":2}}"#;
        let right = r#"{"b":{"y":2,"x":1},"a":1}"#;

        assert!(semantically_equal(left, right));
    }

    #[test]
    fn canonical_form_still_detects_a_real_difference() {
        assert!(!semantically_equal(r#"{"a":1}"#, r#"{"a":2}"#));
        assert!(!semantically_equal(r#"{"a":1,"b":2}"#, r#"{"a":1}"#));
    }

    #[test]
    fn malformed_json_is_not_equal_to_anything() {
        assert!(!semantically_equal("not json", r#"{"a":1}"#));
        assert!(canonical_form("not json").is_none());
    }

    #[test]
    fn error_envelope_carries_code_operation_and_remediation() {
        let envelope =
            ErrorEnvelope::new("ERR-PARSE-2002", "Could not parse artifact as ELF.", "op-7")
                .with_remediation("Choose the linker ELF output.");

        let json = envelope.to_json();

        assert!(json.contains("\"code\":\"ERR-PARSE-2002\""));
        assert!(json.contains("\"operationId\":\"op-7\""));
        assert!(json.contains("linker ELF output"));
        // Absent optional fields must not appear as null noise.
        assert!(!json.contains("null"), "unexpected null in {json}");
    }
}
