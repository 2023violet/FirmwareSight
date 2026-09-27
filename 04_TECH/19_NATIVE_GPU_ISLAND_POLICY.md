---
title: "Native and GPU Island Policy"
doc_id: "FS-TECH-020"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Native / GPU Island Policy

## Default

No wgpu.
No native custom renderer.
No whole-app GUI rewrite.

## First escalation ladder

When UI is slow:

1. profile；
2. reduce IPC payload；
3. query/paginate in SQLite；
4. memoize React where evidence supports；
5. virtualize rows；
6. Canvas/SVG optimization；
7. Web Worker only for frontend-only compute；
8. Rust-side optimized computation；
9. only then evaluate Native/GPU Island。

## Native Island trigger

A wgpu/native island requires:
- reproducible benchmark；
- target hardware；
- current approach misses explicit performance budget；
- proof that data/query optimization cannot solve it；
- ADR；
- contained API boundary。

## Examples where it might become valid

- millions of glyph/rect primitives；
- interactive treemap at very high scale；
- future binary visualization requiring continuous GPU rendering。

Normal tables/forms/diff panels never justify it.

## Architectural requirement

Native Island is a leaf adapter.
It must not own:
- Project state；
- Gate logic；
- Evidence rules；
- persistence；
- release logic。

Removing the island must not invalidate Core.
