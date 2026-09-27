---
title: "Stage Gates and Product Maturity Model"
doc_id: "FS-DEL-006"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Product / Delivery"
last_updated: "2026-09-26"
---

# Stage Gates

本文件把完整生命周期压缩成可以直接执行的阶段门。

## G0 — Problem Baseline

进入条件：
- 用户群体明确；
- 问题明确；
- 当前替代方案明确；
- 产品边界明确。

FirmwareSight：**PASS**

---

## G1 — Technical Validation

目标：
证明核心技术链成立，而不是验证商业产品。

最小链：

```text
Real ELF Fixture
→ Parse
→ Normalize
→ BuildSnapshot
→ CLI JSON
→ Minimal Desktop Summary
```

必须：
- Core/CLI/Desktop 事实一致；
- 真实 fixture 可重复；
- parser 对坏输入不崩溃；
- UI 不阻塞；
- 数据模型可继续支撑 Compare/Gate。

UI 深度：
**极简技术 UI。**

不进入：
- 最终设计系统；
- 商业包装；
- 账号；
- Cloud；
- SBOM；
- updater。

退出：
G1 PASS 后才能进入 Product MVP。

---

## G2 — Product MVP

目标：
验证真实用户是否能独立完成一个完整 Firmware Release 工作流。

必须闭环：

```text
Start / Project
→ Analyze
→ Compare
→ Release Gate
→ Release Bundle
```

UI 深度：
**Minimum Credible Product UI**

要求：
- 完整；
- 一致；
- 清楚；
- 能让陌生用户操作；
- 没有 Demo 感；
- 但不是最终商业视觉。

关键指标：
- Import activation >80%
- Time to First Value <60s
- Analyze→Compare completion >=60%
- Gate meaning comprehension >80%
- 至少 30% 用户发现真实新信息
- 至少 5/8 用户有再次使用意愿
- 至少 3 个明确付费意愿，或 1 个团队 Pilot

退出：
通过真实用户验证，而不是内部自评。

---

## G3 — Productization

目标：
把“已证明有价值的 MVP”变成可以长期依赖的软件。

核心工作：
- compatibility matrix；
- parser fixture expansion；
- crash/recovery；
- migrations；
- diagnostics；
- complete design system；
- onboarding；
- project/history model；
- documentation；
- install/sign/update readiness。

退出：
陌生用户无需开发团队陪同即可稳定使用。

---

## G4 — Private Beta

目标：
真实项目持续使用。

建议规模：
- 20–50 名用户；
- 3–5 个团队。

必须：
- installers；
- signing/notarization strategy；
- diagnostics；
- privacy；
- changelog；
- update path；
- beta support loop。

开始验证：
Free / Pro / Team 的真实付费边界。

---

## G5 — Release Candidate

定义：

> FirmwareSight 已经可以被真实用户用于生产 Release，而不只是“功能齐”。

必须：
- P0 workflow 零阻断 bug；
- supported format matrix green；
- migrations verified；
- security/capability review；
- signed package；
- install/upgrade/recovery；
- Getting Started / Support Matrix / Known Limitations / Privacy。

---

## G6 — GA 1.0

FirmwareSight v1.0 完整性定义：

```text
Analyze
Compare
Release Gate
Release Bundle
History
CLI
Installer
Signing
Update
Documentation
Commercial Distribution
```

“完整产品”不等于“功能最多”。

GA 的意义是：

> 在明确范围内，FirmwareSight 已经可发布、可收费、可维护、可长期依赖。

---

## Post-GA

之后再逐步验证：

- Component Evidence；
- SBOM；
- vulnerability intelligence；
- CI integrations；
- team policy；
- collaboration。

不自动扩展到：
- IDE；
- flashing；
- HIL；
- OTA；
- device fleet；
- cloud dashboard；
- plugin marketplace；
- AI judge。


## v0.4.0 Canonical Stage Mapping

权威 stage identifiers：

```text
G0 Problem Baseline        — passed
V0 Workflow Prototype     — next, parallel with P0 after authorization
P0 Technical Vertical Slice — next, parallel with V0 after authorization
G1 = V0 PASS + P0 PASS
P1 Analyzer
P2 Compare
P3 Release Gate
P4 Release Bundle
G2 Product MVP Candidate
V1 Own-artifact External Validation
P5 Productization
B1 Private Beta
RC1 Release Candidate
GA1 General Availability
```

自然语言 `Problem Validation / Technical Validation / Product MVP / Productization / Beta / RC / GA`
只作描述；自动化、任务、交付和 Agent 必须使用上述 identifier。

V1 比例指标必须报告实际 numerator/denominator；Return intent 阈值统一为 `>=62.5% 且至少 5 人`，不再写死成只适用于 n=8 的 `5/8`。
