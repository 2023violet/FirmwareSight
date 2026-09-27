---
title: "Risks and Assumptions"
doc_id: "FS-RSCH-004"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-26"
---

# Risks & Assumptions

## R1 — 用户是否愿意为 release tooling 付费
**Status:** Unvalidated  
Mitigation:
- Free analyzer；
- prototype interview；
- ask for payment, not compliments。

## R2 — Toolchain fragmentation
不同 linker/map 格式差异大。

Mitigation:
- support matrix；
- adapters；
- no universal parser claim；
- fixture-driven expansion。

## R3 — ELF data ≠ source dependency truth
Binary 不一定包含完整 component metadata。

Mitigation:
- evidence classes；
- Unknown；
- multiple source fusion；
- manual declarations。

## R4 — Compliance positioning creates liability
Mitigation:
- engineering evidence positioning；
- no legal verdict；
- explicit boundaries。

## R5 — Tauri WebView differences
跨平台 WebView 可能有渲染差异。

Mitigation:
- conservative CSS；
- Windows-first QA；
- platform screenshot/e2e；
- no bleeding-edge browser APIs without fallback。

## R6 — Product becomes “feature pile”
Mitigation:
- four verbs: Analyze/Compare/Gate/Release；
- roadmap gates；
- ADR for scope expansion。

## R7 — Name conflict
FirmwareSight 尚未做法律级 clearance。

Mitigation:
- before public launch complete formal screen。


## v0.4 Risk Response Update

R1 commercial/value uncertainty is pulled forward:
- V0 validates workflow/value before full Product MVP build；
- P0 runs in parallel to validate feasibility；
- V1 validates own-artifact recurrence/payment signal；
- core workflow is not hidden behind an entitlement wall during validation。

R2 toolchain fragmentation remains intentional and fixture-gated.
R3 binary-only component evidence remains incomplete by nature; future SBOM must surface Unknown/coverage rather than imply completeness.
