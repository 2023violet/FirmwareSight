---
title: "ADR-0017 Git CLI Provenance Adapter"
doc_id: "ADR-0017"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Decision
Use installed Git CLI for initial read-only provenance.
Do not add `gix` at bootstrap.

# Constraints
- no shell;
- machine-readable output;
- timeout;
- typed errors;
- degrade to Unknown if Git unavailable.

# Why
Small scope, minimum dependency, matches developer repos.

# Revisit
Use gix if Git-less operation becomes a validated requirement.
