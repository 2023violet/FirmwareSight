---
title: "EU Cyber Resilience Act Context"
doc_id: "FS-COMP-003"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Compliance"
last_updated: "2026-09-26"
---

# CRA Context

> 工程背景材料，不是法律意见。

## 已确认时间点

根据欧盟委员会截至 2026-09 的公开材料：

- CRA 于 2024-12-10 生效；
- 报告义务从 **2026-09-11** 开始适用；
- 主要义务从 **2027-12-11** 全面适用。

报告义务包括对 actively exploited vulnerabilities 和 severe incidents 的通知要求。

## 与 FirmwareSight 的关系

CRA 强化了制造商对：
- 软件组件；
- vulnerability handling；
- support period；
- reporting；
- technical documentation；
等信息的工程治理需求。

FirmwareSight 可以支持：
- release provenance；
- component evidence；
- SBOM export；
- evidence retention。

但不直接完成：
- 法律适用性判断；
- conformity assessment；
- ENISA/CSIRT reporting；
- 风险评估结论。

## Design consequence

CRA 不是核心产品存在的唯一理由。

即使法规变化，Analyze/Compare/Gate/Release 仍必须有独立工程价值。

## Primary sources

- European Commission CRA overview:
  https://digital-strategy.ec.europa.eu/en/policies/cyber-resilience-act
- Reporting obligations:
  https://digital-strategy.ec.europa.eu/en/policies/cra-reporting
- Regulation (EU) 2024/2847:
  https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R2847
