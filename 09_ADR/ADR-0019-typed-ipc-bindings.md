---
title: "ADR-0019 Typed IPC Bindings"
doc_id: "ADR-0019"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
---

# ADR-0019 — Typed IPC Bindings

## Status
Accepted.

## Context
v0.3.0 明确 Rust 是 IPC DTO 语义真源，但生成器悬空，容易导致前端手写重复类型。

## Decision
使用稳定 `ts-rs` 路线为 Desktop/Application IPC DTO 生成 TypeScript 类型。
Core domain 不依赖生成器；先做显式 mapping 到 IPC DTO。

## Alternatives
- 手写 TS mirror：拒绝。
- Tauri-specific generator 直接耦合 Core：拒绝。
- 无类型 JSON：拒绝。

## Consequences
增加一个边界层生成步骤，但显著降低 Rust/TS contract drift。

## Revisit trigger
如果 `ts-rs` 无法覆盖真实 DTO contract，评估替代 generator；保持 Core 解耦。
