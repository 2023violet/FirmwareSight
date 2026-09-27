---
title: "ADR-0010 Artifact Parsing Stack"
doc_id: "ADR-0010"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Status
Accepted.

# Decision
- `object` is the primary ELF/object parser.
- `gimli` is optional DWARF parser.
- MAP files use project-owned toolchain adapters.
- no second general ELF parser in MVP.

# Why
`object` already provides common object/ELF reading while preserving access to lower-level ELF data.
`gimli` is specialized for DWARF.

# Rule
Missing DWARF or MAP reduces capability; it must not invalidate an otherwise valid ELF import.

# Revisit
Only if a supported toolchain exposes required ELF facts that `object` cannot safely provide.
