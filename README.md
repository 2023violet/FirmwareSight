---
title: "FirmwareSight Project Baseline"
doc_id: "FS-ROOT-README"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-27"
---

# FirmwareSight v0.5.0

**Embedded Firmware Release Workbench**  
**Know exactly what ships.**

v0.5.0 is the **Expert Review Verification + Whole-Product Consolidation Baseline**.

It inherits v0.4.0 architecture, product scope, Gate semantics and UI authority, then integrates the new expert round without allowing research ideas to silently become MVP requirements.

## What v0.5.0 changes

### 1. Complete UI reference
The seventh source screen `FS-UI-07-Unknown-Dependency.png` is now included.
The accepted reference set is complete: 7 × 1440×900.

### 2. Whole-product baseline
`PRODUCT_BASELINE.md` now provides one authoritative cross-domain reading of:
- positioning/users;
- modules;
- scope;
- architecture;
- UI;
- lifecycle;
- commercial/compliance posture.

### 3. Expert recommendations normalized
Three exploration rounds are retained, but:
- current MVP remains unchanged;
- stale future `P1/P2/P4` labels are translated to `E1/E2/E3/GX`;
- candidate ≠ commitment;
- Active Task remains NONE.

### 4. Research verification
The strongest conclusions are supported:
- embedded memory/stack visualization is a long-running real workflow;
- CI/history/PR integration is a real product pattern;
- ESP-IDF lockfiles are a sound first component-drift source;
- release symbol retention solves real downstream debugging needs;
- CRA reporting obligations are current context.

The following are deliberately **not** frozen as facts:
- exact “7 tools over 17 years” metric;
- ranked lists being quantitatively “most popular”;
- XLSX/DOCX being fake demand;
- universal absence of export monetization;
- stale exact MemBrowse price/unit claims;
- “cloud cannot export self-contained artifacts”;
- “no open-source CLI+Action precedent”.

### 5. Code area is planned before code exists
See:
`05_ENGINEERING/08_CODE_AREA_PLAN.md`

The plan preserves:
- four library-crate Phase-0 budget;
- pure Rust Core;
- thin Tauri shell;
- single frontend IPC boundary;
- fixture/golden discipline;
- root tokens/schema authority.

## Current scope

Current MVP still centers:

```text
Analyze → Compare → Gate → Release
```

Current next authorizable validation:

```text
V0 Workflow Prototype
        +
P0 Technical Vertical Slice
        ↓
       G1
```

Nothing in E1/E2/E3/GX is automatically authorized.

## Post-MVP candidate horizons

### E1
low-cost workflow depth:
enhanced HTML, CSV/MD, recipes/schemas, narrow component drift, fixture-driven toolchain expansion.

### E2
evidence depth:
`.su` stack evidence, SBOM completeness, symbol archive, retention/toolchain evidence.

### E3
conditional advanced experience:
watch, trends/treemap, XLSX, deeper adapters/ABI details.

### GX
Growth/ecosystem:
official CI integrations, team policy, editor extension, full ABI Gate, DOCX if proven.

## Read order

1. `README.md`
2. `PRODUCT_BASELINE.md`
3. `DESIGN.md`
4. `BASELINE.yaml`
5. `10_AUDIT/02_V0.5_EXPERT_REVIEW_RESOLUTION.md`
6. `10_AUDIT/03_ADOPTION_DECISION_REGISTER.md`
7. `05_ENGINEERING/08_CODE_AREA_PLAN.md`
8. `06_DELIVERY/07_POST_MVP_CANDIDATE_ROADMAP.md`
9. `08_RESEARCH/09_EXPERT_ROUND_MARKET_VERIFICATION_2026-09-27.md`
10. `09_ADR/ADR-0024-post-mvp-candidate-governance.md`
11. `.ai/ACTIVE_TASK.md`
