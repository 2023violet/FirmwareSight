---
title: "Licensing Policy"
doc_id: "FS-COMP-004"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Legal/Engineering"
last_updated: "2026-10-18"
---

# Licensing Policy

## Product code

当前基线（2026-10-18 起，[ADR-0035](../09_ADR/ADR-0035-apache-2-open-source-license.md)）：
**FirmwareSight 以 Apache License 2.0 开源。** 根目录 `LICENSE` 为许可证原文，`NOTICE` 为项目声明；贡献按 Apache-2.0 第 5 条入站即出站。

（此前的基线是“在商业模式明确前保持私有/专有”，已由 Owner 决定并经 ADR-0035 取代。）

## Third-party

每个 dependency 必须记录（二进制发布附带的 `THIRD_PARTY_LICENSES.md` 由 `scripts/gen_third_party_licenses.py` 从锁定的依赖图生成）：
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

开源决定已由 ADR-0035 作出；此后对许可证的任何变更同样必须 ADR。
