---
title: "ADR-0021 Memory Accounting"
doc_id: "ADR-0021"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
---

# ADR-0021 — Firmware Memory Accounting

## Status
Accepted.

## Decision
FirmwareSight 分开计算：
- nonvolatile/load image footprint
- runtime RAM footprint

`.data` 可以同时贡献两者；debug 默认排除 device budget；custom section 由 region/segment evidence 决定。

Hard Gate 只在 region/budget/evidence 足够时产生。

## Consequences
实现比 section-name 求和复杂，但避免错误 BLOCK/PASS。

## Revisit trigger
新增工具链 memory model 时以 adapter evidence 扩展，不改变双预算语义。
