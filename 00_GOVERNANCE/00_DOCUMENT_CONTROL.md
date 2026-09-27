---
title: "Document Control Standard"
doc_id: "FS-GOV-001"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-26"
---

# 文档控制规范

## 1. 权威性

项目文档按以下优先级解释：

1. 已接受 ADR
2. 当前版本 PRD / Architecture / Design System
3. `.ai/DECISIONS.md`
4. 当前 Active Task
5. 其他研究或历史记录

研究资料不能覆盖已冻结决策；需要变更时必须写 ADR。

## 2. 编号

- Governance：`FS-GOV-*`
- Product：`FS-PRD-*`
- Brand：`FS-BRAND-*`
- Design：`FS-DESIGN-*`
- Technical：`FS-TECH-*`
- Engineering：`FS-ENG-*`
- Compliance：`FS-COMP-*`
- Research：`FS-RSCH-*`
- ADR：`ADR-xxxx`

## 3. 状态

允许状态：
- `DRAFT`
- `REVIEW`
- `BASELINE`
- `SUPERSEDED`
- `ARCHIVED`

禁止直接删除历史决策文档；用 `SUPERSEDED` 并链接替代文档。

## 4. 版本

项目基线采用 SemVer：
- MAJOR：产品边界或核心架构重置；
- MINOR：新增完整规范或阶段能力；
- PATCH：澄清、错误修复、不改变语义。

## 5. 写作规范

- 先给结论，再给原因。
- 把事实、假设、决策分开。
- 使用“必须 / 应该 / 可以 / 不得”表达约束强度。
- 不使用“智能地”“自动地”“完美”等不可验证形容词。
- 所有安全、合规、兼容性主张都必须带边界。
- 任何外部事实在研究文档中附来源与检索日期。

## 6. 变更记录

每次 baseline 变更至少更新：
- `CHANGELOG.md`
- `.ai/CURRENT_STATE.md`
- `.ai/DECISIONS.md`
- 受影响文档
- 如涉及决策，新增 ADR


## v0.4 Audit Source Authority

`10_AUDIT/SOURCE_REVIEWS/` 保存外部 AI 团队审查/提案原件，仅用于追溯。

它们低于 Accepted ADR、当前 PRD/Architecture/Design、`.ai/DECISIONS` 和 Active Task，不可直接覆盖现行基线。


## v0.5 Expert source authority

`10_AUDIT/SOURCE_REVIEWS_V05/` preserves external expert-team source artifacts verbatim.

These sources do not override accepted ADR/current product/architecture/design documents.
Any externally researched conclusion promoted into baseline must have a disposition in the v0.5 audit/market-verification registers.
