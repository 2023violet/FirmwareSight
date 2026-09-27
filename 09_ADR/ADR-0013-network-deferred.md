---
title: "ADR-0013 Network Client Deferred"
doc_id: "ADR-0013"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Context
General Rust products often use Reqwest + Rustls once HTTP exists, but FirmwareSight MVP is offline-first.

# Decision
Do not include HTTP client/TLS dependencies in MVP bootstrap.

Approved future route:
`Reqwest + Rustls`

# Trigger
An authorized feature actually requires HTTP:
- signed update metadata；
- CVE feed；
- licensing；
- optional cloud。

# Consequence
Smaller attack/dependency surface today.
