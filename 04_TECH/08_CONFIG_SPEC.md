---
title: "Project Configuration Specification"
doc_id: "FS-TECH-009"
product: "FirmwareSight"
version: "0.5.1"
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


## v1 冻结（P3 Release Gate，2026-09-29）

本节就是上面 “正式 JSON Schema / TOML validation 在实现阶段冻结” 等待的那次冻结。实现是
`crates/firmwaresight-project/src/config.rs`，契约是 `schemas/project-config.schema.json`
（`urn:firmwaresight:schema:project-config:1`），可直接复制的样例是 `examples/firmwaresight.toml`。

支持的形状（`schema_version = 1`）：

```toml
schema_version = 1

[project]
name = "motor-controller"

[artifacts]
required = ["elf", "map"]          # v1 词汇：elf | map | bin | hex，必须包含 elf

[memory]
flash_budget = 262144              # 省略 = 规则 N/A；= 0 是 hard error
ram_budget = 131072

[version]
source = "git_tag"                 # MVP 只有一个来源
pattern = "^v(?P<version>\d+\.\d+\.\d+)$"

[release]
require_clean_git = true
require_release_notes = true
release_notes_path = "RELEASE_NOTES.md"   # 必须 project-relative
# expected_commit = "<40 或 64 位小写十六进制>"
# expected_version = "1.2.3"              # 需要 pattern 里有 version 捕获组

[diff]
flash_growth_review_bytes = 4096
ram_growth_review_bytes = 2048

[gate]
unknown_evidence_review_count = 1

[gate.on_unknown]                   # v0.4 要求的每规则处置，落在这里
git_clean = "review"
commit_matches_release = "review"
version_match = "review"
flash_budget = "block"
ram_budget = "block"
baseline_growth = "review"
release_notes = "review"

[sbom]
enabled = false                     # 记录并警告，不产生任何规则
```

硬错误（退出码 2，绝不静默套用默认值）：不支持的 `schema_version`、TOML 语法错误、类型错误、
`on_unknown` 取 `review|block` 之外的值、`artifacts.required` 缺少 `elf` 或使用了词汇表之外的词、
预算为 0、绝对路径或 `../` 越界、无法编译的 `pattern`、`expected_version` 没有可用的 `version`
捕获组、`expected_commit` 不是完整 Git object id、空的 `project.name`。

警告（继续求值）：major 1 的未知键（按 `memory.tcm_budget`、`gate.on_unknown.quantum_check` 这样的
点号路径列出）、`artifacts.required` 重复项、没有 `version` 捕获组的 `pattern`、
`[sbom] enabled = true`。诊断码：`ERR-CONFIG-7001..7008`。

保存规则：先验证再落盘；写同目录临时文件、读回比对，再 rename 替换，因此失败的保存不会留下 0 字节
配置；未知键无法保留时以 `ERR-CONFIG-7007` 拒绝写入，而不是悄悄删掉用户写过的东西。保存会从已验证
的模型重写文件，注释不保留 —— Release 页面在写入前必须把这一点说清楚。`schemas/project-config.schema.json`
关闭每个对象，而运行时的未知键仍是警告：契约描述已文档化的形状，加载器负责让新配置不破坏旧构建。

政策指纹哈希的是语义（`firmwaresight-core` 的 `canonical_policy_text`），不是注释或键序：同样的值
产生同一个 `policy_sha256`，任何改变求值的改动都会改变它。
