---
title: "Repository Structure v0.2"
doc_id: "FS-ENG-001"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Repository Structure

```text
firmwaresight/
├── AGENTS.md
├── README.md
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── .node-version
├── apps/
│   ├── cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── desktop/
│       ├── ui/
│       │   ├── package.json
│       │   └── src/
│       └── src-tauri/
│           ├── Cargo.toml
│           ├── capabilities/
│           └── src/
├── crates/
│   ├── firmwaresight-core/
│   ├── firmwaresight-artifact/
│   ├── firmwaresight-storage/
│   └── firmwaresight-report/
├── fixtures/
│   ├── elf/
│   ├── map/
│   ├── hex/
│   ├── malformed/
│   └── projects/
├── schemas/
├── docs/
├── scripts/
└── .ai/
```

## Crate budget

Phase 0 maximum library crates: **4**.

A fifth shared app crate requires evidence of real duplication.

## Dependency direction

```text
core           <- artifact
core           <- storage
core           <- report
core/adapters  <- apps/cli
core/adapters  <- apps/desktop
```

`core` must have the smallest dependency set.

## Frontend

Frontend is one workspace package at MVP.
Do not split design system, icons, table, or feature packages before independent reuse exists.
