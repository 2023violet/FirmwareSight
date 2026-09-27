---
title: "Decision Summary"
doc_id: "FS-AI-003"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-09-27"
---

# Decisions — v0.5.0

## Existing core
- Evidence First
- Deterministic trusted core
- Local First
- Explicit Unknown
- Core-first / Adapter-driven / Product-specific Shell
- Rust Core + shared CLI
- Tauri 2 + React/TS shell
- SQLite/rusqlite bundled
- five-state Gate
- dual firmware memory accounting
- strict portable schemas
- IBM/Carbon-inspired temperament + FirmwareSight overrides

## New v0.5 decisions
- UI-07 source accepted; seven-screen set complete.
- Repository code-area plan frozen as `FS-ENG-009`.
- ADR-0024: expert exploration uses E1/E2/E3/GX candidate namespace; current P0–P4 MVP lifecycle is unchanged.
- Expert round directional decisions are retained only as candidates.
- Market claims carry explicit verification class; negative-search claims never become universal facts.
- Enhanced export/integration/evidence features do not enter current MVP.
- `.su` stack evidence is a strong E2 candidate requiring a future ADR; full call graph excluded.
- ESP-IDF `dependencies.lock` is the first evidence-backed component-drift source candidate.
- SBOM evidence completeness precedes adding more SBOM formats.
- generic changelog, native PDF, LSP, OTA, binary reverse diff and AI verdicts are not adopted.

## Current next work
After explicit authorization only:
- V0 workflow prototype;
- P0 technical vertical slice;
in parallel.
