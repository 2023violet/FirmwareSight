---
title: "Product Roadmap v0.3"
doc_id: "FS-DEL-001"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product / Delivery"
last_updated: "2026-09-26"
---

# Product Roadmap

FirmwareSight 的路线不再把“写完功能”当作产品完成。

正式采用：

```text
Problem Baseline
→ Technical Validation
→ Product MVP
→ External Validation
→ Productization
→ Private Beta
→ Release Candidate
→ GA 1.0
→ Growth
```

---

## P0 — Foundation / Technical Vertical Slice

目标：
验证架构、Parser、Core、CLI、Desktop 的最小闭环。

范围：

```text
ELF fixture
→ SHA-256
→ object parser
→ normalized BuildSnapshot
→ CLI JSON
→ minimal Desktop Summary
```

重点：
- Rust workspace；
- artifact parser；
- normalized domain model；
- typed errors；
- tracing；
- minimal Tauri shell；
- fixture/golden tests。

退出：
G1 Technical Validation PASS。

---

## P1 — Analyzer

目标：
构建第一个真正可使用的 Analyze 工作区。

包含：
- Identity；
- Memory；
- Sections；
- Symbols；
- Evidence；
- Search/filter/sort；
- capability reporting。

此阶段开始形成 Minimum Credible UI，而不是最终商业视觉。

---

## P2 — Compare

包含：
- Build A/B；
- FLASH/RAM delta；
- section delta；
- symbol added/removed/changed；
- top growth contributors；
- evidence drill-down；
- exportable diff。

---

## P3 — Release Gate

包含：
- project policy；
- Git provenance；
- memory budgets；
- required artifacts；
- version/tag consistency；
- PASS / REVIEW / BLOCK / N/A；
- evidence + remediation。

---

## P4 — Release Bundle

包含：
- bundle preview；
- SHA256SUMS；
- release-manifest；
- analysis；
- diff；
- release-report；
- conflict-safe export。

P1–P4 完成后：

# MVP Candidate

---

## V1 — External MVP Validation

不是继续开发功能。

使用真实工程师 + 真实 artifact 验证：
- workflow；
- UX；
- value；
- repeat usage；
- payment signal。

不通过：
回到对应问题层修正。

通过：
进入 Productization。

---

## P5 — Productization

重点：

### Compatibility
- GNU Arm Embedded；
- Zephyr/ESP-IDF 等实际生态；
- 商业 toolchain Beta；
- support matrix。

### Reliability
- fuzz；
- malformed corpus；
- recovery；
- migrations；
- diagnostics；
- import/export integrity。

### Product UX
- final design system；
- onboarding；
- sample project；
- project history；
- settings；
- accessibility；
- dark theme only when core UI stable。

### Documentation
- Getting Started；
- Concepts；
- CLI；
- Config；
- Toolchain Support；
- Troubleshooting。

### Distribution
- installers；
- signing；
- package QA；
- updater readiness。

---

## B1 — Private Beta

建议：
20–50 用户，3–5 团队。

重点：
- real recurring use；
- support cost；
- compatibility gaps；
- crash/diagnostics；
- pricing experiment；
- Free/Pro boundary。

---

## RC1 — Release Candidate

Production readiness：
- zero P0 blocker；
- package/sign/update；
- migration/recovery；
- security review；
- docs complete；
- supported matrix verified。

---

## GA 1.0 — General Availability

v1.0 目标：
FirmwareSight 已经在明确范围内：
- 可发布；
- 可收费；
- 可维护；
- 可升级；
- 可支持。

---

## Growth — Post 1.0

按真实付费需求逐步增加：

- Component Evidence；
- CycloneDX/SPDX；
- vulnerability intelligence；
- CI integrations；
- Team Policy；
- shared release evidence。

任何 Growth 功能都不能反向破坏：
- local-first；
- deterministic core；
- evidence provenance；
- explicit unknown。


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
