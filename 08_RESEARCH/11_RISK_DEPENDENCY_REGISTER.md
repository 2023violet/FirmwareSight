---
title: "Risk and Dependency Register v0.5"
doc_id: "FS-RSCH-012"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-27"
---

# Risk and Dependency Register

## R1 — willingness to pay
Status: Unvalidated / highest product risk.

Mitigation:
- V0 payment/process questions;
- V1 own-artifact use;
- ≥3 price-anchored willingness signals or ≥1 team pilot before meaningful Growth investment.

## R2 — toolchain fragmentation
Risk:
- GNU MAP variability;
- Keil/ArmClang/IAR format differences.

Mitigation:
fixture-gated adapter support; no universal-parser claim.

## R3 — binary evidence is incomplete component truth
Mitigation:
Observed/Derived/Declared/Unknown;
manual declaration;
completeness model;
never “SBOM complete” without scoped proof.

## R4 — compliance liability
Mitigation:
engineering evidence only;
no legal/compliance verdict;
CRA timing used as context, not certification marketing.

## R5 — WebView/platform differences
Windows Tier 1; screenshot/E2E; conservative CSS; macOS/Linux release smoke.

## R6 — scope creep from expert ideas
Mitigation:
Post-MVP namespace E1/E2/E3/GX;
candidate ≠ commitment;
four verbs;
Scope Creep questions;
ADR for new domain dimension.

## R7 — name/trademark
Working brand only until formal USPTO/EUIPO/CNIPA/domain clearance.

## R8 — market research overclaim
Negative searches and vendor claims can be mistaken for universal facts.

Mitigation:
market verification register with VERIFIED / INFERENCE / UNVALIDATED / DISPUTED labels.

## R9 — `.su` evidence scope explosion
Risk:
worst-stack column can accidentally grow into full static call-graph subsystem.

Mitigation:
ADR scope explicitly excludes full graph.

## R10 — report/export dependency inflation
Candidate libraries for XLSX/DOCX/charting can violate dependency discipline.

Mitigation:
no dependency enters baseline until feature signal + spike + dependency review.

## R11 — CI/cloud trust boundary
Official Actions/team policy would introduce secrets/network/distribution concerns.

Mitigation:
GX only after V1/team signal; new network capability and secret model require review/ADR.

## R12 — advanced ABI analysis
DWARF/toolchain variance can turn an apparently simple gate into an L-sized subsystem.

Mitigation:
hold full ABI Gate until mature symbol diff + real incident evidence.
