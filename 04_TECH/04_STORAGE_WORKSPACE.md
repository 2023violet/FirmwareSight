---
title: "Local Workspace and Storage"
doc_id: "FS-TECH-005"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Local Storage

## Workspace

项目根目录可选：

```text
.firmwaresight/
├── project.json
├── index.sqlite
├── snapshots/
│   └── <snapshot-id>.json
├── reports/
├── cache/
└── logs/
```

`firmwaresight.toml` 可放在 repo root，适合进入 Git；`.firmwaresight/` 默认本地、不要求提交。

## What goes into Git

建议：
- `firmwaresight.toml`
- team gate policy
- optional component declarations

不建议：
- local DB
- cache
- absolute paths
- user-specific UI state
- artifact copies

## Snapshot

Snapshot 必须自包含足够信息，使原 ELF 被删除后仍能：
- 查看已记录分析；
- 做 metadata diff；
- 复核 evidence locator。

但不复制整个 firmware binary，除非用户显式选择 archive。

## SQLite purpose

只用于：
- snapshot index；
- search/filter；
- history；
- project metadata。

SQLite 不是唯一真相。
Portable snapshot/export 使用 versioned JSON schema。

## Schema migration

任何持久化 schema 修改：
- migration；
- fixture；
- downgrade/backup strategy；
- ADR if semantic model changes。

禁止启动时静默破坏旧 DB。
