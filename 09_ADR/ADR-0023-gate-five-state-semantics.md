---
title: "ADR-0023 Gate Five-State Semantics"
doc_id: "ADR-0023"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
---

# ADR-0023 — Gate Five-State Semantics

## Status
Accepted.

## Context
v0.3.0 PRD 的 4 态与 Component/UI 的 5 态冲突，且 UNKNOWN 与 REVIEW/N/A 语义不能合并。

## Decision
Canonical finding state:
`PASS / REVIEW / BLOCK / UNKNOWN / N/A`

另设 effective severity:
`PASS / REVIEW / BLOCK`

UNKNOWN 保持事实状态，由 `on_unknown` policy 映射 severity。

Accepted Review 生成 immutable audit record，不改原 finding/evidence。

## Consequences
领域模型、UI、JSON contract 统一；历史可审计。

## Revisit trigger
只有 Gate model 本身改变时。
