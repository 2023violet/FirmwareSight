---
title: "Error Model"
doc_id: "FS-ENG-004"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project"
last_updated: "2026-09-26"
---

# Error Model

## Error taxonomy

- `InputNotFound`
- `UnsupportedFormat`
- `MalformedArtifact`
- `ParserCapabilityMissing`
- `ConfigInvalid`
- `GitUnavailable`
- `GitStateConflict`
- `StorageFailure`
- `ExportConflict`
- `ExportFailure`
- `PolicyInvalid`
- `InternalBug`

## User message structure

每个错误必须有：
1. What happened
2. Why we know
3. What user can do
4. Diagnostics ID / copy details

例：

**Unsupported MAP format**

FirmwareSight could not match this file to the supported GNU ld MAP adapter.

Detected:
`ARM Linker ...`

Next:
Import the ELF without MAP, or use a supported GNU ld MAP file.

## Internal bug

捕获后：
- 显示 operation id；
- 提供 copy diagnostics；
- 不显示原始 stack trace 给普通用户；
- debug mode 可保留完整 trace。
