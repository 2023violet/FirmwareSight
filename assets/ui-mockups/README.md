---
title: "UI Mockup Asset Register"
doc_id: "FS-DESIGN-ASSET-001"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Design"
last_updated: "2026-09-27"
---

# UI Mockup Asset Register

v0.5.0 contains the **complete seven-screen** accepted reference set.

These screenshots are authoritative for composition, information hierarchy, density and interaction expression, but they remain below `DESIGN.md`, tokens and accepted ADRs in the authority chain.

## Included sources

1. `FS-UI-01-Overview.png`
2. `FS-UI-02-Analyze.png`
3. `FS-UI-03-Compare.png`
4. `FS-UI-04-Release-Gate.png`
5. `FS-UI-05-Bundle-History.png`
6. `FS-UI-06-Parse-Failure.png`
7. `FS-UI-07-Unknown-Dependency.png`

All seven source PNGs are 1440×900.

## UI-07 semantic lock

The seventh screen freezes the intended evidence/dependency presentation:
- `Observed`, `Declared`, `Unknown` are visibly distinct;
- `Unknown` uses neutral treatment and facts, never guessed version data;
- manual declaration is explicitly marked `Declared`;
- a declaration never overwrites an observed fact;
- dependency evidence remains subordinate to the four product verbs, not a new top-level product area.
