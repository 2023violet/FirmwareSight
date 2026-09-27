---
title: "Technical Reference Register"
doc_id: "FS-RSCH-003"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-26"
---

# Technical References

Baseline: 2026-09-26

## Supplied research
See:
`08_RESEARCH/SOURCE_REPORTS/`

The main census establishes:
- Core-first / adapter-driven / product-specific shell；
- Serde as near-default boundary infrastructure；
- conditional Tokio；
- Tauri/Web as strong default for ordinary cross-platform desktop；
- SQLite for stateful local desktop；
- rusqlite vs SQLx split；
- Reqwest/Rustls when HTTP exists；
- wgpu only when GPU need is proven；
- packaging/updater as part of product engineering。

## Official verification

### Rust
https://blog.rust-lang.org/releases/latest/

### Tauri
https://v2.tauri.app/
https://v2.tauri.app/concept/architecture/
https://v2.tauri.app/security/capabilities/
https://v2.tauri.app/plugin/updater/
https://v2.tauri.app/distribute/pipelines/github/

### React
https://react.dev/versions

### Vite
https://vite.dev/guide/
https://vite.dev/blog/announcing-vite8

### Node
https://nodejs.org/en/about/previous-releases

### pnpm
https://pnpm.io/installation/
https://github.com/pnpm/pnpm/releases

### object / gimli
https://docs.rs/object/latest/object/
https://docs.rs/gimli/latest/gimli/

### rusqlite
https://docs.rs/rusqlite/latest

### CycloneDX / SPDX
https://cyclonedx.org/specification/overview/
https://spdx.dev/use/specifications/

## Rule

Research is evidence.
ADR is decision.

Never infer “popular = mandatory”.
