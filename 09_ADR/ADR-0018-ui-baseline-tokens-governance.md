---
title: "ADR-0018 UI Baseline and Design Tokens Governance"
doc_id: "ADR-0018"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
---

# ADR-0018 — UI 基准与 Design Tokens 治理

## Status

Accepted / Baseline in v0.4.0.

## Context

v0.3.0 存在三个真实问题：
1. 没有统一气质基准；
2. `design-tokens.json` 仍是 v0.1.0 draft 且缺交互/状态/密度 token；
3. Voice 文档 4 态与 Component Rules 5 态冲突。

外部设计调研选择 IBM/Carbon-inspired 作为最符合 Instrument-grade clarity 的单一气质参考，用户已确认所有开放项按默认建议冻结。

## Decision

### D1 — Single temperament reference

采用 `VoltAgent/awesome-design-md` 的 IBM `DESIGN.md` 作为唯一上游气质参考。

Pinned blob:
`dbef1c5b20357a8953e6832e5d383a866a66f11a`

它是反向整理的设计分析，不是 IBM 官方 Carbon 规范。

### D2 — Authority order

`治理红线 > frozen project assets/ADRs > root DESIGN.md > accepted screenshots > upstream reference`

### D3 — FirmwareSight overrides

- gradient: forbidden
- radius: token 6/8/10 + compact status pill
- accent: FirmwareSight accent token, not IBM brand blue
- table density: FirmwareSight table tokens
- font: system stack in MVP; Plex only future candidate
- motion: productive token timings only
- gray scale: existing restrained semantic scale
- shadow: overlays only
- Unknown: neutral gray / hollow icon / fact / next action
- theme: MVP Light only

### D4 — Five states are authoritative

`PASS / REVIEW / BLOCK / UNKNOWN / N/A`

Brand Voice 的旧 4-state table 被本 ADR supersede。

### D5 — Tokens

`assets/design-tokens.json` → v0.2.0, `baseline-authoritative`.

原 14 semantic colors 保持不动；补齐交互、focus、diff bg、overlay、pill、table、motion、chart、disabled/layout token。

### D6 — Governance artifacts

- root `DESIGN.md`
- `AGENTS.md` UI Rules
- `templates/DESIGN_CHECKLIST_TEMPLATE.md`
- accepted UI reference screen register

## Alternatives

- Vercel/WIRED/Cal.com 混合：拒绝，避免多基准漂移。
- 纯 Carbon 数值/组件复刻：拒绝；FirmwareSight 自有 tokens 是数值真源。
- Carbon React 直接成为依赖：不纳入 baseline；未来仅在真实组件成本证明后评估。
- 纯自研无外部气质锚：拒绝，增加重复审美争论。

## Consequences

Positive:
- 人与 AI 有共同视觉语言；
- 数值来源单一；
- UI reference 可审查；
- Unknown / Gate 状态语义一致。

Costs:
- 需要维护 token lint/checklist；
- 上游只能人工参考，不能自动同步。

## Revisit trigger

只有品牌重定位、MVP Dark theme、核心 UI 框架变化，或真实用户研究证明当前密度/可访问性失败时重新评估。
