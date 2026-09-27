---
title: "Coding Standards"
doc_id: "FS-ENG-002"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Coding Standards

## Rust

- `rustfmt`
- `clippy -D warnings` for project code
- Result-based error handling
- library code 不 `unwrap()` user input
- parsing code 不 panic
- domain model 尽量使用 enum/newtype 表达不可能状态
- public API 有 rustdoc
- avoid premature async

## TypeScript

- strict mode
- no `any` without documented boundary
- domain DTO generated/shared from a stable contract when feasible
- UI state 与 domain state 分离
- no parser/business rules in components
- components small enough to review

## Naming

Rust:
- `snake_case`
- types `PascalCase`

UI:
- component `PascalCase`
- file 与主 component 同名

Domain terms 必须使用 glossary 中名称。

## Logging

日志必须可分类：
- INFO lifecycle
- WARN degraded capability
- ERROR operation failure
- DEBUG parser detail

默认日志不得包含 source code 或完整用户绝对路径。
