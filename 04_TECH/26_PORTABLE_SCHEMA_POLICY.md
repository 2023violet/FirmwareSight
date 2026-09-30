---
title: "Portable Schema Policy"
doc_id: "FS-TECH-027"
product: "FirmwareSight"
version: "0.5.1"
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

## P3 落地（2026-09-30）：Gate 文档与 project 配置契约

- `gate-results:1` 保持 v1，顶层属性表一个不动：`policy_sha256`、
  `project_config_schema_version`、`baseline_snapshot_id`、`overall_effective_severity` 一律走
  `extensions`。五态词表未变，effective_severity 仍是独立字段（ADR-0023）。
- `accepted-reviews:1` 只加一件事：**可选** 属性 `original_state`（`enum ["REVIEW"]`），required
  列表不变。契约测试两头都跑 —— P3 之前写下的最小 v1 记录仍然通过，P3 自己产出的每条记录都带
  `original_state = "REVIEW"`。
- 新增 `urn:firmwaresight:schema:project-config:1`（`schemas/project-config.schema.json`）：
  `firmwaresight.toml` 自此是公开契约，语义见 `04_TECH/08`。
- 三份契约共用 `firmwaresight_report::schema_check`，draft/2020-12 的无依赖子集；未实现的关键词
  返回失败而不是放行。`format: date-time` 按存储实际写入的 UTC 形状断言，而不是当注解忽略。
