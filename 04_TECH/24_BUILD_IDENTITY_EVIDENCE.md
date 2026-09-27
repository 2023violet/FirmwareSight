---
title: "Build Identity Evidence Model"
doc_id: "FS-TECH-025"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
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
