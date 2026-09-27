---
title: "Applying the 2026 Rust Product Census to FirmwareSight"
doc_id: "FS-RSCH-005"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-26"
---

# Applying the Rust Product Census to FirmwareSight

## 1. Source conclusion

The supplied Rust product research converges on:

> Core-first, Adapter-driven, Product-specific Shell.

It also explicitly warns against:
- Tokio everywhere；
- Tauri everywhere；
- SQLx everywhere；
- wgpu everywhere。

This document translates that general rule into FirmwareSight-specific choices.

## 2. Evidence → decision mapping

| Research signal | FirmwareSight interpretation |
|---|---|
| Serde is near-universal across product categories | adopt Serde at contracts/boundaries |
| CLI is a strong Rust product form | ship `fwsight` as first-class interface |
| Tauri/Web leads reviewed ordinary desktop products | Tauri 2 + React/TS |
| Native Rust GUI fits specialized engineering/custom render products | not needed for main shell |
| Tokio is dominant once real async appears | keep at app/IO edge, not Core |
| Tracing appears strongly in complex products | adopt from day one |
| SQLite is common in stateful local desktop products | SQLite is local source of truth |
| rusqlite fits local-only SQLite | choose rusqlite + bundled |
| Reqwest/Rustls are default once HTTP exists | pre-approve but do not install until network exists |
| wgpu is an important GPU convergence point | pre-approve as Native Island after profiling |
| mature desktop apps plan updater/release pipeline | design release/signing/updater now; enable later |
| complex products use workspaces | use a small 4-crate workspace |

## 3. Where FirmwareSight deliberately differs from a generic stack

### No direct Reqwest/Rustls in MVP
Because FirmwareSight core use case is offline.

### No SQLx
Because there is no async server/multi-DB requirement.

### No baseline wgpu
Because tables and diff inspection are not GPU workloads.

### No Tokio in Core
Because parsing/diff/gate are deterministic synchronous domain operations.

### No giant workspace
Because architecture should reflect current product, not hypothetical scale.

## 4. Research limitations preserved

The GitHub census measures:
- high-star open-source engineering practice

It does **not** measure:
- commercial revenue；
- all closed-source software；
- final B2B purchasing behavior。

Therefore it is used as an engineering prior, not as proof of business success.
