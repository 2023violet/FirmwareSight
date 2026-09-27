---
title: "Baseline Changelog"
doc_id: "FS-ROOT-CHANGELOG"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-27"
---

# Changelog

## 0.5.0 — 2026-09-27

### Expert review verification
- reviewed three Post-MVP exploration rounds and 27 adjudicated questions;
- independently verified the strongest current market claims;
- downgraded overbroad/negative-search claims;
- removed stale/unverified vendor pricing from baseline facts;
- corrected stage-name conflict against v0.4.

### Whole-product consolidation
- added `PRODUCT_BASELINE.md`;
- added Product Module Map and Capability Portfolio;
- froze reconstructed Code Area Plan (`FS-ENG-009`);
- added Post-MVP candidate roadmap using E1/E2/E3/GX namespace;
- added milestone/deliverable matrix and interview question bank;
- added risk/dependency register.

### UI
- integrated `FS-UI-07-Unknown-Dependency.png`;
- seven-screen source set is now complete;
- added future presentation rules without authorizing future features.

### Research decisions
- visualization: category need supported; exact reinvention count downgraded;
- XLSX/DOCX: `UNVALIDATED`, not “fake demand”;
- native PDF: rejected in favor of HTML print;
- component drift: ESP-IDF lockfile first candidate;
- `.su`: strong evidence-depth candidate, future ADR required;
- symbol archive: accepted candidate;
- CI/history integration: real market pattern, Growth candidate;
- generic changelog generator: not adopted as product pillar.

### Governance
- ADR-0024 establishes candidate namespace and prevents MVP expansion.
- Active Task remains NONE.
- v0.4 V0 + P0 parallel next-step decision is preserved.

## 0.4.0 — 2026-09-27
Audit resolution + UI baseline freeze; Gate semantics, memory accounting, typed IPC, strict schemas, six source UI screens.

## 0.3.0 — 2026-09-26
Lifecycle baseline.

## 0.2.0 — 2026-09-26
Technical baseline.

## 0.1.0 — 2026-09-26
Initial product baseline.
