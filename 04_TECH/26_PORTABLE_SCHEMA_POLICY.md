---
title: "Portable Schema Policy"
doc_id: "FS-TECH-027"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
---

# Portable Schema Policy

## Scope

以下是对外可持久/可移交 contract：
- release manifest
- gate results
- accepted reviews
- CLI machine-readable export（对应版本化 schema）

它们比同车发布的 Desktop IPC 更严格。

## Rules

- stable object 默认 `additionalProperties: false`
- 显式 extension point 使用 `extensions`
- `$id` 使用稳定 URN，避免伪域名
- schema_version required
- semantics 改变需要 schema major
- additive optional field 仍需 contract tests
- unknown input schema major → hard error，不静默重释

## v0.4 schemas

- `urn:firmwaresight:schema:release-manifest:1`
- `urn:firmwaresight:schema:gate-results:1`
- `urn:firmwaresight:schema:accepted-reviews:1`

## Hosting

真正公开 schema URL/域名在品牌/domain clearance 后确定；URN 在此之前保持稳定。
