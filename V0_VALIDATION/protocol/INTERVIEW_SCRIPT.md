---
title: "V0 Interview and Commercial Discovery Script"
doc_id: "FS-V0-013"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Research"
last_updated: "2026-09-27"
---

# Interview / Commercial Discovery

This script operationalizes the baseline `06_DELIVERY/09_USER_INTERVIEW_QUESTION_BANK.md`.

## Current workflow
- 你现在 Release 前怎么做这些检查？
- 哪一步最浪费时间？
- FirmwareSight 真正替代了哪一步？
- 没有它你会回到什么方法？

## Export / handoff
- 分析结果还需要给谁看？
- 对方实际会打开什么格式？
- self-contained file 比 cloud link 是否更实用？为什么？

## Repeated analysis / History
- 一次 Release 周期你重复检查几次？
- 什么事件触发重新检查？
- 自动 watch 会省事还是制造噪声？

## CI
- CI 中 Gate fail 对你是价值还是干扰？
- 谁拥有/修改 release policy？

## Stack evidence
- 现在是否使用 `.su` / Static Stack Analyzer？
- 最近一次 stack sizing/overflow 事故是什么？

## Symbol retention
- 有没有几个月后重新找 ELF/AXF/MAP 的经历？
- 今天怎么保存和定位？

## Component evidence
- 你项目里哪个来源才是 dependency version authority？
- ESP-IDF 项目是否提交 `dependencies.lock`？

## CRA / external review
- 客户/法规是否已经要求 SBOM/provenance/release evidence？
- 是否存在真实截止时间？
- 具体要的 evidence 是什么？

Do not ask whether the user “needs FirmwareSight for CRA compliance”.

## Repeat/value signal
- 下一次 Release 你会主动打开它吗？为什么？
- 哪些只是方便？哪些真的降低 Release 风险？

## Payment
Use concrete price anchors decided by the research operator before sessions.
For each anchor:
- 你会自己购买还是需要公司报销？
- 谁批准？
- 如果不会买，最主要原因是什么？
- 愿不愿意拿一次真实 Release 做 Team Pilot？

Record the actual anchor and answer. Do not claim the anchor is a frozen product price.
