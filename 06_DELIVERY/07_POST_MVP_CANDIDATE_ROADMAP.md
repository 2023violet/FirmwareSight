---
title: "Post-MVP Candidate Roadmap"
doc_id: "FS-DEL-008"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product / Delivery"
last_updated: "2026-09-27"
---

# Post-MVP Candidate Roadmap

## Governing rule

This registry absorbs the three expert exploration rounds and the 27 directional decisions.

It **does not expand the current MVP**.

Old exploration labels such as “P1/P2/P4” are retained only in source/audit documents. They must not be used as current scheduling authority because P1–P4 are already canonical MVP implementation phases.

## E1 — Low-cost workflow depth

Entry condition:
- V1 evidence demonstrates recurring use/value;
- item passes Scope Creep and dependency review.

Candidate priority:

### E1-A — Enhanced self-contained HTML
Why:
- strong fit with Bundle;
- local-first/shareable;
- existing report crate;
- useful beyond app without cloud account.

### E1-B — CSV + Markdown export
Why:
- low implementation risk;
- machine/human portability;
- no heavy renderer dependency required.

### E1-C — CLI/config workflow recipes
- stable JSON examples;
- config JSON Schema distribution;
- policy examples;
- Git hook recipes;
- VS Code task recipes.

This increases workflow reach without creating a new service/platform.

### E1-D — Component version drift
Start narrowly with ESP-IDF `dependencies.lock`, whose exact-version/reproducibility semantics are well defined.
Do not write a “universal lockfile parser”.

### E1-E — fixture-driven toolchain expansion
Keil/ArmClang first only when real artifacts/fixtures are available.

## E2 — Evidence depth

### E2-A — `.su` worst-stack evidence
Status: strongest new category signal from this round.

Requires ADR before implementation because it extends domain/storage/diff/evidence shape.

Scope proposal:
- per-function worst-stack value;
- sortable column;
- declared quota/context where supplied;
- Unknown when `.su` is absent;
- deterministic Gate rule only if threshold evidence exists.

Explicitly excluded:
- full call-graph reconstruction/visualization.

### E2-B — SBOM evidence completeness model
Define the evidence model first:
- observed;
- derived;
- declared;
- unknown;
- source-level counts/coverage facts.

Do not turn it into a “compliance score”.

Completeness definition precedes adding another SBOM output format such as SPDX 3.0.

### E2-C — Release symbol archive
Potentially low cost:
- explicit selection;
- manifest role/kind;
- hash;
- size warning;
- version/build mapping.

No separate “symbol cloud/library” product page by default.

### E2-D — toolchain flag evidence
Ride along Build Identity; no standalone product project.

### E2-E — retention semantics
Clarify what History/Bundle retains and verifies before building a retention management subsystem.

## E3 — Conditional advanced experience

- treemap only if measured user task improves over ranked table/bars;
- history trend visualization after real History usage;
- watch mode only after repeated-analysis frequency is proven;
- XLSX after spreadsheet handoff signal;
- IAR/CSV input only after fixtures and user pull;
- DWARF-assisted ABI detail only after real upgrade incidents.

## GX — Growth / ecosystem

- official GitHub Action / GitLab integration;
- team policy distribution;
- CI comments/checks;
- editor extension only if tasks/recipes are insufficient;
- full ABI compatibility Gate only after strong evidence;
- DOCX only if enterprise handoff workflows demand it.

GX must preserve:
- local-first core;
- deterministic trusted path;
- evidence provenance;
- explicit Unknown.

## Default no

- native PDF renderer;
- PPTX/RTF/ODF;
- LSP integration;
- full call graph bundled into `.su`;
- binary/partition reverse-engineering suite;
- OTA platform;
- cloud portal clone;
- AI release verdicts;
- independent generic changelog generator.

## Interview triggers

Before promoting HOLD/E3/GX items, ask:
- Who consumes the exported analysis?
- Which format do they actually open/forward?
- How often do they repeat analysis?
- Do they need CI to block or only report?
- What actual `.su`/stack failure happened?
- Have they needed a symbol file months after release?
- Does CRA/customer review create real purchase timing pressure?
