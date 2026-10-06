---
title: "V1 Product Findings Register"
doc_id: "FS-V1-032"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Research / Engineering"
last_updated: "2026-10-06"
---

# V1 Product Findings Register

Every finding from a real session is classified before it is described (§31). The classification exists to stop
two opposite errors: calling a preference a bug, and calling correctly-unsupported Keil/IAR input a parser
defect.

## Classification

`PRODUCT` · `RESEARCH_HARNESS` · `ENVIRONMENT` · `UNSUPPORTED_COHORT` · `KNOWN_LIMITATION` · `PREFERENCE` ·
`FEATURE_REQUEST` · `OBSERVATION`

## Product severity

| Level | Meaning | Consequence (§32, §36) |
|---|---|---|
| S0 | catastrophic data, security or integrity damage | stop formal sessions, preserve evidence, return to the Architect |
| S1 | core workflow blocker, or materially wrong supported evidence | same |
| S2 | major product or usability defect | continue only if it does not invalidate measured tasks; flag for Interim Review; the same S2 seen independently by ≥ 3 participants is an early-stop trigger |
| S3 | minor defect | record |
| S4 | cosmetic or observation | record |

`RESEARCH_HARNESS` findings are about the moderator's own tooling and notes; V0 and V1 both own a history of
harness defects being recorded against the harness rather than the product, and this category exists so that
history continues. `ENVIRONMENT` covers a participant's machine, policy or permissions. `UNSUPPORTED_COHORT`
covers Keil/IAR and anything outside the measured support boundary in
`P5_VALIDATION/P5_COMPATIBILITY_MATRIX.md`. `KNOWN_LIMITATION` links to a numbered row of
`P5_VALIDATION/P5_KNOWN_LIMITATIONS.md` instead of re-reporting it as new.

## Register

| ID | Classification | Severity | Found in | What happened | Frozen build affected? | Metric touched | Disposition |
|---|---|---|---|---|---|---|---|
| — | | | | | | | |

**0 findings, because 0 sessions have been conducted.**

## Freeze rule while this register is open (§32)

The formal cohort runs against one frozen artifact: head `08fdfcb`, artifact id `11419727517`. No silent product
patch mid-cohort. A product change creates a new cohort version and a **new CI artifact**, and pre- and
post-hotfix metrics are never pooled. V1 recommends; it does not fix (§33) — which is also why this register can
stay open across a batch without a single line of Rust changing.

## Later synthesis

When `N ≥ 8` or the Architect explicitly asks for interim synthesis, this register is summarized into
`analysis/V1_PRODUCT_FINDINGS.md` (§52). That file does not exist yet and creating it now would summarize
nothing.
