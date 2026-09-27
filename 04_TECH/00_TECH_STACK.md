---
title: "FirmwareSight Technology Stack v0.2"
doc_id: "FS-TECH-001"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Technology Stack

## 1. Product classification

FirmwareSight 是：
- local-first desktop developer/engineering tool；
- data-heavy inspector；
- headless CLI companion；
- mostly local file / SQLite workload；
- deterministic evidence application。

它不是：
- web SaaS；
- mobile app；
- high-frequency GPU renderer；
- network-first service。

因此不能直接套用“通用 Rust 全家桶”。

## 2. Selected stack

| Layer | Choice | Status |
|---|---|---|
| Language | Rust 1.98.1, Edition 2024 | LOCKED |
| Workspace | Cargo workspace, resolver 3 | LOCKED |
| Core serialization | Serde | LOCKED |
| Library errors | thiserror | LOCKED |
| App error context | anyhow only at binary/app boundary | LOCKED |
| Hash | SHA-256 via sha2 | LOCKED |
| ELF/Object | object crate | LOCKED |
| DWARF | gimli, optional/feature-scoped | LOCKED |
| CLI | clap | LOCKED |
| Logging | tracing + tracing-subscriber | LOCKED |
| Desktop | Tauri 2 stable | LOCKED |
| Frontend | React 19.3 + TypeScript + Vite 8 | LOCKED |
| JS package manager | pnpm 12.x pinned exact in repo | LOCKED |
| Local DB | SQLite | LOCKED |
| SQLite binding | rusqlite + bundled | LOCKED |
| Async runtime | Application edge only | CONDITIONAL |
| HTTP client | Reqwest + Rustls | APPROVED WHEN NEEDED |
| GPU | wgpu Native Island | APPROVED WHEN PROVEN |
| Server | none | NOT MVP |
| gRPC | none | NOT MVP |
| AI | none in trusted core | NOT MVP |

## 3. Why this combination

### Rust Core
FirmwareSight 的真正资产是：
- binary parsing；
- normalization；
- diff；
- evidence；
- gate；
- release manifest。

这些要求：
- correctness；
- predictable memory；
- cross-platform；
- headless reuse；
- CLI reuse。

因此 Rust 位于价值最高的核心层。

### Tauri 2 + React
FirmwareSight 的 UI 主要由：
- tables；
- filters；
- panels；
- diff views；
- forms；
- evidence inspector；
- report preview
构成。

这不是自定义 GPU editor，因此成熟 Web UI + thin desktop shell 更合适。

### SQLite + rusqlite
产品是 local-first、SQLite-only、结构化历史数据密集。
不需要多数据库 abstraction，也不需要 async ORM。

### Conditional Tokio
文件解析和 diff 本身是同步/CPU 工作。
Tokio 只解决 app edge 的异步协调，不应改变 Core API。

### No network baseline
MVP 可完全离线成立。提前引入 HTTP/TLS 会扩大 attack/dependency surface，却没有用户价值。

### No GPU baseline
100k symbol table 的问题首先应通过：
- indexing；
- pagination；
- virtualization；
解决，而不是 GPU。

## 4. Rejected defaults

### SQLx
现在不选：
- single-process SQLite-only；
- Core 不需要 async DB；
- rusqlite 更直接。

### egui / iced / Slint
不是“差”，而是当前产品 UI 约束没有证明 Native UI 的收益高于成熟 Web UI。

### GPUI / Floem
不承担 pre-1.0 / ecosystem risk，除非 UI/rendering 本身成为产品核心。

### Electron
没有必要随应用分发完整 Chromium runtime。

### wgpu
不提前优化。

### Axum/Tonic
当前没有 server/gRPC 产品面。

## 5. Lock policy

Baseline 锁 **major architecture**，lockfile 锁 **exact dependency graph**。

版本升级原则：
- patch/minor：CI + regression 后可常规升级；
- major：ADR；
- Tauri major：ADR；
- Rust toolchain：先在 CI validation branch 验证。
