---
title: "External Technology Verification"
doc_id: "FS-RSCH-006"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Research"
last_updated: "2026-09-26"
---

# External Technology Verification — 2026-09-26

This file records current official-source checks made during the v0.2.0 technical-baseline research that is retained inside v0.4.0.

## Rust
Official Rust blog:
- Rust 1.98.1 published 2026-09-03.
- v0.2.0 pins this release for bootstrap reproducibility.

Source:
https://blog.rust-lang.org/releases/latest/

## React
Official React docs:
- latest documented major line: React 19
- latest version listed at verification: 19.3

Source:
https://react.dev/versions

## Vite
Official Vite:
- Vite 8 released 2026-03
- Vite 8.1 released 2026-06
- current guide uses Rolldown-based Vite line
- Vite requires a supported modern Node version

Sources:
https://vite.dev/guide/
https://vite.dev/blog/announcing-vite8

## Node
Official Node release page at verification:
- Node 24 = LTS
- Node 26 = Current

FirmwareSight chooses Node 24 LTS for build reproducibility.

Source:
https://nodejs.org/en/about/previous-releases

## pnpm
Official pnpm docs/GitHub at verification:
- pnpm 12 is current release line
- 12.7.0 released 2026-09-25

Sources:
https://pnpm.io/installation/
https://github.com/pnpm/pnpm/releases

## Tauri
Official Tauri v2 documentation remains the stable product line.
Tauri 3 visible upstream is alpha and is not selected.

Updater docs state update signatures are mandatory for the updater; the signing private key is an operational critical asset.

Sources:
https://v2.tauri.app/
https://v2.tauri.app/plugin/updater/
https://v2.tauri.app/distribute/pipelines/github/

## Parsing
`object` supports a unified object-file read interface including ELF.
`gimli` provides DWARF read/write support.

Sources:
https://docs.rs/object/latest/object/
https://docs.rs/gimli/latest/gimli/

## SQLite
Current rusqlite documentation explicitly recommends `bundled` for programs controlling their own SQLite database because it avoids system SQLite version/linking issues.

Source:
https://docs.rs/rusqlite/latest
