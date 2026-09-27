---
title: "v0.5 Adoption Decision Register"
doc_id: "FS-AUDIT-004"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Audit"
last_updated: "2026-09-27"
---

# v0.5.0 Adoption Decision Register

Priority here means **decision priority**, not current implementation order.

## A — Adopt into baseline now

| Item | Why |
|---|---|
| complete UI-07 source asset | source now available; closes prior asset gap |
| code-area governance plan | removes repository ambiguity before implementation |
| four-verb extension rule | prevents feature sprawl |
| Post-MVP namespace E1/E2/E3/GX | resolves stale P1/P2 label collision |
| deterministic report/export discipline | consistent with evidence-first/reproducibility |
| chart/table presentation rules | keeps UI instrument-like |
| capability banner one-source-one-slot | evidence availability remains single truth |
| change facts before Gate verdict | separates facts from policy judgment |
| interview question bank | validates weak/conditional candidates efficiently |
| market verification labels | prevents research overclaim |

## B — Accepted candidate direction, first Post-MVP pool

- enhanced self-contained HTML;
- CSV / Markdown export;
- CLI/config/schema/workflow recipes;
- component drift starting with ESP-IDF lock evidence;
- fixture-driven Keil/ArmClang expansion;
- optional baseline pin if History usage validates it.

Reason:
high architecture fit, low/medium cost, attaches to existing four verbs.

## C — Accepted but conditional / evidence-depth

- `.su` worst-stack evidence — strong signal but requires ADR/domain change;
- SBOM evidence-completeness model;
- release symbol archive;
- evidence retention refinements;
- toolchain flag evidence;
- History trends/treemap;
- watch;
- XLSX;
- IAR/CSV adapter expansion;
- ABI detail/gate;
- official CI Action/GitLab;
- DOCX;
- editor extension.

Reason:
valuable enough to retain, but either unvalidated, costly, dependency-heavy, or trust-boundary expanding.

## D — Not adopted

| Item | Reason |
|---|---|
| native PDF layout engine | HTML print view covers archive need with lower deterministic/support cost |
| PPTX/RTF/ODF export | no core release-evidence value/signal |
| generic changelog generator as feature pillar | incumbent platform support; weak differentiation |
| full call graph bundled with `.su` | separate L-sized problem; scope explosion |
| LSP | turns product toward IDE/tooling surface without evidence |
| binary/partition reverse-engineering diff | collides with Non-goals; low incremental release value |
| OTA platform | explicit scope boundary |
| cloud portal clone | undermines current positioning and adds premature trust/ops burden |
| full environment fingerprint by declared inputs | unbounded maintenance/weak observability |
| AI verdict/AI trusted generation | violates ADR-0005 deterministic trusted core |

## Disputed claims excluded from baseline facts

- exact “≥7 reinventions over 17 years” metric;
- “ranked lists are most popular” as measured fact;
- “XLSX/DOCX are fake demand”;
- “no report monetization precedent anywhere”;
- old exact MemBrowse price/unit claims;
- “cloud competitors cannot export self-contained reports”;
- “no open-source CLI+Action precedent”.
