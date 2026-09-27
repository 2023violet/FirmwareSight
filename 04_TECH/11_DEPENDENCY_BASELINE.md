---
title: "Dependency Baseline"
doc_id: "FS-TECH-012"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Dependency Baseline

## Core dependencies

### `serde`
Purpose:
- versioned snapshots；
- release manifest；
- config contracts；
- IPC DTOs。

Rule:
Do not serialize arbitrary internal state merely because derive is convenient.

### `thiserror`
Purpose:
typed library error enums.

Rule:
Errors must preserve category and source; no stringly typed domain errors.

### `sha2`
Purpose:
SHA-256 for artifacts and release bundles.

Rule:
Stream hash from file; do not require whole file in memory just to hash.

## Artifact dependencies

### `object`
Primary binary/object parser.
Use high-level Object APIs first.
Use ELF-specific low-level APIs only where needed.

### `gimli`
Only for DWARF-derived evidence.
DWARF absence must degrade capability, not fail an ELF import.

### MAP parser
Project-owned adapters.
No “universal regex parser”.

MVP adapter:
- GNU ld

Future:
- ArmClang
- IAR

## Storage

### `rusqlite` + `bundled`
Reason:
- local desktop；
- SQLite-only；
- synchronous access；
- consistent SQLite across OS；
- fewer deployment surprises。

No SQLx in MVP.

## CLI

### `clap`
Derive API allowed.
CLI output contracts remain independent of clap types.

## Observability

### `tracing`
### `tracing-subscriber`

Logging must be structured enough to filter by:
- operation_id；
- project_id when safe；
- adapter；
- phase；
- elapsed_ms。

Potential `tracing-appender` is allowed only when persistent rotating log is implemented.

## Desktop

Tauri plugin policy:
- add plugin only for real use;
- frontend never gets shell plugin by default;
- filesystem access stays behind Rust commands;
- dialog plugin is acceptable for explicit user file selection;
- updater plugin is deferred until signing phase.

## Frontend

Baseline:
- react
- react-dom
- TypeScript
- Vite

Allowed minimal helpers:
- icon library when design system needs it;
- testing packages.

No baseline:
- Redux；
- Zustand；
- large chart library；
- full UI framework；
- Tailwind。

## Dependency admission checklist

Before adding:
- feature requiring it；
- maintained upstream；
- compatible license；
- security/advisory status；
- transitive size；
- native build implications；
- WASM/WebView implications if frontend；
- deterministic/offline impact。

## v0.4 IPC type dependency

### `ts-rs`
Approved for Desktop/Application DTO generation.

Rules:
- no Tauri type leakage into Core
- generated TS is not hand-edited
- CI verifies generated output is current
- dependency stays at boundary crate/module

Do not add a second generator without removing/justifying the first.
