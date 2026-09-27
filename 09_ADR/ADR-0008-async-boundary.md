---
title: "ADR-0008 Async Boundary"
doc_id: "ADR-0008"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Context
The Rust census shows Tokio is highly prevalent in actual async products but explicitly warns against forcing it into simple/synchronous cores.

FirmwareSight MVP is mostly CPU/file/SQLite work.

# Decision
- Core synchronous.
- CLI synchronous by default.
- Desktop may use Tauri/Tokio at application edge.
- blocking parse/database jobs run off UI executor.
- Tokio types never appear in Core API.

# Consequence
Async remains replaceable infrastructure and domain tests remain simple.

# Trigger for expansion
Real network/IPC/background scheduling requirements.
