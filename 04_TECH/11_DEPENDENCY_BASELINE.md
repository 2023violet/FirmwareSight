---
title: "Dependency Baseline"
doc_id: "FS-TECH-012"
product: "FirmwareSight"
version: "0.5.1"
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


## P3 project policy and provenance (ADR-0027, 2026-09-29)

A fifth library crate, `crates/firmwaresight-project`, was admitted together with two direct
dependencies. Both were already resolved in `Cargo.lock` before the crate existed, so the lock gained
one package stanza — the new first-party crate — and no third-party package:

| Crate | Locked version | License | Why it is needed | Why not the existing set |
| --- | --- | --- | --- | --- |
| `toml` | `1.1.6+spec-1.1.0` | MIT OR Apache-2.0 | `firmwaresight.toml` is TOML, and its keys must be typed, validated and written back | `serde_json` reads JSON; a hand-written TOML reader would be new unreviewed parser code in the product |
| `regex` | `1.13.1` | MIT OR Apache-2.0 | `[version] pattern` is a user-declared regex (`04_TECH/08`), matched against the workspace tag | no matching capability exists in `std`, and Core must stay dependency-free |
| `sha2` | `0.11.0` | MIT OR Apache-2.0 | policy fingerprint, run id, Release Notes digest | reused at the version `firmwaresight-artifact` already locks |
| `thiserror` | `2.0.21` | MIT OR Apache-2.0 | typed library errors for config and save failures | reused |
| `serde` | `1.0.229` | MIT OR Apache-2.0 | the config model | reused |

`tracing` is authorized by the prompt and was **not** installed: nothing in this crate needed a
diagnostic that a typed error does not already carry. An authorized dependency is not an installed one.

Two boundaries this admission does not move:

- The `regex` ban in the MAP parser section is unchanged. "No universal regex parser" forbids sniffing
  a *file format* with patterns; matching a user-declared version pattern from config is a different
  job, and the pattern comes from the release owner rather than being guessed at the artifact.
- `gix`, `anyhow` and `rayon` stay banned. Git facts come from the read-only system client with a
  bounded timeout (`04_TECH/22`), errors stay typed, and the adapter is synchronous.

Neither `toml` nor `regex` may be used by `firmwaresight-core`: Core keeps its empty `[dependencies]`
(`AGENTS.md` 3), so config semantics and pattern matching are turned into plain facts by the project
crate before Core ever sees them.

### P3 exposure edges (2026-09-30)

The CLI and Desktop work added **no third-party package**. What changed is which first-party crate is
allowed to see which, and two test-only edges:

- `fwsight` gains `firmwaresight-project` as a direct dependency: reading `firmwaresight.toml`, the
  read-only Git probe and the run fingerprint are the project crate's job, and a second implementation
  in the CLI is the failure mode ADR-0027 exists to prevent.
- `fwsight` dev-dependency on `firmwaresight-report`, so `gate --json` output is validated against the
  published `gate-results` v1 schema by the CLI's own tests rather than only by the report crate's.
- `firmwaresight-project` dev-dependencies on `firmwaresight-report` and `serde_json`: prompt §55 asks
  for the P2 contract-test helper to be reused rather than copied, and the helper lives in `report` as
  `schema_check`. `serde_json` is there because a JSON Schema can only be checked against a JSON tree.

Dev-dependencies do not enter the shipped graph, so the binary and the portable artifact set are
unchanged. `cargo deny check licenses bans sources advisories` is re-run at stage close.
