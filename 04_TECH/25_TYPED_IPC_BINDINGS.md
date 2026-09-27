---
title: "Typed Rust to TypeScript IPC Bindings"
doc_id: "FS-TECH-026"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
---

# Typed IPC Bindings

## v0.4.0 decision

选择 `ts-rs` 作为 Rust→TypeScript DTO 类型生成基线，作用范围仅限 **Desktop/Application IPC DTO boundary**。

不采用：
- 手写 100+ 重复 TS domain types；
- 在 Core 暴露 Tauri 类型；
- 将未稳定的 Tauri-specific generator 绑定进 Core。

## Boundary

```text
Core domain
   ↓ explicit mapping
IPC DTOs (Serde + ts-rs export)
   ↓
TypeScript generated types
```

Generated files：
- 由 CI 验证 clean
- 不手改
- 需要稳定、有限、有界 DTO

## Why

FirmwareSight 需要前后端类型一致，但 Desktop shell 仍必须可替换。类型生成属于 adapter contract，不属于领域模型。

## Generator change

未来更换 generator 属实现工具变更；只要 IPC 语义不变，不需要架构 ADR。若生成方案要求 Tauri/Core 耦合，则必须 ADR。
