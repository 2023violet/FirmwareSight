---
title: "IPC and Data Contracts"
doc_id: "FS-TECH-015"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# IPC / Data Contracts

## 1. Command philosophy

Tauri commands represent user use cases.

Good:
- `open_project`
- `import_artifact`
- `get_analysis_summary`
- `query_symbols`
- `compare_builds`
- `run_release_gate`
- `prepare_release_bundle`
- `attach_release_file`
- `get_release_attachments`

Bad:
- `read_file`
- `execute_shell`
- `run_sql`
- `get_any_path`
- `set_attachment_path`（把宿主路径交给 WebView 再交回来；选择只发生在原生对话框一侧，
  `ADR-0030` / `04_TECH/28` §4.4）

## 2. DTOs

IPC DTOs:
- serializable via Serde；
- bounded；
- UI-oriented but factual；
- contain stable IDs, not borrowed Rust references；
- never expose rusqlite/Tauri/object crate types。

`ReleaseAttachmentDto`（C1-U4，2026-10-10）是这条规则的一个新例子：它带 leaf 名、字节数、完整 SHA-256、
声明的 kind、`kind_basis: declared` 和 `provenance: unknown`，但不带路径——文件在 Rust 一侧被记住，
WebView 只拿到一个会话内句柄（`att-<pid>-<n>`）。`ReleaseAttachmentsDto.matchesGateRun` 同样由 Rust 计算：
判定“附件集合还是不是那次 run 所评的集合”需要恒等式规则，而那条规则只有一处实现
（`04_TECH/28` §4.2），屏幕不得有第二份。此处注册命令共 34 个（2026-10-10 由 30 增加 4 个附件命令），
由 `drift/ipc bindings` 与 `drift/ipc bindings unchanged` 两步核对生成物。

## 3. Large data

Symbol query:

```text
request:
snapshot_id
filter
sort
cursor/offset
limit <= configured max

response:
rows
next_cursor
total_or_estimate
capabilities
```

Never send the entire symbol table by default.

## 4. Versioning

Desktop frontend and backend ship together, so IPC does not need public API semantic versioning in MVP.

Still required:
- contract tests；
- generated fixtures；
- stable error codes；
- no silent field reinterpretation。

Portable schemas such as release manifest **are** versioned public contracts.

## 5. Type generation

v0.4 decision:
- Rust remains source of truth for IPC DTO semantics；
- v0.4.0 selects `ts-rs` at the Desktop/Application DTO boundary；
- generated TypeScript files are verified in CI and are not hand-edited；
- Core domain types remain generator-independent；

The generator choice is implementation-level unless it changes IPC semantics.

## 6. Error envelope

```text
code
message
operation_id
details?       // safe structured diagnostics
remediation?
```

Stack traces never cross to normal UI.
