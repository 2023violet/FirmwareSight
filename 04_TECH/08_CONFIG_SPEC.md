---
title: "Project Configuration Specification"
doc_id: "FS-TECH-009"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# `firmwaresight.toml`

## Goals

- 可以进入 Git；
- human-readable；
- team policy 可复现；
- 不含 secrets；
- 相对路径优先。

## Draft

```toml
schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "bin", "map"]

[memory]
flash_budget = 262144
ram_budget = 131072

[version]
source = "git_tag"
pattern = "^v(?P<version>\\d+\\.\\d+\\.\\d+)$"

[release]
require_clean_git = true
require_release_notes = true

[diff]
flash_growth_review_bytes = 4096
ram_growth_review_bytes = 2048

[sbom]
enabled = false
```

## Rules

- unknown keys：warning in same major schema；
- invalid type：hard error；
- unsupported schema_version：hard error；
- config error 不允许被默认值悄悄覆盖。

正式 JSON Schema / TOML validation 在实现阶段冻结。


## v0.4 Unknown policy

Gate rule 不得把证据缺失静默当 PASS。

每个需要外部 evidence 的 rule 必须有明确：
`on_unknown = "review" | "block"`

UNKNOWN 是 finding state；`on_unknown` 只映射 effective severity，不改变 finding 的 UNKNOWN 事实。

Git 不可用时：
- artifact import/Analyze 继续；
- Git fields = Unknown；
- Git-dependent rule 按 `on_unknown` 处理。
