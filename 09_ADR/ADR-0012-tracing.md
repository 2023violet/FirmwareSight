---
title: "ADR-0012 Structured Tracing"
doc_id: "ADR-0012"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Decision
Adopt `tracing` + `tracing-subscriber` at project start.

# Why
FirmwareSight parses untrusted artifacts and will need reproducible field diagnostics. Plain `println!` is insufficient.

# Privacy
Logs must not include source/artifact content or full sensitive paths by default.
