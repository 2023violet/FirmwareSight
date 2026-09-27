---
title: "Validation Interview Question Bank"
doc_id: "FS-DEL-010"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Research / Product"
last_updated: "2026-09-27"
---

# Validation Interview Question Bank

Use alongside the core V0/V1 tasks, not as a survey-only substitute for observation.

## Export / handoff
- After analysis, who else needs the result?
- What do they actually open: browser HTML, JSON, CSV, Markdown, spreadsheet, Word, PDF?
- Is a self-contained file more useful than a cloud link?
- What information must survive handoff?

## Repeated analysis / History
- How often do you re-run size/release checks?
- What triggers it: local build, commit, PR, nightly, pre-release?
- Would automatic watch reduce work or create noise?
- Which event deserves a system notification?

## Stack evidence
- Do you generate GCC `.su`/stack-usage data today?
- Tell us about the last stack-sizing/overflow incident.
- Is worst-stack per function useful without a call graph?
- What quota/threshold evidence exists in your workflow?

## CI / automation
- Would a failing Gate in CI be useful or disruptive?
- Who owns that policy?
- Would you pay for CI/team policy, or is local CLI enough?

## Release symbol archive
- Have you needed the exact ELF/AXF/MAP after a release for crash post-mortem?
- How do you find it today?
- How long after release did you need it?

## Component evidence
- Which dependency source is authoritative in your firmware project?
- Do component versions drift without explicit review?
- For ESP-IDF, do you commit `dependencies.lock`?

## CRA / external review
- Are customers/regulators asking for SBOM/provenance/release evidence now?
- Is there a deadline driving tool purchase?
- What evidence is actually requested?
- Never phrase the question as “do you need CRA compliance from FirmwareSight?”

## Payment
Ask for concrete price/approval process, not compliments:
- Would you budget for this?
- Who approves it?
- What current manual work/cost does it replace?
- Would you run a team pilot?
