---
title: "Capability Portfolio and Adoption Status"
doc_id: "FS-PRD-010"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Product"
last_updated: "2026-09-27"
---

# Capability Portfolio

## Status classes

- **BASELINE** — frozen current product/MVP capability or governing rule.
- **E1** — first Post-MVP candidate horizon after V1 evidence.
- **E2** — evidence-depth candidate; usually model/ADR work.
- **E3** — conditional advanced experience.
- **GX** — Growth/ecosystem candidate.
- **HOLD** — no direct demand signal; do not build until evidence appears.
- **REJECT** — conflicts with scope/value/architecture.

## Current baseline

- artifact import and immutable identity;
- Analyze tables/ranked bars needed for existing facts;
- Compare old/new/delta + contributors;
- five-state Gate;
- Bundle HTML/JSON + hashes;
- local History foundation;
- shared `fwsight` CLI;
- seven-screen UI baseline.

## E1 — low-cost workflow depth

Candidate pool, not commitment:
- enhanced self-contained HTML report;
- CSV export;
- Markdown export;
- CLI pipeline recipes and schema examples;
- config JSON Schema distribution;
- policy preset examples;
- Git hook recipes;
- VS Code task recipes;
- component version drift, starting with ESP-IDF `dependencies.lock` if V1 confirms value;
- explicit baseline pin in History, if History usage validates it;
- fixture-driven Keil/ArmClang input expansion.

## E2 — evidence depth

- `.su` per-function worst-stack evidence: **strong category evidence**, ADR required;
- SBOM evidence-completeness model before adding more SBOM formats;
- optional release symbol archive;
- toolchain flag summary as Build Identity evidence;
- evidence retention semantics/refinement.

## E3 — conditional advanced experience

- treemap only if it beats table/ranked bars for a measured task;
- history trend views with strict axis/order rules;
- watch mode only if V1 shows frequent repeat analysis;
- XLSX only if interview/customer workflow demands spreadsheet handoff;
- IAR/CSV adapters only after fixture/support evidence.

## GX — growth/ecosystem

- official GitHub Action / GitLab template/package;
- team policy distribution;
- CI review surfaces;
- editor extension only if recipes prove insufficient;
- DOCX only if enterprise handoff signal is explicit;
- full ABI gate only after evidence of real ABI incidents + mature diff foundation.

## HOLD / REJECT

### HOLD
- DOCX;
- XLSX until signal;
- SPDX 3.0 export until evidence completeness is defined;
- non-Git provenance until demand;
- advanced ABI detail.

### REJECT / default no
- native-layout PDF engine (use HTML print view);
- PowerPoint/RTF/ODF export;
- full static call-graph visualization bundled into `.su`;
- LSP as an integration strategy;
- binary/partition reverse-engineering diff;
- OTA platform/transport;
- full build-environment fingerprint with unbounded declared inputs;
- independent changelog generator as a product pillar;
- AI verdicts or AI-generated trusted release decisions;
- cloud portal clone as a prerequisite for product value.
