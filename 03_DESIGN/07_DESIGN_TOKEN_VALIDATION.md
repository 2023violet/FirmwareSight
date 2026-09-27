---
title: "Design Token Validation"
doc_id: "FS-DESIGN-008"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Design"
last_updated: "2026-09-27"
---

# Design Token Validation

## Purpose

v0.4.0 将 `assets/design-tokens.json` 升为 v0.2.0 baseline-authoritative，并冻结 ADR-0018。

## Preservation rule

v0.1.0 已有 14 个语义色角色保持不动：
- canvas/surface/subtle
- primary/secondary text
- border
- accent
- pass/review/block/unknown
- added/removed/changed

v0.2.0 只补齐组件实现所需的治理缺口。

## Added groups

- accent hover / pressed / disabled
- focus ring
- disabled text
- on-accent / on-status text
- stronger review text
- diff background tints
- overlay shadows
- pill radius
- table row density
- layout widths
- motion timing/ease
- chart single-blue sequence

## Contrast notes

Validated pair examples:
- review foreground on white: AA-sized body text acceptable
- review strong is preferred for compact 12px table status text
- added/removed/changed foregrounds on their tinted backgrounds meet body-text contrast target
- `text.disabled` is **not** permitted as informative body text; it is disabled-control affordance only

## Chart rule

Chart blue sequence is categorical/ordinal visual support, not a contrast-coded status system.

Do not claim adjacent blue swatches meet 3:1 color-to-color contrast.
Charts must encode meaning through:
- labels
- values
- position
- bar length/shape
in addition to color.

## Mechanical validation

G2 前应增加：
- token schema validation
- hard-coded color lint
- forbidden gradient/backdrop/shadow lint
- focus-visible component tests
- five-state component test matrix
