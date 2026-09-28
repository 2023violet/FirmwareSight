---
title: "ADR-0020 Validation Sequence"
doc_id: "ADR-0020"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Product / Architecture"
last_updated: "2026-09-27"
---

# ADR-0020 — V0 Prototype + P0 Vertical Slice Parallel Validation

## Status
Accepted.

One clause of this ADR is superseded by `ADR-0025-conditional-pre-g1-analyze-implementation.md`
(2026-09-28): `P1 Product MVP implementation 只有两者都 PASS 后开始。` After P0 `PASS`, the architect may
authorize a bounded, reversible Pre-G1 Analyze slice without waiting for V0. The parallel-validation
decision itself, `G1 = V0 PASS + P0 PASS`, and every consequence recorded below are unchanged, and the
superseded sentence is left in place rather than rewritten, because it is what this ADR decided at the
time it decided it.

## Context
立项过程原先要求先 UI prototype 真人验证，v0.3.0 又要求先 technical slice。两者未解释，造成流程冲突。

## Decision
授权后并行：
- V0 Workflow Prototype Validation
- P0 Technical Vertical Slice

P1 Product MVP implementation 只有两者都 PASS 后开始。

## Alternatives
- 只先写代码：可能把错误 workflow 做实。
- 只先做原型：可能验证无法可靠实现的承诺。

## Consequences
多一条并行工作线，但更早同时消除 desirability 与 feasibility 风险。

## Revisit trigger
无；每个新主流程都可复用这一验证模式。
