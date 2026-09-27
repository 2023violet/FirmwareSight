---
title: "ADR-0009 SQLite via rusqlite"
doc_id: "ADR-0009"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Context
FirmwareSight is local-first, SQLite-only and requires indexed historical data. The supplied Rust census distinguishes local synchronous desktop use (`rusqlite`) from async/multi-database products (`SQLx`).

# Decision
Use:
- SQLite
- rusqlite
- bundled SQLite

SQLite is the local structured source of truth.

# Rejected
SQLx for MVP.

# Why bundled
Cross-platform desktop product should not depend on a possibly missing/old system SQLite.

# Consequences
- simpler storage layer；
- consistent SQLite binary；
- larger build contribution accepted；
- database work remains behind storage adapter。
