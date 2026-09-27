---
title: "Release Gate State Semantics"
doc_id: "FS-TECH-028"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
---

# Release Gate State Semantics

## Problem resolved

v0.3.0 PRD 曾写 4 态：
PASS / REVIEW / BLOCK / NOT_APPLICABLE

但组件规则和已接受 UI 使用：
PASS / REVIEW / BLOCK / UNKNOWN / N/A

这不是纯 UI 差异，而是核心领域模型冲突。

## Canonical finding state — 5

### PASS
规则已确定求值且满足。

### REVIEW
规则已确定求值，但 policy/人需要 disposition。

### BLOCK
确定性阻断条件失败。

### UNKNOWN
缺少/无法验证 evidence，规则不能确定求值。

### N/A
规则对该场景本来就不适用。

## Effective severity

`finding.state` 与 `effective_severity` 分开：

- PASS → PASS
- REVIEW → REVIEW，接受后可使 run 的 effective severity 下降，但 finding 本身仍是 REVIEW
- BLOCK → BLOCK
- UNKNOWN → 由 rule policy `on_unknown = review | block` 映射
- N/A → neutral

这样不会把 UNKNOWN 改写成 BLOCK/REVIEW，从而丢失“证据不足”事实。

## Review acceptance

接受 Review 必须生成 immutable audit record：
- finding_id
- actor
- timestamp
- reason
- original_state

不得修改 evidence，不得把历史 finding 变 PASS。

## Manual N/A

只有 rule 明确允许 manual N/A 时：
- actor
- timestamp
- reason
必须记录。

## Bundle

Release Bundle 包含：
- `gate_results.json`
- `accepted_reviews.json`

BLOCK effective severity 存在时不能创建“ready/pass” bundle。
