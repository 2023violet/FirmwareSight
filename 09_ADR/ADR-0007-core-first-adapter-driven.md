---
title: "ADR-0007 Core-first Adapter-driven Architecture"
doc_id: "ADR-0007"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Context
The supplied 2026 Rust product census identifies `Core-first, Adapter-driven, Product-specific Shell` as the reusable cross-product engineering principle.

FirmwareSight has two interfaces (Desktop/CLI), multiple artifact adapters and local persistence.

# Decision
Formalize Ports & Adapters boundaries around a headless Rust Core.

# Rules
- domain facts only in Core；
- external technologies implement adapters；
- use-case commands sit in application layer；
- interface layer never owns business truth；
- no adapter type leaks into Core public contracts。

# Consequence
Some mapping code is accepted to prevent infrastructure coupling.

# Revisit
No planned revisit; implementation may simplify module names but not dependency direction.
