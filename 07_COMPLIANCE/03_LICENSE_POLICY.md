---
title: "Licensing Policy"
doc_id: "FS-COMP-004"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Legal/Engineering"
last_updated: "2026-09-26"
---

# Licensing Policy

## Product code

当前基线：
**Private / proprietary by default until commercial model is explicitly decided.**

不要提前添加 MIT/Apache/GPL LICENSE 到主仓库。

## Third-party

每个 dependency 必须记录：
- name；
- version；
- license；
- source；
- redistribution obligations。

## Preferred licenses

一般优先：
- MIT
- Apache-2.0
- BSD-2/3-Clause
- ISC
- MPL-2.0 需按场景审查

GPL/AGPL/copyleft 进入商业桌面分发前必须专项评估。

## Free tier ≠ open source

免费 Analyzer 的商业策略与是否开源是两个独立决策。

未来若开源某个 parser/core，必须 ADR。
