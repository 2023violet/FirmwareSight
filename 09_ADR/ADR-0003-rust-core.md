---
title: "ADR-0003 Rust Core and Shared CLI"
doc_id: "ADR-0003"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted, strengthened in v0.2.0.

# Decision

FirmwareSight follows:

> Core-first, Adapter-driven, Product-specific Shell.

Pure Rust Core owns:
- domain；
- normalization；
- diff；
- evidence semantics；
- gate；
- release model。

Core does not depend on:
- Tauri；
- Tokio types；
- rusqlite；
- frontend；
- OS UI。

Desktop and CLI share Core.

# Consequences
- CLI becomes a real product interface, not a wrapper；
- parser/storage can be replaced behind ports；
- UI framework is not an architectural lock-in；
- unit tests do not require WebView/database。
