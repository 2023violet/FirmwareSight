---
title: "IPC and Data Contracts"
doc_id: "FS-TECH-015"
product: "FirmwareSight"
version: "0.5.0"
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

Bad:
- `read_file`
- `execute_shell`
- `run_sql`
- `get_any_path`

## 2. DTOs

IPC DTOs:
- serializable via Serde；
- bounded；
- UI-oriented but factual；
- contain stable IDs, not borrowed Rust references；
- never expose rusqlite/Tauri/object crate types。

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
