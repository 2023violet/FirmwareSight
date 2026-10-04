---
title: "Build Identity Evidence Model"
doc_id: "FS-TECH-025"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-10-04"
---

# Build Identity Evidence Model

## Principle

“Git 当前是 commit X” 不等于“这个 ELF 一定由 commit X 构建”。

FirmwareSight 必须把 artifact 身份与 workspace provenance 分开。

## Fields

### Artifact identity — Observed
- SHA-256
- byte size
- parsed architecture
- build-id if present in artifact

### Project/release version
允许来源：
- embedded/version section: Observed
- exact Git tag related to linked workspace: Derived context
- user declaration: Declared
- unavailable: Unknown

不同来源不得互相覆盖；UI 可显示 conflict。

### Git provenance
- repo root
- HEAD commit
- exact tag
- dirty state
是 workspace evidence，而不是任意 binary 的绝对 build proof。

### Time
必须拆分：
- `artifact_mtime`: filesystem observation
- `imported_at`: FirmwareSight observation
- `build_time`: 只有 artifact/toolchain evidence 或用户声明时才存在

禁止把 mtime/import time 叫 build time。

## Gate implications

- missing Git never aborts artifact analysis；
- Git-dependent rule → UNKNOWN if unavailable；
- policy `on_unknown` 决定 effective severity；
- version mismatch BLOCK 只有在被比较的 evidence sources 明确定义后成立。

## Release Notes identity：字节即证据（ADR-0028）

`observe_release_notes` 对文件原始字节做 SHA-256；路径上没有 decode、没有 text mode、也没有 newline 处理。
`ADR-0028` 把这一点定为规则，而不是巧合：

- LF 与 CRLF 是两份不同的 evidence；同一篇 notes 的两种行尾是两个不同的 release。
- Release Notes digest 进入 Gate canonical input，所以 Gate run id 与 release id 可能随 checkout 的行尾移动，
  而 verdict 不变 —— identity 与 correctness 是两个问题，把它们混在一起才会让 normalization 变成一个
  presentation fix 溜进来。
- bundle 发布它 hashing 的那份字节，`verify_bundle` 仍然可以从 bundle 自身字节重算 release id。
- FirmwareSight 不 normalize、不改写、不替换 Release Notes，也不写 `.gitattributes`；被分析的项目目录不属于
  这个产品（`AGENTS.md` §7）。

`release.require_clean_git = false` 仍然是有效的项目显式 policy 选项，ADR-0028 明确保留它，不强制为 `true`。
它的代价必须与开关一起被读到：`false` 时，工作区字节差异（行尾在内）可以改变 Release Notes digest、Gate run id
与 release id，而 `git.clean` 不会 BLOCK，屏幕上也没有一句话说明为什么。推荐的项目做法是在**用户自己的仓库**
用 `.gitattributes` 固定 byte-sensitive 的 release 文件；那是给项目仓库的建议，不是产品行为。Diagnostics 在报告
这个 flag 的同时报告这句话。

契约测试：`crates/firmwaresight-project/tests/bundle_builder.rs` 的
`a_notes_file_that_differs_only_in_line_endings_is_a_different_release`。它是 ADR-0028 的可执行表述，不可删除，
也不可弱化；改它等于改这条 ADR。
